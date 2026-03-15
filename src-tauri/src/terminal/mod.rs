pub mod ax_reader;
pub mod browser;
pub mod detect;
#[allow(dead_code)]
pub mod detect_linux;
#[allow(dead_code)]
pub mod detect_windows;
pub mod context;
pub mod filter;
pub mod process;

#[cfg(target_os = "windows")]
pub mod uia_reader;

pub mod linux_reader;

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
    /// True when the session is running inside Windows Subsystem for Linux.
    /// Always false on non-Windows platforms.
    pub is_wsl: bool,
}

/// Full context about the frontmost application, returned to the frontend.
///
/// Includes app identity, terminal context (if applicable), and browser console state.
/// Returned for ANY frontmost app, not just terminals.
#[derive(Debug, Clone, Serialize)]
pub struct AppContext {
    /// Display name of the frontmost app, cleaned for badge display.
    /// e.g., "Chrome", "Code", "Terminal", "Finder"
    pub app_name: Option<String>,
    /// Terminal context (CWD, shell type, visible output, running process).
    /// Present when frontmost app is a terminal or has an integrated terminal with a shell.
    pub terminal: Option<TerminalContext>,
    /// Whether browser DevTools console was detected open.
    pub console_detected: bool,
    /// Last line from browser console output (filtered for sensitive data).
    pub console_last_line: Option<String>,
    /// Visible text from any non-terminal app, read via generic AX tree walking.
    /// Populated for apps like Notion, browsers, editors that expose text via Accessibility.
    /// Up to ~4KB of concatenated text from AXStaticText, AXTextField, AXTextArea elements.
    pub visible_text: Option<String>,
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

/// Public API: full app context detection with 500ms hard timeout.
///
/// Returns AppContext for ANY frontmost app (not just terminals). Includes:
/// - Cleaned app display name
/// - Terminal context (CWD, shell, output) if a shell was found in the process tree
/// - Browser console detection state if the app is a known browser
///
/// Returns None only if the timeout expires before detection completes.
pub fn detect_full(previous_app_pid: i32, pre_captured_text: Option<String>) -> Option<AppContext> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = detect_app_context(previous_app_pid, pre_captured_text);
        let _ = tx.send(result);
    });
    rx.recv_timeout(Duration::from_millis(750)).ok().flatten()
}

