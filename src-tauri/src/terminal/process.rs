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
    let shell_pid = match find_shell_pid(terminal_pid) {
        Some(pid) => pid,
        None => {
            return ProcessInfo {
                cwd: None,
                shell_type: None,
                running_process: None,
            }
        }
    };

    // Get CWD from the shell process via proc_pidinfo PROC_PIDVNODEPATHINFO
    let cwd = get_process_cwd(shell_pid);

    // Get shell type from actual binary name (not $SHELL -- avoids Pitfall 7)
    let shell_name = get_process_name(shell_pid);
    let shell_type = shell_name.clone();

    // Check if a foreground process is running inside the shell (e.g., node, python)
    let running_process = find_running_process(shell_pid, &shell_name);

    ProcessInfo {
        cwd,
        shell_type,
        running_process,
    }
}

/// Get the current working directory of a process via libproc proc_pidinfo.
///
/// Uses PROC_PIDVNODEPATHINFO flavor which returns a proc_vnodepathinfo struct
/// containing the current directory (pvi_cdir) and root directory (pvi_rdir).
/// The CWD path is extracted from the pvi_cdir.pvip.vip_path field.
#[cfg(target_os = "macos")]
fn get_process_cwd(pid: i32) -> Option<String> {
    use std::ffi::CStr;

    let mut buf = [0u8; PROC_VNODEPATHINFO_SIZE];
    // SAFETY: buf is stack-allocated with the exact size of proc_vnodepathinfo.
    // proc_pidinfo is a stable macOS public C API. We check the return value.
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
        return None;
    }

    // Extract the path from pvi_cdir.pvip.vip_path (at VIP_PATH_OFFSET within the struct)
    let path_slice = &buf[VIP_PATH_OFFSET..VIP_PATH_OFFSET + PATH_MAX];
    // SAFETY: We have a valid zero-terminated buffer from the kernel.
    let c_str = unsafe { CStr::from_ptr(path_slice.as_ptr() as *const i8) };
    let path = c_str.to_string_lossy();
    if path.is_empty() {
        None
    } else {
        Some(path.into_owned())
    }
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

/// Get the child PIDs of a process using proc_listchildpids.
///
/// Returns an empty Vec if there are no children or on error.
#[cfg(target_os = "macos")]
fn get_child_pids(pid: i32) -> Vec<i32> {
    // First call with NULL buffer to get the count
    // SAFETY: NULL buffer with 0 size is a documented pattern for proc_listchildpids
    let count = unsafe { ffi::proc_listchildpids(pid, std::ptr::null_mut(), 0) };
    if count <= 0 {
        return Vec::new();
    }

    // count is the number of bytes needed (n_pids * sizeof(pid_t) = n_pids * 4)
    let n_pids = (count as usize) / std::mem::size_of::<i32>();
    if n_pids == 0 {
        return Vec::new();
    }

    let mut pids: Vec<i32> = vec![0i32; n_pids];
    // SAFETY: pids Vec is properly allocated with the right size.
    let ret = unsafe {
        ffi::proc_listchildpids(pid, pids.as_mut_ptr(), count)
    };

    if ret <= 0 {
        return Vec::new();
    }

    // Actual number of PIDs written
    let actual_n = (ret as usize) / std::mem::size_of::<i32>();
    pids.truncate(actual_n);
    // Filter out zero PIDs (padding/error values)
    pids.retain(|&p| p > 0);
    pids
}

/// Non-macOS stub.
#[cfg(not(target_os = "macos"))]
fn get_child_pids(_pid: i32) -> Vec<i32> {
    Vec::new()
}

/// Walk child processes from a terminal app PID to find the foreground shell PID.
///
/// Handles multiplexers (tmux/screen) and wrapper processes (login, sshd, su, sudo):
/// - Terminal.app: Terminal → login → zsh
/// - tmux: Terminal → tmux-server → tmux-client → zsh
/// Walks up to 3 levels deep to find the actual shell.
fn find_shell_pid(terminal_pid: i32) -> Option<i32> {
    find_shell_recursive(terminal_pid, 3)
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
