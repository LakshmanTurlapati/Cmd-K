//! Process inspection for terminal context reading.
//!
//! Uses the macOS libproc C library directly via FFI.
//! The darwin-libproc crate pins memchr ~2.3 which conflicts with the rest of our
//! dependency tree, so we call the underlying C functions directly.
//!
//! Key libproc functions used:
//! - proc_pidinfo(pid, PROC_PIDVNODEPATHINFO, ...) -> CWD path
//! - proc_pidpath(pid, buf, size)                  -> full binary path (for shell name)
//! - proc_listchildpids(ppid, buf, size)            -> child PIDs for process tree walk

/// Known shell binary names. Used to distinguish shells from other processes.
/// Includes both Unix names (for macOS) and Windows names (stripped of .exe by get_process_name).
const KNOWN_SHELLS: &[&str] = &[
    "bash", "zsh", "fish", "sh", "dash", "tcsh", "csh", "ksh", "nu", "elvish", "ion", "xonsh",
    // Windows shell names (after stripping .exe)
    "powershell", "pwsh", "cmd",
];

/// Process names indicating a terminal multiplexer that requires deeper process tree walking.
const MULTIPLEXERS: &[&str] = &["tmux", "screen"];

/// Wrapper processes that sit between the terminal app and the actual shell.
/// Terminal.app spawns: Terminal → login → zsh. We need to walk through these.
const SHELL_WRAPPERS: &[&str] = &["login", "sshd", "su", "sudo"];

/// PATH_MAX on macOS is 1024 bytes.
const PATH_MAX: usize = 1024;

/// PROC_PIDVNODEPATHINFO flavor (9) -- returns proc_vnodepathinfo struct with CWD path.
const PROC_PIDVNODEPATHINFO: i32 = 9;

/// proc_vnodepathinfo struct layout from <sys/proc_info.h>.
///
/// We only need pvi_cdir.pvip.vip_path (the CWD), which is at the start of the struct.
/// The struct is 2 * (vnode_info_path) = 2 * (vnode_info + PATH_MAX).
///
/// vnode_info is 64 bytes (8 fields of various sizes per proc_info.h).
/// vnode_info_path is vnode_info (64) + char[PATH_MAX] (1024) = 1088 bytes.
/// proc_vnodepathinfo is 2 * vnode_info_path = 2176 bytes.
///
/// We use a raw byte array sized to hold the full struct, then extract the path
/// from the known offset (vnode_info = 64 bytes, path starts at byte 64).
const VNODE_INFO_SIZE: usize = 64;
const VNODE_INFO_PATH_SIZE: usize = VNODE_INFO_SIZE + PATH_MAX;
const PROC_VNODEPATHINFO_SIZE: usize = 2 * VNODE_INFO_PATH_SIZE;
/// Offset to vip_path within vnode_info_path (i.e., after the vnode_info header).
const VIP_PATH_OFFSET: usize = VNODE_INFO_SIZE;

#[cfg(target_os = "macos")]
mod ffi {
    extern "C" {
        /// Get process information by flavor. Returns bytes written on success, -1 on error.
        pub fn proc_pidinfo(
            pid: i32,
            flavor: i32,
            arg: u64,
            buffer: *mut u8,
            buffersize: i32,
        ) -> i32;

        /// Get full path to the executable for a PID. Returns bytes written on success, <= 0 on error.
        pub fn proc_pidpath(pid: i32, buffer: *mut u8, buffersize: u32) -> i32;

        /// List child PIDs of a parent PID.
        /// Pass NULL/0 first to get count, then allocate and call again.
        /// Returns number of bytes written (n_pids * 4) on success, -1 on error.
        pub fn proc_listchildpids(ppid: i32, buffer: *mut i32, buffersize: i32) -> i32;
    }
}

/// Result of process inspection for a terminal's foreground shell.
pub struct ProcessInfo {
    /// Current working directory of the shell process.
    pub cwd: Option<String>,
    /// Shell binary name, e.g. "zsh", "bash", "fish".
    pub shell_type: Option<String>,
    /// Name of the foreground process running inside the shell (if any).
    /// None if the shell is idle.
    pub running_process: Option<String>,
    /// True when wsl.exe is found in the process ancestry (Windows only).
    pub is_wsl: bool,
}