/// Public API: full app context detection with HWND for Windows UIA text reading.
///
/// Same as detect_full but also takes the previous HWND so the Windows path
/// can read terminal text via UI Automation.
pub fn detect_full_with_hwnd(
    previous_app_pid: i32,
    pre_captured_text: Option<String>,
    previous_hwnd: Option<isize>,
) -> Option<AppContext> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        // Windows: read UIA text FIRST to extract shell type hint for process tree disambiguation
        #[cfg(target_os = "windows")]
        let (uia_text, shell_type_hint) = {
            let uia = previous_hwnd.and_then(|hwnd| {
                uia_reader::read_terminal_text_windows(hwnd)
                    .map(|text| filter::filter_sensitive(&text))
            });
            let hint = uia.as_deref().and_then(detect_windows::detect_shell_type_from_uia_text);
            eprintln!("[detect_full_with_hwnd] UIA shell_type_hint: {:?}", hint);
            (uia, hint)
        };

        // Windows: call detect_app_context_windows directly with shell_type_hint
        #[cfg(target_os = "windows")]
        #[allow(unused_mut)]
        let mut result = detect_app_context_windows(previous_app_pid, pre_captured_text, shell_type_hint);

        // Non-Windows: use generic detect_app_context (no shell_type_hint needed)
        #[cfg(not(target_os = "windows"))]
        #[allow(unused_mut)]
        let mut result = detect_app_context(previous_app_pid, pre_captured_text);

        // Windows: window title WSL detection + UIA text processing (reusing already-captured UIA text)
        #[cfg(target_os = "windows")]
        if let Some(ref mut ctx) = result {
            if let Some(ref mut terminal) = ctx.terminal {
                // Step 0: Window title WSL detection (fast, < 1ms)
                // VS Code/Cursor Remote-WSL shows "[WSL: Ubuntu]" in the window title.
                // This is the FASTEST and most reliable WSL signal for IDE terminals.
                if let Some(hwnd) = previous_hwnd {
                    if !terminal.is_wsl {
                        if let Some(title) = detect_windows::get_window_title(hwnd) {
                            if detect_windows::detect_wsl_from_title(&title) {
                                eprintln!("[detect_full_with_hwnd] WSL detected from window title: {}", &title);
                                terminal.is_wsl = true;
                                // For Remote-WSL, the process tree shell (cmd.exe) is wrong.
                                // Get Linux shell and CWD via WSL subprocess as initial values.
                                // UIA text inference will override these if it finds better data.
                                terminal.cwd = process::get_wsl_cwd().or(terminal.cwd.take());
                                terminal.shell_type = process::get_wsl_shell().or(terminal.shell_type.take());
                            }
                        }
                    }
                }

                // Reuse already-captured UIA text (no double reads)
                if terminal.visible_output.is_none() {
                    if let Some(ref text) = uia_text {
                        eprintln!("[detect_full_with_hwnd] UIA text: {} bytes, content: {:?}", text.len(), &text[..text.len().min(500)]);

                        // Step 1: Try UIA text-based WSL detection UNCONDITIONALLY
                        // This is the PRIMARY WSL detection mechanism.
                        // Process tree ancestry (detect_wsl_in_ancestry) is the fallback,
                        // but it fails for WSL 2 where Linux processes run in Hyper-V VM.
                        if !terminal.is_wsl {
                            if detect_wsl_from_text(text) {
                                eprintln!("[detect_full_with_hwnd] WSL detected from UIA text (process tree missed it)");
                                terminal.is_wsl = true;
                            }
                        }

                        // Step 2: If WSL (from either process tree or UIA text), infer Linux context
                        if terminal.is_wsl {
                            if let Some(linux_cwd) = infer_linux_cwd_from_text(text) {
                                eprintln!("[detect_full_with_hwnd] inferred Linux CWD from UIA text: {}", &linux_cwd);
                                terminal.cwd = Some(linux_cwd);
                            }
                            let shell = infer_shell_from_text(text);
                            eprintln!("[detect_full_with_hwnd] WSL shell from UIA text: {}", shell);
                            terminal.shell_type = Some(shell.to_string());
                        } else if terminal.cwd.is_none() {
                            // Non-WSL: infer shell type from visible text when process tree detection failed
                            let shell = infer_shell_from_text(text);
                            eprintln!("[detect_full_with_hwnd] inferred shell from UIA text: {}", shell);
                            terminal.shell_type = Some(shell.to_string());
                        }
                    }
                    terminal.visible_output = uia_text;
                }

                // Final fallback: detect WSL from CWD path style
                // Catches cases where CWD is \\wsl$\..., \\wsl.localhost\..., or /home/...
                if !terminal.is_wsl {
                    if let Some(ref cwd) = terminal.cwd {
                        if detect_wsl_from_cwd(cwd) {
                            eprintln!("[detect_full_with_hwnd] WSL detected from CWD path style: {}", cwd);
                            terminal.is_wsl = true;
                            terminal.cwd = process::get_wsl_cwd().or(terminal.cwd.take());
                            terminal.shell_type = process::get_wsl_shell().or(terminal.shell_type.take());
                        }
                    }
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        let _ = previous_hwnd;

        let _ = tx.send(result);
    });
    rx.recv_timeout(Duration::from_millis(750)).ok().flatten()
}

