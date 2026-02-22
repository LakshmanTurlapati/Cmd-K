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
const KNOWN_SHELLS: &[&str] = &[
    "bash", "zsh", "fish", "sh", "dash", "tcsh", "csh", "ksh", "nu", "elvish", "ion", "xonsh",
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

    let shell_pid = match find_shell_pid(terminal_pid) {
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

    ProcessInfo {
        cwd,
        shell_type,
        running_process,
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

/// Non-macOS stub.
#[cfg(not(target_os = "macos"))]
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

/// Non-macOS stub.
#[cfg(not(target_os = "macos"))]
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

/// Non-macOS stub.
#[cfg(not(target_os = "macos"))]
fn get_child_pids(_pid: i32) -> Vec<i32> {
    Vec::new()
}

/// Find the foreground shell PID for a given app.
///
/// Strategy:
/// 1. Fast path: walk direct children (handles Terminal.app → login → zsh).
/// 2. Broad path: use `pgrep` to find all shell processes system-wide,
///    then check which ones are descendants of this app PID by walking
///    up the parent chain. This handles deep Electron trees in VS Code/Cursor
///    (e.g., VSCode → Helper → node → pty-helper → zsh).
fn find_shell_pid(terminal_pid: i32) -> Option<i32> {
    // Try the fast recursive walk first (works for simple terminal apps)
    if let Some(pid) = find_shell_recursive(terminal_pid, 3) {
        return Some(pid);
    }

    // Broad search: find shells that are descendants of this app
    eprintln!("[process] recursive walk failed for {}, trying ancestry search", terminal_pid);
    find_shell_by_ancestry(terminal_pid)
}

/// Find shell processes that are descendants of the given app PID.
///
/// Uses `pgrep` to find all running shell processes, then for each one
/// walks up the parent chain to check if the app PID is an ancestor.
/// Collects ALL matching shells and picks the most recently spawned one
/// (highest PID), which is most likely the active/focused terminal tab.
#[cfg(target_os = "macos")]
fn find_shell_by_ancestry(app_pid: i32) -> Option<i32> {
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

    // When multiple shells exist (e.g., Cursor with user zsh terminals + Claude Code bash),
    // prefer shells matching the user's configured $SHELL to filter out tool-spawned shells.
    let preferred_shell = std::env::var("SHELL").ok()
        .and_then(|s| std::path::Path::new(&s).file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.to_string()));

    let candidates: Vec<&(i32, Option<String>)> = if let Some(ref pref) = preferred_shell {
        let matching: Vec<_> = descendant_shells.iter()
            .filter(|(_, name)| name.as_deref() == Some(pref.as_str()))
            .collect();
        if !matching.is_empty() {
            eprintln!("[process] preferring {} shells matching $SHELL ({} found)", pref, matching.len());
            matching
        } else {
            descendant_shells.iter().collect()
        }
    } else {
        descendant_shells.iter().collect()
    };

    // Among candidates, pick the most recently spawned (highest PID).
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

#[cfg(not(target_os = "macos"))]
fn find_shell_by_ancestry(_app_pid: i32) -> Option<i32> {
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