/// Get CWD, shell type, and running process for the terminal identified by `terminal_pid`.
///
/// Walks the process tree from the terminal app PID to find the foreground shell,
/// handles tmux/screen multiplexers by walking deeper (max 3 levels).
pub fn get_foreground_info(terminal_pid: i32) -> ProcessInfo {
    eprintln!("[process] get_foreground_info for terminal_pid={}", terminal_pid);
    let children = get_child_pids(terminal_pid);
    eprintln!("[process] direct children of {}: {:?}", terminal_pid, children);
    for &child in &children {
        let name = get_process_name(child);
        eprintln!("[process]   child {} -> name: {:?}", child, name);
    }

    let shell_pid = match find_shell_pid(terminal_pid, None) {
        Some(pid) => {
            eprintln!("[process] found shell pid: {} (name: {:?})", pid, get_process_name(pid));
            pid
        }
        None => {
            eprintln!("[process] no shell pid found for terminal_pid={}", terminal_pid);
            return ProcessInfo {
                cwd: None,
                shell_type: None,
                running_process: None,
                is_wsl: false,
            }
        }
    };

    // Get CWD from the shell process via proc_pidinfo PROC_PIDVNODEPATHINFO
    let cwd = get_process_cwd(shell_pid);
    eprintln!("[process] cwd for shell_pid {}: {:?}", shell_pid, cwd);

    // Get shell type from actual binary name (not $SHELL -- avoids Pitfall 7)
    let shell_name = get_process_name(shell_pid);
    let shell_type = shell_name.clone();
    eprintln!("[process] shell_type for {}: {:?}", shell_pid, shell_type);

    // Check if a foreground process is running inside the shell (e.g., node, python)
    let running_process = find_running_process(shell_pid, &shell_name);

    // Detect WSL session on Windows
    #[cfg(target_os = "windows")]
    let is_wsl = {
        // Check 1: wsl.exe in the process ancestry (existing)
        let mut wsl = detect_wsl_in_ancestry(shell_pid);

        // Check 2: wsl.exe as a CHILD of the shell process (VS Code WSL terminal)
        if !wsl {
            let children = get_child_pids(shell_pid);
            eprintln!("[process] shell pid {} children: {:?}", shell_pid,
                children.iter().map(|&c| (c, get_process_name(c))).collect::<Vec<_>>());
            for &child in &children {
                if let Some(name) = get_process_name(child) {
                    if name.eq_ignore_ascii_case("wsl") {
                        eprintln!("[process] WSL detected: wsl.exe (pid {}) is child of shell pid {}", child, shell_pid);
                        wsl = true;
                        break;
                    }
                }
            }
        }

        // Check 3: scan ALL wsl.exe under app for diagnostic visibility
        if !wsl {
            eprintln!("[process] WSL diagnostic: scanning all wsl.exe under app tree...");
            scan_wsl_processes_diagnostic(shell_pid);
        }

        wsl
    };
    #[cfg(not(target_os = "windows"))]
    let is_wsl = false;

    ProcessInfo {
        cwd,
        shell_type,
        running_process,
        is_wsl,
    }
}

/// Get the current working directory of a process.
///
/// Tries proc_pidinfo with PROC_PIDVNODEPATHINFO first (fast, no subprocess).
/// Falls back to `lsof -a -p <pid> -d cwd -Fn` if proc_pidinfo fails
/// (which happens for processes spawned by root-owned parents like `login`).
#[cfg(target_os = "macos")]
fn get_process_cwd(pid: i32) -> Option<String> {
    // Fast path: proc_pidinfo
    if let Some(cwd) = get_process_cwd_libproc(pid) {
        return Some(cwd);
    }

    // Fallback: lsof
    eprintln!("[process] proc_pidinfo CWD failed for pid {}, trying lsof fallback", pid);
    get_process_cwd_lsof(pid)
}

/// Fast path: use proc_pidinfo with PROC_PIDVNODEPATHINFO.
#[cfg(target_os = "macos")]
fn get_process_cwd_libproc(pid: i32) -> Option<String> {
    use std::ffi::CStr;

    let mut buf = [0u8; PROC_VNODEPATHINFO_SIZE];
    let ret = unsafe {
        ffi::proc_pidinfo(
            pid,
            PROC_PIDVNODEPATHINFO,
            0,
            buf.as_mut_ptr(),
            PROC_VNODEPATHINFO_SIZE as i32,
        )
    };

    if ret <= 0 {
        eprintln!("[process] proc_pidinfo({}, VNODEPATHINFO) returned {}", pid, ret);
        return None;
    }

    let path_slice = &buf[VIP_PATH_OFFSET..VIP_PATH_OFFSET + PATH_MAX];
    let c_str = unsafe { CStr::from_ptr(path_slice.as_ptr() as *const i8) };
    let path = c_str.to_string_lossy();
    if path.is_empty() {
        None
    } else {
        Some(path.into_owned())
    }
}

/// Fallback: use `lsof` to get CWD of a process.
#[cfg(target_os = "macos")]
fn get_process_cwd_lsof(pid: i32) -> Option<String> {
    let output = std::process::Command::new("lsof")
        .args(["-a", "-p", &pid.to_string(), "-d", "cwd", "-Fn"])
        .output()
        .ok()?;

    if !output.status.success() {
        eprintln!("[process] lsof CWD for pid {} failed", pid);
        return None;
    }

    // lsof -Fn output: lines starting with 'n' contain the path
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix('n') {
            if !path.is_empty() {
                eprintln!("[process] lsof CWD for pid {}: {}", pid, path);
                return Some(path.to_string());
            }
        }
    }

    None
}