/// Inner detection logic (runs on background thread spawned by detect()).
///
/// macOS:
/// 1. Resolve bundle ID of previous frontmost app via NSRunningApplication ObjC FFI.
/// 2. Check if it is a known terminal emulator.
/// 3. Read process info (CWD, shell name, running process) via libproc.
/// 4. For non-GPU terminals: read visible output via Accessibility API.
/// 5. Filter captured text for sensitive data before returning.
///
/// Windows:
/// 1. Resolve exe name from PID via QueryFullProcessImageNameW.
/// 2. Check if it is a known terminal or IDE.
/// 3. Walk process tree to find shell child.
/// 4. Read CWD via PEB, visible output via UIA.
fn detect_inner(previous_app_pid: i32) -> Option<TerminalContext> {
    eprintln!("[detect_inner] starting for pid {}", previous_app_pid);

    #[cfg(target_os = "macos")]
    {
        // 1. Get bundle ID of the previous frontmost app.
        let bundle_id = detect::get_bundle_id(previous_app_pid);
        let bundle_str = bundle_id.as_deref().unwrap_or("unknown");
        let is_terminal = bundle_id.as_deref().is_some_and(detect::is_known_terminal);

        if is_terminal {
            eprintln!("[detect_inner] {} IS a known terminal", bundle_str);
        } else {
            eprintln!("[detect_inner] {} is NOT a known terminal, trying generic shell detection", bundle_str);
        }

        // 2. Read process info (CWD, shell type, running process) via libproc.
        let proc_info = process::get_foreground_info(previous_app_pid);

        if proc_info.shell_type.is_none() && proc_info.cwd.is_none() {
            eprintln!("[detect_inner] no shell found in process tree of {} ({})", previous_app_pid, bundle_str);
            return None;
        }

        // 3. Read visible output via AX only for known AX-capable terminals.
        let visible_output = if is_terminal {
            let is_gpu = bundle_id.as_deref().is_none_or(detect::is_gpu_terminal);
            if !is_gpu {
                ax_reader::read_terminal_text(previous_app_pid, bundle_id.as_deref().unwrap_or(""))
                    .map(|text| filter::filter_sensitive(&text))
            } else {
                None
            }
        } else {
            None
        };

        Some(TerminalContext {
            shell_type: proc_info.shell_type,
            cwd: proc_info.cwd,
            visible_output,
            running_process: proc_info.running_process,
            is_wsl: false,
        })
    }

    #[cfg(target_os = "windows")]
    {
        detect_inner_windows(previous_app_pid)
    }

    #[cfg(target_os = "linux")]
    {
        detect_inner_linux(previous_app_pid)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let _ = previous_app_pid;
        None
    }
}

/// Linux-specific inner detection.
#[cfg(target_os = "linux")]
fn detect_inner_linux(previous_app_pid: i32) -> Option<TerminalContext> {
    let exe_name = detect_linux::get_exe_name_for_pid(previous_app_pid);
    let exe_str = exe_name.as_deref().unwrap_or("unknown");
    let is_terminal = detect_linux::is_known_terminal_exe(exe_str);
    let is_ide = detect_linux::is_ide_with_terminal_exe(exe_str);

    eprintln!("[detect_inner_linux] exe={} is_terminal={} is_ide={}", exe_str, is_terminal, is_ide);

    if !is_terminal && !is_ide {
        eprintln!("[detect_inner_linux] {} is not a terminal or IDE, trying generic shell detection", exe_str);
    }

    // Walk process tree to find shell (reuses Plan 01 /proc implementation)
    let proc_info = process::get_foreground_info(previous_app_pid);

    if proc_info.shell_type.is_none() && proc_info.cwd.is_none() {
        eprintln!("[detect_inner_linux] no shell found in process tree of {} ({})", previous_app_pid, exe_str);
        return None;
    }

    let visible_output = if is_terminal {
        linux_reader::read_terminal_text_linux(previous_app_pid, exe_str)
            .map(|text| filter::filter_sensitive(&text))
    } else {
        None
    };

    Some(TerminalContext {
        shell_type: proc_info.shell_type,
        cwd: proc_info.cwd,
        visible_output,
        running_process: proc_info.running_process,
        is_wsl: false,
    })
}

