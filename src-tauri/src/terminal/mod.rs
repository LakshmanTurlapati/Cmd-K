pub mod detect;
pub mod process;

use serde::Serialize;

/// Context information about the terminal that was frontmost before the overlay appeared.
///
/// All fields are Option<String> because detection can fail at any step:
/// - CWD: the process may not have a valid CWD accessible via libproc
/// - shell_type: derived from binary name, may be unknown
/// - visible_output: populated by Plan 02 (AX text reading); always None in this plan
/// - running_process: only present when something is running inside the shell
#[derive(Debug, Clone, Serialize)]
pub struct TerminalContext {
    /// Shell type derived from actual running process binary name, e.g. "zsh", "bash", "fish".
    /// NOT derived from $SHELL env var (Pitfall 7 from research).
    pub shell_type: Option<String>,
    /// Current working directory of the foreground shell process.
    /// Read via darwin libproc proc_pidinfo with PROC_PIDVNODEPATHINFO flavor.
    pub cwd: Option<String>,
    /// Visible terminal output filtered to remove prompts/noise.
    /// Always None in Plan 01 -- Plan 02 adds Accessibility API text reading.
    pub visible_output: Option<String>,
    /// Name of the process running inside the shell, if any (e.g., "node", "python").
    /// None if the shell is idle (no foreground child process).
    pub running_process: Option<String>,
}

/// Main terminal detection entry point.
///
/// Given the PID of the application that was frontmost BEFORE our overlay appeared,
/// determines if it was a known terminal emulator, then reads its context
/// (CWD, shell type, running process) via libproc.
///
/// Returns None if:
/// - The PID does not belong to a known terminal bundle ID
/// - Bundle ID cannot be resolved for the given PID
///
/// Returns Some(TerminalContext) with fields that may individually be None if
/// the corresponding information could not be read.
pub fn detect(previous_app_pid: i32) -> Option<TerminalContext> {
    // 1. Get bundle ID of the previous frontmost app
    let bundle_id = detect::get_bundle_id(previous_app_pid)?;

    // 2. Check if it is a known terminal emulator
    if !detect::is_known_terminal(&bundle_id) {
        // Not a terminal -- overlay works fine without terminal context
        return None;
    }

    // 3. Find foreground process (shell or running command) via libproc
    let proc_info = process::get_foreground_info(previous_app_pid);

    Some(TerminalContext {
        shell_type: proc_info.shell_type,
        cwd: proc_info.cwd,
        visible_output: None, // Plan 02 adds Accessibility API text reading
        running_process: proc_info.running_process,
    })
}