/// Windows: get CWD of a process via NtQueryInformationProcess → PEB → ProcessParameters.
///
/// This reads the remote process's PEB (Process Environment Block) to extract the
/// CurrentDirectory path from RTL_USER_PROCESS_PARAMETERS. Requires PROCESS_QUERY_INFORMATION
/// and PROCESS_VM_READ access. Returns None for elevated processes (Access Denied) or on error.
#[cfg(target_os = "windows")]
fn get_process_cwd(pid: i32) -> Option<String> {
    get_process_cwd_windows(pid as u32)
}

#[cfg(target_os = "windows")]
fn get_process_cwd_windows(pid: u32) -> Option<String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if handle.is_null() {
            eprintln!("[process] get_process_cwd_windows: OpenProcess failed for pid {} (access denied?)", pid);
            return None;
        }

        let result = read_cwd_from_peb(handle);
        CloseHandle(handle);
        result
    }
}

/// Read CWD from a process handle by walking PEB → ProcessParameters → CurrentDirectory.
#[cfg(target_os = "windows")]
unsafe fn read_cwd_from_peb(handle: windows_sys::Win32::Foundation::HANDLE) -> Option<String> {
    use std::mem::size_of;
    use windows_sys::Wdk::System::Threading::NtQueryInformationProcess;

    // PROCESS_BASIC_INFORMATION layout for 64-bit
    #[repr(C)]
    struct ProcessBasicInformation {
        _reserved1: usize,
        peb_base_address: usize,
        _reserved2: [usize; 2],
        unique_process_id: usize,
        _reserved3: usize,
    }

    let mut pbi = std::mem::zeroed::<ProcessBasicInformation>();
    let mut return_length: u32 = 0;
    let status = NtQueryInformationProcess(
        handle,
        0, // ProcessBasicInformation
        &mut pbi as *mut _ as *mut _,
        size_of::<ProcessBasicInformation>() as u32,
        &mut return_length,
    );

    if status != 0 {
        eprintln!("[process] NtQueryInformationProcess failed: 0x{:08x}", status);
        return None;
    }

    if pbi.peb_base_address == 0 {
        return None;
    }

    // Read ProcessParameters pointer from PEB (offset 0x20 on 64-bit)
    let params_ptr_addr = pbi.peb_base_address + 0x20;
    let mut params_ptr: usize = 0;
    let ok = read_process_memory(handle, params_ptr_addr, &mut params_ptr as *mut _ as *mut u8, size_of::<usize>());
    if !ok || params_ptr == 0 {
        return None;
    }

    // RTL_USER_PROCESS_PARAMETERS: CurrentDirectory.DosPath is a UNICODE_STRING at offset 0x38
    // UNICODE_STRING layout: u16 Length, u16 MaxLength, padding, u64 Buffer pointer
    let cwd_unicode_string_addr = params_ptr + 0x38;
    let mut length: u16 = 0;
    let ok = read_process_memory(handle, cwd_unicode_string_addr, &mut length as *mut _ as *mut u8, size_of::<u16>());
    if !ok || length == 0 {
        return None;
    }

    let mut buffer_ptr: usize = 0;
    let buffer_ptr_addr = cwd_unicode_string_addr + 8; // skip Length (2), MaxLength (2), padding (4)
    let ok = read_process_memory(handle, buffer_ptr_addr, &mut buffer_ptr as *mut _ as *mut u8, size_of::<usize>());
    if !ok || buffer_ptr == 0 {
        return None;
    }

    // Read the actual CWD string (UTF-16)
    let char_count = (length as usize) / 2;
    let mut buf = vec![0u16; char_count];
    let ok = read_process_memory(handle, buffer_ptr, buf.as_mut_ptr() as *mut u8, length as usize);
    if !ok {
        return None;
    }

    let cwd = String::from_utf16_lossy(&buf);
    // Strip trailing backslash (e.g., "C:\Users\foo\" → "C:\Users\foo")
    let cwd = cwd.trim_end_matches('\\').to_string();
    if cwd.is_empty() {
        None
    } else {
        eprintln!("[process] CWD for pid via PEB: {}", &cwd);
        Some(cwd)
    }
}

/// Helper: ReadProcessMemory wrapper.
#[cfg(target_os = "windows")]
unsafe fn read_process_memory(handle: windows_sys::Win32::Foundation::HANDLE, address: usize, buffer: *mut u8, size: usize) -> bool {
    use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
    let mut bytes_read: usize = 0;
    let ok = ReadProcessMemory(
        handle,
        address as *const _,
        buffer as *mut _,
        size,
        &mut bytes_read,
    );
    ok != 0 && bytes_read == size
}