/// Windows-specific inner detection.
#[cfg(target_os = "windows")]
fn detect_inner_windows(previous_app_pid: i32) -> Option<TerminalContext> {
    let exe_name = detect_windows::get_exe_name_for_pid(previous_app_pid as u32);
    let exe_str = exe_name.as_deref().unwrap_or("unknown");
    let is_terminal = detect_windows::is_known_terminal_exe(exe_str);
    let is_ide = detect_windows::is_ide_with_terminal_exe(exe_str);

    eprintln!("[detect_inner_windows] exe={} is_terminal={} is_ide={}", exe_str, is_terminal, is_ide);

    if !is_terminal && !is_ide {
        eprintln!("[detect_inner_windows] {} is not a terminal or IDE, trying generic shell detection", exe_str);
    }

    // Walk process tree to find shell (no shared snapshot in detect_inner path, no UIA hint)
    let proc_info = process::get_foreground_info(previous_app_pid, None, None);

    if proc_info.shell_type.is_none() && proc_info.cwd.is_none() {
        eprintln!("[detect_inner_windows] no shell found in process tree of {} ({})", previous_app_pid, exe_str);
        return None;
    }

    // Read visible output via UIA for known terminals
    let visible_output = if is_terminal {
        // GPU terminals (Alacritty, kitty, WezTerm) may not support UIA text reading
        let is_gpu = matches!(exe_str.to_lowercase().as_str(), "alacritty.exe" | "kitty.exe" | "wezterm-gui.exe");
        if !is_gpu {
            // Read the HWND from AppState (set by hotkey handler) for UIA
            // Since we don't have AppState here, try reading from the PID's windows
            None // UIA reading will be wired via detect_app_context with hwnd
        } else {
            None
        }
    } else {
        None
    };

    let is_wsl = proc_info.is_wsl;

    // If WSL session detected, override CWD and shell with Linux-native values
    let (cwd, shell_type) = if is_wsl {
        let wsl_cwd = process::get_wsl_cwd().or(proc_info.cwd);
        let wsl_shell = process::get_wsl_shell().or(proc_info.shell_type);
        (wsl_cwd, wsl_shell)
    } else {
        (proc_info.cwd, proc_info.shell_type)
    };

    Some(TerminalContext {
        shell_type,
        cwd,
        visible_output,
        running_process: proc_info.running_process,
        is_wsl,
    })
}

/// Full context orchestrator (runs on background thread spawned by detect_full()).
///
/// macOS:
/// 1. Get bundle ID and localized display name from NSRunningApplication.
/// 2. Read process info to determine if a shell is present (terminal context).
/// 3. For AX-capable terminals, read visible output.
/// 4. For known browsers without a shell, attempt DevTools console detection.
/// 5. Return AppContext wrapping all gathered state.
///
/// Windows:
/// 1. Get exe name and derive display name.
/// 2. Walk process tree for shell detection.
/// 3. Read CWD via PEB, visible output via UIA.
fn detect_app_context(previous_app_pid: i32, pre_captured_text: Option<String>) -> Option<AppContext> {
    eprintln!("[detect_app_context] starting for pid {}", previous_app_pid);

    #[cfg(target_os = "macos")]
    {
        detect_app_context_macos(previous_app_pid, pre_captured_text)
    }

    #[cfg(target_os = "windows")]
    {
        detect_app_context_windows(previous_app_pid, pre_captured_text, None)
    }

    #[cfg(target_os = "linux")]
    {
        detect_app_context_linux(previous_app_pid, pre_captured_text)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let _ = (previous_app_pid, pre_captured_text);
        None
    }
}

/// Linux-specific full app context detection.
#[cfg(target_os = "linux")]
fn detect_app_context_linux(previous_app_pid: i32, _pre_captured_text: Option<String>) -> Option<AppContext> {
    let exe_name = detect_linux::get_exe_name_for_pid(previous_app_pid);
    let exe_str = exe_name.as_deref().unwrap_or("unknown");
    let app_name = Some(detect_linux::clean_linux_app_name(exe_str));

    eprintln!("[detect_app_context_linux] exe={} app_name={:?}", exe_str, &app_name);

    // Walk process tree to find shell (reuses Plan 01 /proc implementation)
    let proc_info = process::get_foreground_info(previous_app_pid);
    let has_shell = proc_info.shell_type.is_some() || proc_info.cwd.is_some();

    let is_terminal = detect_linux::is_known_terminal_exe(exe_str);
    let is_ide = detect_linux::is_ide_with_terminal_exe(exe_str);

    let terminal = if has_shell {
        let visible_output = if is_terminal || is_ide {
            linux_reader::read_terminal_text_linux(previous_app_pid, exe_str)
                .map(|text| filter::filter_sensitive(&text))
        } else {
            None
        };

        Some(TerminalContext {
            shell_type: proc_info.shell_type,
            cwd: proc_info.cwd,
            visible_output,
            running_process: proc_info.running_process,
            is_wsl: false,
        })
    } else {
        None
    };

    Some(AppContext {
        app_name,
        terminal,
        console_detected: false,
        console_last_line: None,
        visible_text: None,
    })
}

