pub mod ax_reader;
pub mod detect;
pub mod filter;
pub mod process;

use serde::Serialize;
use std::sync::mpsc;
use std::time::Duration;

/// Context information about the terminal that was frontmost before the overlay appeared.
///
/// All fields are Option<String> because detection can fail at any step:
/// - CWD: the process may not have a valid CWD accessible via libproc
/// - shell_type: derived from binary name, may be unknown
/// - visible_output: populated by AX text reading for Terminal.app and iTerm2;
///   always None for GPU-rendered terminals (Alacritty, kitty, WezTerm)
/// - running_process: only present when something is running inside the shell
#[derive(Debug, Clone, Serialize)]
pub struct TerminalContext {
    /// Shell type derived from actual running process binary name, e.g. "zsh", "bash", "fish".
    /// NOT derived from $SHELL env var (Pitfall 7 from research).
    pub shell_type: Option<String>,
    /// Current working directory of the foreground shell process.
    /// Read via darwin libproc proc_pidinfo with PROC_PIDVNODEPATHINFO flavor.
    pub cwd: Option<String>,
    /// Visible terminal output captured via the Accessibility API and filtered for secrets.
    /// Some(text) for Terminal.app and iTerm2; None for GPU-rendered terminals.
    pub visible_output: Option<String>,
    /// Name of the process running inside the shell, if any (e.g., "node", "python").
    /// None if the shell is idle (no foreground child process).
    pub running_process: Option<String>,
}

/// Public API: terminal detection with 500ms hard timeout.
///
/// Wraps detect_inner() in a background thread with a 500ms recv_timeout.
/// This ensures the overlay never stalls even if AX messaging is slow.
///
/// The per-element AX messaging timeout (1 second, set in ax_reader.rs) is the
/// inner guard. This outer 500ms timeout is the pipeline-level safety net.
///
/// Returns None if:
/// - The timeout expires before detection completes
/// - The previous app PID does not belong to a known terminal emulator
/// - Bundle ID cannot be resolved for the given PID
pub fn detect(previous_app_pid: i32) -> Option<TerminalContext> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = detect_inner(previous_app_pid);
        // Ignore send error -- receiver may have timed out and dropped.
        let _ = tx.send(result);
    });
    rx.recv_timeout(Duration::from_millis(500)).ok().flatten()
}

/// Inner detection logic (runs on background thread spawned by detect()).
///
/// 1. Resolve bundle ID of previous frontmost app via NSRunningApplication ObjC FFI.
/// 2. Check if it is a known terminal emulator.
/// 3. Read process info (CWD, shell name, running process) via libproc.
/// 4. For non-GPU terminals: read visible output via Accessibility API.
/// 5. Filter captured text for sensitive data before returning.
fn detect_inner(previous_app_pid: i32) -> Option<TerminalContext> {
    // 1. Get bundle ID of the previous frontmost app.
    let bundle_id = detect::get_bundle_id(previous_app_pid)?;

    // 2. Check if it is a known terminal emulator.
    if !detect::is_known_terminal(&bundle_id) {
        // Not a terminal -- overlay works fine without terminal context.
        return None;
    }

    // 3. Check if this is a GPU-rendered terminal (cannot read AX text).
    let is_gpu = detect::is_gpu_terminal(&bundle_id);

    // 4. Read process info (CWD, shell type, running process) via libproc.
    let proc_info = process::get_foreground_info(previous_app_pid);

    // 5. Read visible output for AX-capable terminals only (Terminal.app, iTerm2).
    //    GPU terminals (Alacritty, kitty, WezTerm) return None silently --
    //    this is not an error, it is expected behavior.
    let visible_output = if !is_gpu {
        ax_reader::read_terminal_text(previous_app_pid, &bundle_id)
            .map(|text| filter::filter_sensitive(&text))
    } else {
        None // GPU terminals: silent fallback, no AX text available
    };

    Some(TerminalContext {
        shell_type: proc_info.shell_type,
        cwd: proc_info.cwd,
        visible_output,
        running_process: proc_info.running_process,
    })
}