/// Non-macOS, non-Windows stub.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn get_process_cwd(_pid: i32) -> Option<String> {
    None
}

/// Get the binary name of a process by extracting the filename from its full path.
///
/// Uses proc_pidpath to get the full executable path (e.g., "/bin/zsh"),
/// then extracts just the filename (e.g., "zsh").
#[cfg(target_os = "macos")]
fn get_process_name(pid: i32) -> Option<String> {
    let mut buf = [0u8; PATH_MAX];
    // SAFETY: buf is stack-allocated at PATH_MAX. proc_pidpath is stable macOS public API.
    let ret = unsafe { ffi::proc_pidpath(pid, buf.as_mut_ptr(), PATH_MAX as u32) };

    if ret <= 0 {
        return None;
    }

    let path_str = std::str::from_utf8(&buf[..ret as usize]).ok()?;
    let path = std::path::Path::new(path_str.trim_end_matches('\0'));
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

/// Windows: get process name via QueryFullProcessImageNameW.
#[cfg(target_os = "windows")]
fn get_process_name(pid: i32) -> Option<String> {
    super::detect_windows::get_exe_name_for_pid(pid as u32)
        .map(|exe| exe.trim_end_matches(".exe").trim_end_matches(".EXE").to_string())
}

/// Non-macOS, non-Windows stub.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn get_process_name(_pid: i32) -> Option<String> {
    None
}

/// Get the child PIDs of a process.
///
/// Tries proc_listchildpids first (fast), falls back to sysctl KERN_PROC scan
/// if that returns nothing (proc_listchildpids can fail for root-owned processes
/// like `login` when called from a user-owned process).
#[cfg(target_os = "macos")]
fn get_child_pids(pid: i32) -> Vec<i32> {
    // Try proc_listchildpids first (fast path)
    let pids = get_child_pids_libproc(pid);
    if !pids.is_empty() {
        return pids;
    }

    // Fallback: scan all processes via sysctl and find those with ppid == pid
    eprintln!("[process] proc_listchildpids({}) returned empty, trying sysctl fallback", pid);
    get_child_pids_sysctl(pid)
}

/// Fast path: use proc_listchildpids.
#[cfg(target_os = "macos")]
fn get_child_pids_libproc(pid: i32) -> Vec<i32> {
    let count = unsafe { ffi::proc_listchildpids(pid, std::ptr::null_mut(), 0) };
    if count <= 0 {
        return Vec::new();
    }

    let n_pids = (count as usize) / std::mem::size_of::<i32>();
    if n_pids == 0 {
        return Vec::new();
    }

    let mut pids: Vec<i32> = vec![0i32; n_pids];
    let ret = unsafe {
        ffi::proc_listchildpids(pid, pids.as_mut_ptr(), count)
    };

    if ret <= 0 {
        return Vec::new();
    }

    let actual_n = (ret as usize) / std::mem::size_of::<i32>();
    pids.truncate(actual_n);
    pids.retain(|&p| p > 0);
    pids
}

/// Fallback: use `pgrep -P <pid>` to find child processes.
/// This works reliably for all process types including root-owned processes
/// like `login`, where proc_listchildpids may fail due to permissions.
#[cfg(target_os = "macos")]
fn get_child_pids_sysctl(ppid: i32) -> Vec<i32> {
    let output = std::process::Command::new("pgrep")
        .arg("-P")
        .arg(ppid.to_string())
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let pids: Vec<i32> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter_map(|line| line.trim().parse::<i32>().ok())
                .filter(|&p| p > 0)
                .collect();
            eprintln!("[process] pgrep -P {} found: {:?}", ppid, pids);
            pids
        }
        _ => {
            eprintln!("[process] pgrep -P {} failed or no children", ppid);
            Vec::new()
        }
    }
}

/// Windows: get child PIDs via CreateToolhelp32Snapshot.
#[cfg(target_os = "windows")]
fn get_child_pids(pid: i32) -> Vec<i32> {
    get_child_pids_windows(pid as u32)
        .into_iter()
        .map(|p| p as i32)
        .collect()
}

#[cfg(target_os = "windows")]
fn get_child_pids_windows(ppid: u32) -> Vec<u32> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Vec::new();
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut children = Vec::new();
        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                if entry.th32ParentProcessID == ppid && entry.th32ProcessID != ppid {
                    children.push(entry.th32ProcessID);
                }
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
        children
    }
}

/// Non-macOS, non-Windows stub.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn get_child_pids(_pid: i32) -> Vec<i32> {
    Vec::new()
}