#[cfg(target_os = "macos")]
fn detect_app_context_macos(previous_app_pid: i32, pre_captured_text: Option<String>) -> Option<AppContext> {
    // 1. Get bundle ID and display name.
    let bundle_id = detect::get_bundle_id(previous_app_pid);
    let raw_name = detect::get_app_display_name(previous_app_pid);
    let app_name = raw_name.map(|n| detect::clean_app_name(&n));

    let bundle_str = bundle_id.as_deref().unwrap_or("unknown");
    let is_terminal = bundle_id.as_deref().is_some_and(detect::is_known_terminal);

    eprintln!("[detect_app_context] bundle={} app_name={:?}", bundle_str, app_name);

    // 2. Read process info (CWD, shell type, running process).
    let proc_info = process::get_foreground_info(previous_app_pid);
    let has_shell = proc_info.shell_type.is_some() || proc_info.cwd.is_some();

    // 3. Build terminal context if a shell was found.
    let terminal = if has_shell {
        let visible_output = if is_terminal {
            let is_gpu = bundle_id.as_deref().is_none_or(detect::is_gpu_terminal);
            if !is_gpu {
                ax_reader::read_terminal_text(
                    previous_app_pid,
                    bundle_id.as_deref().unwrap_or(""),
                )
                .map(|text| filter::filter_sensitive(&text))
            } else {
                None
            }
        } else {
            None
        };

        Some(TerminalContext {
            shell_type: proc_info.shell_type,
            cwd: proc_info.cwd,
            visible_output,
            running_process: proc_info.running_process,
            is_wsl: false,
        })
    } else {
        None
    };

    // 4. Browser console detection (only for known browsers that have no shell).
    let is_browser = bundle_id.as_deref().is_some_and(detect::is_known_browser);
    let (console_detected, console_last_line) = if is_browser && terminal.is_none() {
        let (detected, line) = browser::detect_console(
            previous_app_pid,
            bundle_id.as_deref().unwrap_or(""),
        );
        let filtered_line = line.map(|l| filter::filter_sensitive(&l));
        eprintln!(
            "[detect_app_context] browser console detected={} last_line={:?}",
            detected, filtered_line
        );
        (detected, filtered_line)
    } else {
        (false, None)
    };

    // 5. Generic AX text reading for non-terminal apps.
    let has_visible_output = terminal
        .as_ref()
        .and_then(|t| t.visible_output.as_ref())
        .is_some();
    let visible_text = if !is_terminal && !has_visible_output {
        if let Some(pre) = pre_captured_text {
            eprintln!("[detect_app_context] using pre-captured text ({} bytes)", pre.len());
            Some(filter::filter_sensitive(&pre))
        } else {
            ax_reader::read_focused_text(previous_app_pid)
                .map(|text| filter::filter_sensitive(&text))
        }
    } else {
        None
    };
    eprintln!("[detect_app_context] visible_text len={:?}", visible_text.as_ref().map(|t| t.len()));

    Some(AppContext {
        app_name,
        terminal,
        console_detected,
        console_last_line,
        visible_text,
    })
}