/// Find the foreground shell PID for a given app.
///
/// Strategy:
/// 1. Fast path: walk direct children (handles Terminal.app -> login -> zsh).
/// 2. Broad path: use `pgrep` to find all shell processes system-wide,
///    then check which ones are descendants of this app PID by walking
///    up the parent chain. This handles deep Electron trees in VS Code/Cursor
///    (e.g., VSCode -> Helper -> node -> pty-helper -> zsh).
///
/// `focused_cwd`: AX-derived CWD from the focused terminal tab. Used to
/// disambiguate between multiple candidate shells in Electron IDEs with
/// multiple terminal tabs. Pass `None` for non-IDE callers.
pub(crate) fn find_shell_pid(terminal_pid: i32, focused_cwd: Option<&str>) -> Option<i32> {
    // Try the fast recursive walk first (works for simple terminal apps).
    // The fast path does NOT need focused_cwd because it only returns a result
    // when there is exactly one shell in a shallow tree (Terminal.app, iTerm2
    // single tab), where ambiguity does not arise.
    if let Some(pid) = find_shell_recursive(terminal_pid, 3) {
        return Some(pid);
    }

    // Broad search: find shells that are descendants of this app
    eprintln!("[process] recursive walk failed for {}, trying ancestry search", terminal_pid);
    find_shell_by_ancestry(terminal_pid, focused_cwd)
}

/// Find shell processes that are descendants of the given app PID.
///
/// Uses `pgrep` to find all running shell processes, then for each one
/// walks up the parent chain to check if the app PID is an ancestor.
/// Collects ALL matching shells and picks the most recently spawned one
/// (highest PID), which is most likely the active/focused terminal tab.
#[cfg(target_os = "macos")]
fn find_shell_by_ancestry(app_pid: i32, focused_cwd: Option<&str>) -> Option<i32> {
    // Build a pgrep pattern matching all known shells
    let shell_pattern = KNOWN_SHELLS.join("|");
    let output = std::process::Command::new("pgrep")
        .arg("-x") // exact match on process name
        .arg(&shell_pattern)
        .output();

    let shell_pids: Vec<i32> = match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter_map(|line| line.trim().parse::<i32>().ok())
                .filter(|&p| p > 0)
                .collect()
        }
        _ => {
            eprintln!("[process] pgrep for shells failed");
            return None;
        }
    };

    eprintln!("[process] found {} shell processes system-wide, checking ancestry to {}", shell_pids.len(), app_pid);

    // Collect ALL shells that are descendants of the app
    let mut descendant_shells: Vec<(i32, Option<String>)> = Vec::new();
    for &shell_pid in &shell_pids {
        if is_descendant_of(shell_pid, app_pid) {
            let name = get_process_name(shell_pid);
            eprintln!("[process] shell pid {} ({:?}) is a descendant of app {}", shell_pid, name, app_pid);
            descendant_shells.push((shell_pid, name));
        }
    }

    if descendant_shells.is_empty() {
        eprintln!("[process] no shell found as descendant of {}", app_pid);
        return None;
    }

    eprintln!("[process] found {} descendant shells: {:?}", descendant_shells.len(),
        descendant_shells.iter().map(|(pid, n)| (*pid, n.clone())).collect::<Vec<_>>());

    // Step 1: Filter out sub-shells (shells spawned from within another shell).
    // E.g., Claude Code's bash spawned from a user's zsh terminal.
    let shell_pid_set: Vec<i32> = descendant_shells.iter().map(|(pid, _)| *pid).collect();
    let parent_map = build_parent_map();

    let top_level: Vec<&(i32, Option<String>)> = descendant_shells.iter()
        .filter(|(pid, name)| {
            let is_sub = is_sub_shell_of_any(*pid, &shell_pid_set, &parent_map);
            if is_sub {
                eprintln!("[process] filtering out sub-shell pid {} ({:?}) -- descendant of another shell", pid, name);
            }
            !is_sub
        })
        .collect();

    let remaining = if top_level.is_empty() {
        descendant_shells.iter().collect::<Vec<_>>()
    } else {
        top_level
    };

    // Step 2: If mixed shell types remain (e.g., extension-spawned bash + user zsh),
    // prefer shells matching $SHELL since we can't determine the active IDE tab.
    let has_mixed_types = {
        let first = remaining.first().and_then(|(_, n)| n.as_deref());
        remaining.iter().any(|(_, n)| n.as_deref() != first)
    };

    let candidates = if has_mixed_types {
        let preferred = std::env::var("SHELL").ok()
            .and_then(|s| std::path::Path::new(&s).file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_string()));

        if let Some(ref pref) = preferred {
            let matching: Vec<_> = remaining.iter()
                .filter(|(_, name)| name.as_deref() == Some(pref.as_str()))
                .cloned()
                .collect();
            if !matching.is_empty() {
                eprintln!("[process] mixed shell types in IDE, preferring {} matching $SHELL ({} found)", pref, matching.len());
                matching
            } else {
                remaining
            }
        } else {
            remaining
        }
    } else {
        remaining
    };

    // Step 2.5: CWD-based focused tab matching.
    // If we have a focused CWD from AX (IDE terminal tab), compare it against
    // the CWD of each candidate shell to find the one matching the focused tab.
    if let Some(target_cwd) = focused_cwd {
        if candidates.len() > 1 {
            let mut cwd_matches: Vec<&(i32, Option<String>)> = Vec::new();
            for candidate in &candidates {
                if let Some(shell_cwd) = get_process_cwd(candidate.0) {
                    eprintln!("[process] shell pid {} CWD: {}", candidate.0, &shell_cwd);
                    if shell_cwd == target_cwd {
                        cwd_matches.push(candidate);
                    }
                }
            }
            if cwd_matches.len() == 1 {
                let matched = cwd_matches[0];
                eprintln!("[process] CWD match: shell pid {} matches focused tab CWD {}", matched.0, target_cwd);
                return Some(matched.0);
            } else if cwd_matches.len() > 1 {
                eprintln!("[process] {} shells match focused CWD {} -- falling through to highest PID among matches", cwd_matches.len(), target_cwd);
                // Multiple shells in the same CWD -- pick highest PID among matches (best we can do)
                return cwd_matches.iter().max_by_key(|(pid, _)| *pid).map(|(pid, _)| *pid);
            }
            // No CWD match found -- fall through to Step 3 (highest PID)
            eprintln!("[process] no CWD match for focused tab CWD {} among {} candidates", target_cwd, candidates.len());
        }
    }

    // Step 3: Among candidates, pick the most recently spawned (highest PID).
    let best = candidates.iter().max_by_key(|(pid, _)| *pid);
    best.map(|(pid, _)| *pid)
}