#[cfg(target_os = "windows")]
fn detect_app_context_windows(previous_app_pid: i32, _pre_captured_text: Option<String>, shell_type_hint: Option<&str>) -> Option<AppContext> {
    let exe_name = detect_windows::get_exe_name_for_pid(previous_app_pid as u32);
    let exe_str = exe_name.as_deref().unwrap_or("unknown");
    let app_name = Some(detect_windows::clean_exe_name(exe_str));
    let _is_terminal = detect_windows::is_known_terminal_exe(exe_str);

    eprintln!("[detect_app_context_windows] exe={} app_name={:?} shell_type_hint={:?}", exe_str, &app_name, shell_type_hint);

    // Create a single ProcessSnapshot for the entire detection pipeline (PROC-03)
    let snapshot = process::ProcessSnapshot::capture();
    eprintln!("[detect_app_context_windows] ProcessSnapshot: {}", if snapshot.is_some() { "captured" } else { "failed" });

    // Walk process tree to find shell using shared snapshot, with UIA shell hint for disambiguation
    let proc_info = process::get_foreground_info(previous_app_pid, snapshot.as_ref(), shell_type_hint);
    let has_shell = proc_info.shell_type.is_some() || proc_info.cwd.is_some();

    let terminal = if has_shell {
        let mut is_wsl = proc_info.is_wsl;

        // Fallback: detect WSL from CWD path style (\\wsl$\..., \\wsl.localhost\..., /home/...)
        if !is_wsl {
            if let Some(ref cwd) = proc_info.cwd {
                if detect_wsl_from_cwd(cwd) {
                    eprintln!("[detect_app_context_windows] WSL detected from CWD path style: {}", cwd);
                    is_wsl = true;
                }
            }
        }

        // Strip ".exe" from shell_type if present (e.g., "powershell.exe" -> "powershell")
        let shell_type = proc_info.shell_type.map(|s| {
            s.trim_end_matches(".exe")
                .trim_end_matches(".EXE")
                .to_lowercase()
        });

        // If WSL session detected, override CWD and shell with Linux-native values
        let (cwd, shell_type) = if is_wsl {
            let wsl_cwd = process::get_wsl_cwd().or(proc_info.cwd);
            let wsl_shell = process::get_wsl_shell().or(shell_type);
            (wsl_cwd, wsl_shell)
        } else {
            (proc_info.cwd, shell_type)
        };

        Some(TerminalContext {
            shell_type,
            cwd,
            visible_output: None, // UIA reading done separately via get_terminal_output command
            running_process: proc_info.running_process,
            is_wsl,
        })
    } else if _is_terminal {
        // Known terminal (e.g. WindowsTerminal.exe) but no shell found in process tree.
        // Windows Terminal uses ConPTY where shell processes are not descendants of WT.
        // Default to "powershell" to ensure terminal mode is used.
        eprintln!("[detect_app_context_windows] known terminal {} but no shell in process tree, defaulting to powershell", exe_str);
        Some(TerminalContext {
            shell_type: Some("powershell".to_string()),
            cwd: None,
            visible_output: None,
            running_process: None,
            is_wsl: false,
        })
    } else {
        None
    };

    Some(AppContext {
        app_name,
        terminal,
        console_detected: false,
        console_last_line: None,
        visible_text: None,
    })
}

/// Detect WSL from CWD path style.
/// WSL UNC paths: \\wsl$\Ubuntu\... or \\wsl.localhost\Ubuntu\...
/// Linux absolute paths: /home/... (if CWD was set from WSL subprocess)
#[cfg(target_os = "windows")]
fn detect_wsl_from_cwd(cwd: &str) -> bool {
    cwd.starts_with("\\\\wsl$\\")
        || cwd.starts_with("\\\\wsl.localhost\\")
        || cwd.starts_with("/")
}

/// Infer shell type from visible terminal text.
/// "PS C:\>" or "PS>" patterns -> powershell, "C:\>" without PS -> cmd, "$" prompt -> bash, "#" prompt -> bash (root).
fn infer_shell_from_text(text: &str) -> &'static str {
    // Check ALL lines — UIA text can have prompts at the top and UI chrome at the bottom
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("PS ") || trimmed.starts_with("PS>") {
            return "powershell";
        }
        // CMD prompt: "C:\path>" without leading "PS "
        if trimmed.len() >= 3 && trimmed.chars().nth(1) == Some(':') && trimmed.contains('>') && !trimmed.starts_with("PS") {
            return "cmd";
        }
        // Linux bash/zsh prompt: user@host:/path or user@host:~
        if trimmed.contains('@') && trimmed.contains(':') {
            if let Some(colon_pos) = trimmed.find(':') {
                let before_colon = &trimmed[..colon_pos];
                let after_colon = &trimmed[colon_pos + 1..];
                if before_colon.contains('@')
                    && (after_colon.starts_with('/')
                        || after_colon.starts_with('~')
                        || after_colon.trim_start().starts_with('/')
                        || after_colon.trim_start().starts_with('~'))
                {
                    return "bash";
                }
            }
        }
        // Fish prompt: "user@host /path>"
        if trimmed.contains('@') && trimmed.ends_with('>') && !trimmed.contains('\\') {
            return "fish";
        }
        // Bash/zsh prompt (user or root)
        if trimmed.ends_with('$') || trimmed.contains("$ ") {
            return "bash";
        }
        // Root prompt (common in Linux/WSL)
        if trimmed.contains('@') && (trimmed.ends_with('#') || trimmed.contains("# ")) {
            return "bash";
        }
    }
    "powershell" // default fallback
}

/// Detect WSL from visible terminal text (UIA).
/// This is the PRIMARY WSL detection mechanism -- process tree ancestry fails for WSL 2
/// where Linux processes run in a Hyper-V VM invisible to Windows process APIs.
///
/// Uses a multi-signal scoring system (threshold >= 2) to avoid false positives
/// from editor content that happens to contain Linux paths:
/// - score += 1 for Linux paths (/home/, /root/, /var/, /etc/, /usr/, /tmp/, /opt/)
/// - score += 2 for WSL mount paths (/mnt/c/, /mnt/d/, etc.) -- strong signal
/// - score += 1 for user@host:/path or user@host:~ prompt pattern
/// - score += 1 for user@host...$ or user@host...# prompt ending
#[allow(dead_code)]
fn detect_wsl_from_text(text: &str) -> bool {
    eprintln!("[detect_wsl_from_text] scanning {} bytes of UIA text", text.len());
    let mut score: u32 = 0;

    // Signal 1: WSL mount paths (/mnt/c/, /mnt/d/, etc.) -- strong signal, score 2
    // These are unambiguous: only WSL mounts Windows drives at /mnt/<letter>/
    let has_wsl_mount = ('a'..='z').any(|c| text.contains(&format!("/mnt/{}/", c)));
    if has_wsl_mount {
        score += 2;
        eprintln!("[detect_wsl_from_text] +2 WSL mount path detected (score={})", score);
    }

    // Signal 2: Linux paths (/home/, /root/, /var/, /etc/, /usr/, /tmp/, /opt/) -- weak signal, score 1
    // A single Linux path could be editor content (Dockerfile, README, config file)
    let linux_paths = ["/home/", "/root/", "/var/", "/etc/", "/usr/", "/tmp/", "/opt/"];
    if score < 2 {
        for lp in &linux_paths {
            if text.contains(lp) {
                score += 1;
                eprintln!("[detect_wsl_from_text] +1 Linux path {} (score={})", lp, score);
                break; // Only count once even if multiple paths present
            }
        }
    }

    // Signal 3 & 4: Check lines for prompt patterns
    let mut has_prompt_pattern = false; // user@host:/path or user@host:~
    let mut has_prompt_ending = false;  // user@host...$ or user@host...#

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Pattern: user@host:/path or user@host:~ (canonical Linux prompt)
        if !has_prompt_pattern {
            if let Some(colon_pos) = trimmed.find(':') {
                let before_colon = &trimmed[..colon_pos];
                if before_colon.contains('@') {
                    let after_colon = &trimmed[colon_pos + 1..];
                    let path_part = after_colon
                        .trim_end_matches(|c: char| c == '$' || c == '#' || c == ' ')
                        .trim();
                    if path_part.starts_with('/') || path_part.starts_with('~') {
                        has_prompt_pattern = true;
                        score += 1;
                        eprintln!("[detect_wsl_from_text] +1 prompt pattern '{}' (score={})", trimmed, score);
                    }
                }
            }
        }

        // Pattern: user@host...$ or user@host...# without Windows backslash paths
        if !has_prompt_ending {
            if trimmed.contains('@') {
                if let Some(at_pos) = trimmed.find('@') {
                    let after_at = &trimmed[at_pos + 1..];
                    let host_part = after_at.split(|c: char| c.is_whitespace() || c == ':').next().unwrap_or("");
                    if !host_part.contains('\\')
                        && (trimmed.ends_with('$') || trimmed.ends_with('#')
                            || trimmed.contains("$ ") || trimmed.contains("# "))
                    {
                        has_prompt_ending = true;
                        score += 1;
                        eprintln!("[detect_wsl_from_text] +1 prompt ending '{}' (score={})", trimmed, score);
                    }
                }
            }
        }

        // Early exit if we already have enough
        if score >= 2 {
            break;
        }
    }

    eprintln!("[detect_wsl_from_text] final score={} (threshold=2), result={}", score, score >= 2);
    score >= 2
}