/// Check if `pid` is a descendant of `ancestor_pid` by walking up the parent chain.
/// Walks at most 15 levels to avoid infinite loops from PID 1.
#[cfg(target_os = "macos")]
fn is_descendant_of(pid: i32, ancestor_pid: i32) -> bool {
    let mut current = pid;
    for _ in 0..15 {
        let parent = get_parent_pid(current);
        match parent {
            Some(ppid) if ppid == ancestor_pid => return true,
            Some(ppid) if ppid <= 1 => return false, // reached init/launchd
            Some(ppid) => current = ppid,
            None => return false,
        }
    }
    false
}

/// Get the parent PID of a process using `ps`.
#[cfg(target_os = "macos")]
fn get_parent_pid(pid: i32) -> Option<i32> {
    let output = std::process::Command::new("ps")
        .args(["-o", "ppid=", "-p", &pid.to_string()])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<i32>()
        .ok()
}

/// Build a PID -> parent PID map from a single `ps` call.
/// Much faster than calling `get_parent_pid` per-process for sub-shell filtering.
#[cfg(target_os = "macos")]
fn build_parent_map() -> std::collections::HashMap<i32, i32> {
    let mut map = std::collections::HashMap::new();
    let output = std::process::Command::new("ps")
        .args(["-eo", "pid=,ppid="])
        .output();

    if let Ok(out) = output {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                if let (Ok(pid), Ok(ppid)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                    map.insert(pid, ppid);
                }
            }
        }
    }
    map
}

/// Check if `pid` is a descendant of any PID in `ancestor_pids` using the pre-built parent map.
/// This identifies sub-shells (e.g., Claude Code's bash spawned from a user's zsh).
#[cfg(target_os = "macos")]
fn is_sub_shell_of_any(pid: i32, ancestor_pids: &[i32], parent_map: &std::collections::HashMap<i32, i32>) -> bool {
    let mut current = pid;
    for _ in 0..20 {
        match parent_map.get(&current) {
            Some(&ppid) if ppid <= 1 => return false,
            Some(&ppid) => {
                if ancestor_pids.contains(&ppid) {
                    return true;
                }
                current = ppid;
            }
            None => return false,
        }
    }
    false
}