/// Attempt to infer the Linux CWD from visible terminal text.
/// Looks for common prompt patterns: user@host:/path$ or user@host:/path #
#[cfg(target_os = "windows")]
fn infer_linux_cwd_from_text(text: &str) -> Option<String> {
    // Check ALL lines — UIA text can have prompts at top and UI chrome at bottom
    for line in text.lines() {
        let trimmed = line.trim();
        // Pattern: user@host:/absolute/path$ or user@host:/path #
        if let Some(colon_pos) = trimmed.find(':') {
            if trimmed[..colon_pos].contains('@') {
                let after_colon = &trimmed[colon_pos + 1..];
                let path = after_colon
                    .trim_end_matches(|c: char| c == '$' || c == '#' || c == ' ')
                    .trim();
                if path.starts_with('/') || path == "~" {
                    return Some(path.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::detect_wsl_from_text;

    // === MUST RETURN FALSE (single weak signal) ===

    #[test]
    fn test_single_linux_path_no_prompt_is_false() {
        // A single /home/user path without any prompt should NOT trigger WSL
        assert!(!detect_wsl_from_text("/home/user/.bashrc"));
    }

    #[test]
    fn test_editor_content_etc_path_is_false() {
        // Editor viewing a config file with Linux path
        assert!(!detect_wsl_from_text("/etc/nginx/nginx.conf"));
    }

    #[test]
    fn test_dockerfile_content_is_false() {
        // Dockerfile content with Linux paths should not false-positive
        let text = "FROM ubuntu:22.04\nRUN apt-get update\nCOPY /home/app/config .";
        assert!(!detect_wsl_from_text(text));
    }

    #[test]
    fn test_powershell_viewing_linux_path_is_false() {
        // PowerShell prompt with a Linux path argument
        let text = "PS C:\\Users\\dev> cat /home/user/readme.md";
        assert!(!detect_wsl_from_text(text));
    }

    #[test]
    fn test_user_at_host_without_path_or_prompt_ending_is_false() {
        // Just user@host with no path or prompt ending
        assert!(!detect_wsl_from_text("user@host"));
    }

    // === MUST RETURN TRUE (multiple signals or strong signal) ===

    #[test]
    fn test_wsl_mount_mnt_c_is_true() {
        // WSL mount path is a strong signal (score 2)
        assert!(detect_wsl_from_text("/mnt/c/Users/dev"));
    }

    #[test]
    fn test_wsl_mount_mnt_d_is_true() {
        // WSL mount path for D: drive
        assert!(detect_wsl_from_text("/mnt/d/projects"));
    }

    #[test]
    fn test_prompt_with_path_is_true() {
        // user@host:/path$ is prompt + path = score 2
        assert!(detect_wsl_from_text("user@ubuntu:/home/user$"));
    }

    #[test]
    fn test_linux_path_plus_prompt_is_true() {
        // Linux path on one line, prompt on another = score 2
        let text = "/home/user\nuser@host:~$";
        assert!(detect_wsl_from_text(text));
    }

    #[test]
    fn test_prompt_pattern_with_ending_is_true() {
        // Full WSL terminal with prompt pattern + ending = score 2
        let text = "user@myhost:~/projects$ ls\nfile1 file2\nuser@myhost:~/projects$";
        assert!(detect_wsl_from_text(text));
    }

    #[test]
    fn test_full_wsl_terminal_output_is_true() {
        // Full WSL terminal output with multiple signals
        let text = "user@ubuntu:~$ cd /home/user/projects\nuser@ubuntu:~/projects$ ls\nfile1\nuser@ubuntu:~/projects$";
        assert!(detect_wsl_from_text(text));
    }

    #[test]
    fn test_mnt_c_alone_is_true() {
        // /mnt/c/ alone is a strong signal (score 2)
        assert!(detect_wsl_from_text("/mnt/c/"));
    }
}