/// Windows: find shell processes that are descendants of the given app PID.
///
/// Uses CreateToolhelp32Snapshot to build a full process tree and walk ancestors.
#[cfg(target_os = "windows")]
fn find_shell_by_ancestry(app_pid: i32, _focused_cwd: Option<&str>) -> Option<i32> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return None;
        }

        // Build parent map and collect shell candidates
        let mut parent_map: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
        let mut shell_candidates: Vec<(u32, String)> = Vec::new();
        let mut openconsole_pids: Vec<u32> = Vec::new();

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                parent_map.insert(entry.th32ProcessID, entry.th32ParentProcessID);

                // Get exe name from the entry
                let name_len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..name_len]);

                if name.eq_ignore_ascii_case("OpenConsole.exe") {
                    openconsole_pids.push(entry.th32ProcessID);
                }
                if super::detect_windows::is_known_shell_exe(&name) {
                    shell_candidates.push((entry.th32ProcessID, name));
                }

                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);

        let app_pid_u32 = app_pid as u32;

        // Check which shells are descendants of app_pid
        let mut descendant_shells: Vec<(u32, String)> = Vec::new();
        for (pid, name) in &shell_candidates {
            let mut current = *pid;
            let mut is_desc = false;
            for _ in 0..20 {
                match parent_map.get(&current) {
                    Some(&ppid) if ppid == app_pid_u32 => {
                        is_desc = true;
                        break;
                    }
                    Some(&ppid) if ppid <= 1 || ppid == current => break,
                    Some(&ppid) => current = ppid,
                    None => break,
                }
            }
            if is_desc {
                eprintln!("[process] Windows shell pid {} ({}) is a descendant of {}", pid, name, app_pid);
                descendant_shells.push((*pid, name.clone()));
            }
        }

        if descendant_shells.is_empty() {
            eprintln!("[process] no shell found as descendant of {} on Windows", app_pid);

            // Fallback for Windows Terminal: shells are children of OpenConsole.exe
            // which is NOT a descendant of WindowsTerminal.exe in the process tree
            // (ConPTY architecture). Search for shells parented by OpenConsole.exe.
            eprintln!("[process] ConPTY fallback: {} OpenConsole PIDs found, {} shell candidates", openconsole_pids.len(), shell_candidates.len());
            for (pid, name) in &shell_candidates {
                let ppid = parent_map.get(pid).copied();
                eprintln!("[process]   shell candidate: pid={} name={} ppid={:?}", pid, name, ppid);
            }
            if !openconsole_pids.is_empty() {
                let mut conpty_shells: Vec<(u32, String)> = Vec::new();
                for (pid, name) in &shell_candidates {
                    if let Some(&ppid) = parent_map.get(pid) {
                        if openconsole_pids.contains(&ppid) {
                            eprintln!("[process] ConPTY shell: pid {} ({}) is child of OpenConsole.exe {}", pid, name, ppid);
                            conpty_shells.push((*pid, name.clone()));
                        }
                    }
                }
                if !conpty_shells.is_empty() {
                    return conpty_shells.iter()
                        .max_by_key(|(pid, _)| *pid)
                        .map(|(pid, _)| *pid as i32);
                }
            }

            return None;
        }

        // For IDE terminals (VS Code, Cursor), deprioritize cmd.exe.
        // VS Code spawns many internal cmd.exe processes for git, tasks, and extensions.
        // User terminal tabs are typically powershell.exe, pwsh.exe, bash.exe, etc.
        let is_ide = super::detect_windows::is_ide_with_terminal_exe(
            &super::detect_windows::get_exe_name_for_pid(app_pid_u32).unwrap_or_default()
        );
        if is_ide && descendant_shells.len() > 1 {
            let interactive: Vec<_> = descendant_shells.iter()
                .filter(|(_, name)| {
                    let lower = name.to_lowercase();
                    // Keep only real interactive shells, exclude cmd.exe (VS Code internal)
                    lower != "cmd.exe"
                })
                .collect();
            if !interactive.is_empty() {
                eprintln!("[process] IDE mode: preferring interactive shells ({} of {} descendants)",
                    interactive.len(), descendant_shells.len());
                for (pid, name) in &interactive {
                    eprintln!("[process]   interactive shell: pid={} name={}", pid, name);
                }
                return interactive.iter()
                    .max_by_key(|(pid, _)| *pid)
                    .map(|(pid, _)| *pid as i32);
            }
        }

        // Pick the most recently spawned (highest PID) shell
        descendant_shells.iter()
            .max_by_key(|(pid, _)| *pid)
            .map(|(pid, _)| *pid as i32)
    }
}

/// Non-macOS, non-Windows stub.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn find_shell_by_ancestry(_app_pid: i32, _focused_cwd: Option<&str>) -> Option<i32> {
    None
}

/// Recursively walk child processes to find a shell, walking through
/// multiplexers and wrapper processes (login, sshd, su, sudo).
fn find_shell_recursive(pid: i32, max_depth: u32) -> Option<i32> {
    if max_depth == 0 {
        return None;
    }

    let children = get_child_pids(pid);
    if children.is_empty() {
        return None;
    }

    // First pass: look for a known shell among direct children
    for &child in &children {
        if let Some(ref n) = get_process_name(child) {
            if KNOWN_SHELLS.contains(&n.as_str()) {
                return Some(child);
            }
        }
    }

    // Second pass: walk through multiplexers and wrapper processes
    for &child in &children {
        if let Some(ref n) = get_process_name(child) {
            if MULTIPLEXERS.iter().any(|m| n.contains(m))
                || SHELL_WRAPPERS.contains(&n.as_str())
            {
                if let Some(shell) = find_shell_recursive(child, max_depth - 1) {
                    return Some(shell);
                }
            }
        }
    }

    // Fallback: try walking into the first child (might be an unlisted wrapper)
    find_shell_recursive(children[0], max_depth - 1)
}

/// Find a non-shell foreground process running inside the shell.
///
/// Looks at the shell's child processes. If one is not the shell itself,
/// it is the running process (e.g., "node", "python", "vim").
fn find_running_process(shell_pid: i32, shell_name: &Option<String>) -> Option<String> {
    let children = get_child_pids(shell_pid);
    for &child in &children {
        let name = get_process_name(child);
        if let Some(ref n) = name {
            // Skip if same name as the shell (sub-shell scenario)
            if shell_name.as_ref() == Some(n) {
                continue;
            }
            return Some(n.clone());
        }
    }
    None
}

/// Diagnostic: find ALL wsl.exe processes and log their parent chain.
/// Helps debug VS Code WSL terminal detection by showing where wsl.exe lives.
#[cfg(target_os = "windows")]
fn scan_wsl_processes_diagnostic(_shell_pid: i32) {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return;
        }

        let mut parent_map: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
        let mut exe_map: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
        let mut wsl_pids: Vec<u32> = Vec::new();

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                parent_map.insert(entry.th32ProcessID, entry.th32ParentProcessID);
                let name_len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..name_len]);
                if name.eq_ignore_ascii_case("wsl.exe") {
                    wsl_pids.push(entry.th32ProcessID);
                }
                exe_map.insert(entry.th32ProcessID, name);
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);

        eprintln!("[process] WSL diagnostic: found {} wsl.exe processes total", wsl_pids.len());
        for &wsl_pid in &wsl_pids {
            // Build parent chain for this wsl.exe
            let mut chain = Vec::new();
            let mut current = wsl_pid;
            for _ in 0..10 {
                let name = exe_map.get(&current).cloned().unwrap_or_else(|| "?".to_string());
                chain.push(format!("{}({})", name, current));
                match parent_map.get(&current) {
                    Some(&ppid) if ppid > 1 && ppid != current => current = ppid,
                    _ => break,
                }
            }
            chain.reverse();
            eprintln!("[process] WSL diagnostic: wsl.exe {} chain: {}", wsl_pid, chain.join(" → "));
        }
    }
}

/// Detect if wsl.exe is in the process ancestry of the given PID.
///
/// Takes a separate process snapshot and walks the parent chain looking for wsl.exe.
/// The snapshot is cheap (~1ms) and avoids changing find_shell_by_ancestry's signature.
#[cfg(target_os = "windows")]
fn detect_wsl_in_ancestry(pid: i32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return false;
        }

        let mut parent_map: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
        let mut exe_map: std::collections::HashMap<u32, String> = std::collections::HashMap::new();

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                parent_map.insert(entry.th32ProcessID, entry.th32ParentProcessID);

                let name_len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..name_len]);
                exe_map.insert(entry.th32ProcessID, name);

                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);

        // Walk ancestry from the given PID looking for wsl.exe
        let mut current = pid as u32;
        for _ in 0..20 {
            if let Some(exe) = exe_map.get(&current) {
                if exe.eq_ignore_ascii_case("wsl.exe") {
                    eprintln!("[process] WSL detected: wsl.exe found at pid {} in ancestry of {}", current, pid);
                    return true;
                }
            }
            match parent_map.get(&current) {
                Some(&ppid) if ppid <= 1 || ppid == current => break,
                Some(&ppid) => current = ppid,
                None => break,
            }
        }
        false
    }
}

/// Get the Linux CWD from WSL via subprocess.
///
/// Spawns `wsl.exe -e sh -c "pwd"` to read the default WSL distro's CWD.
/// Note: This returns the HOME directory of the WSL default user, not the
/// active shell's CWD. UIA text inference provides better CWD when available.
#[cfg(target_os = "windows")]
pub fn get_wsl_cwd() -> Option<String> {
    let output = std::process::Command::new("wsl.exe")
        .args(["-e", "sh", "-c", "pwd"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if output.status.success() {
        let cwd = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !cwd.is_empty() && cwd.starts_with('/') {
            eprintln!("[process] WSL CWD via subprocess: {}", &cwd);
            Some(cwd)
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_wsl_cwd() -> Option<String> {
    None
}

/// Get the Linux shell type from WSL via subprocess.
///
/// Reads $SHELL from the default WSL distro. This is the configured default shell,
/// which may differ from the actually running shell.
#[cfg(target_os = "windows")]
pub fn get_wsl_shell() -> Option<String> {
    let output = std::process::Command::new("wsl.exe")
        .args(["-e", "sh", "-c", "basename \"$SHELL\""])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if output.status.success() {
        let shell = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !shell.is_empty() {
            eprintln!("[process] WSL shell via subprocess: {}", &shell);
            Some(shell)
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_wsl_shell() -> Option<String> {
    None
}
