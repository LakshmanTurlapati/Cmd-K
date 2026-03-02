---
phase: 11-build-infrastructure-overlay-foundation
verified: 2026-03-02T16:30:00Z
status: gaps_found
score: 7/9 must-haves verified
gaps:
  - truth: "Before overlay shows on Windows, the HWND of the currently focused window is captured and stored in AppState.previous_hwnd"
    status: failed
    reason: "The HWND capture code is unreachable on Windows. get_frontmost_pid() returns None on non-macOS (line 55-58 hotkey.rs), so the entire `if let Some(pid) = pid` block -- which contains the Windows HWND capture -- never executes on Windows. The HWND is never written to AppState.previous_hwnd."
    artifacts:
      - path: "src-tauri/src/commands/hotkey.rs"
        issue: "Windows HWND capture at lines 251-263 is inside `if let Some(pid) = pid` which is always None on Windows because get_frontmost_pid() returns None on non-macOS"
    missing:
      - "Move the Windows HWND capture block OUTSIDE the `if let Some(pid) = pid` gate -- it should execute independently of PID capture on Windows"
      - "The HWND capture should be at the same level as `let pid = get_frontmost_pid();`, not nested inside the Some(pid) branch"
  - truth: "On overlay dismiss (Escape or hotkey re-press), focus is restored to the previously captured HWND via AttachThreadInput + SetForegroundWindow"
    status: failed
    reason: "Focus restoration logic in hide_overlay (window.rs lines 104-118) is correctly structured and reads previous_hwnd from AppState. However, because previous_hwnd is never populated (due to the unreachable HWND capture above), the `if let Some(hwnd) = prev_hwnd` branch never triggers. Focus restoration is a dead path."
    artifacts:
      - path: "src-tauri/src/commands/window.rs"
        issue: "Focus restoration reads previous_hwnd which is always None due to the HWND capture bug in hotkey.rs"
    missing:
      - "Fix the HWND capture in hotkey.rs first (gap above), then focus restoration will work as written"
human_verification:
  - test: "Compile and run on Windows hardware, press Ctrl+Shift+K"
    expected: "Overlay appears with Acrylic frosted glass vibrancy above all windows, does not appear in Alt+Tab or taskbar"
    why_human: "Cannot verify visual effects (Acrylic), always-on-top behavior, or Alt+Tab exclusion without Windows hardware"
  - test: "Press Ctrl+Shift+K in a terminal window on Windows, then press Escape"
    expected: "Focus returns to the terminal window that was active before the overlay appeared"
    why_human: "Requires Windows hardware and actual focus management behavior verification"
---

# Phase 11: Build Infrastructure and Overlay Foundation Verification Report

**Phase Goal:** Windows build compiles without breaking macOS, and the overlay window appears with native vibrancy on Ctrl+Shift+K with correct focus management
**Verified:** 2026-03-02T16:30:00Z
**Status:** gaps_found
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Project compiles on macOS with cargo build without regressions | VERIFIED | All macOS-only imports (tauri_nspanel, NSVisualEffectMaterial, accessibility_sys, core_foundation_sys) are behind `#[cfg(target_os = "macos")]` gates. Commits aa53731, e7d0375 document clean `cargo check` on macOS. |
| 2 | macOS-only crates are behind cfg(target_os = "macos") gates | VERIFIED | Cargo.toml lines 35-39: `[target.'cfg(target_os = "macos")'.dependencies]` contains tauri-nspanel, accessibility-sys, core-foundation-sys, keyring. lib.rs lines 21-26 gate all nspanel/vibrancy imports. permissions.rs gates core_foundation_sys usage inside the cfg(target_os = "macos") functions. |
| 3 | Windows-only crate (windows-sys) is behind cfg(target_os = "windows") | VERIFIED | Cargo.toml lines 41-44: `[target.'cfg(target_os = "windows")'.dependencies]` contains windows-sys 0.59 with required features and raw-window-handle 0.6. |
| 4 | window-vibrancy upgraded to 0.7 for HasWindowHandle compatibility | VERIFIED | Cargo.toml line 25: `window-vibrancy = "0.7"` in cross-platform `[dependencies]`. |
| 5 | AppState has previous_hwnd field for Windows HWND tracking | VERIFIED | state.rs lines 92-93: `pub previous_hwnd: Mutex<Option<isize>>` with doc comment. Initialized as `Mutex::new(None)` in Default impl (line 107). |
| 6 | On Windows, overlay displays with Acrylic frosted glass vibrancy and correct window styles | VERIFIED (code) / NEEDS HUMAN (visual) | lib.rs lines 121-155: Windows block applies `apply_acrylic(&window, Some((18, 18, 18, 125)))`, `set_always_on_top(true)`, and WS_EX_TOOLWINDOW via `SetWindowLongPtrW`. Visual verification requires Windows hardware. |
| 7 | Show/hide overlay uses standard Tauri window APIs on Windows (not NSPanel) | VERIFIED | window.rs lines 32-40: `#[cfg(not(target_os = "macos"))]` block uses `window.show()` + `window.set_focus()` for show. Lines 68-100: `window.hide()` for hide. Both paths correctly gated. |
| 8 | Before overlay shows on Windows, the HWND of the currently focused window is captured and stored in AppState.previous_hwnd | FAILED | HWND capture code (hotkey.rs lines 250-263) is inside `if let Some(pid) = pid` block. On Windows, `get_frontmost_pid()` always returns `None` (non-macOS stub at lines 55-58). Therefore `if let Some(pid) = pid` never matches on Windows and the HWND capture is unreachable. `previous_hwnd` is always `None` on Windows. |
| 9 | On overlay dismiss, focus is restored to the previously captured HWND via AttachThreadInput + SetForegroundWindow | FAILED | `restore_focus()` implementation in hotkey.rs (lines 89-141) is correctly implemented with IsWindow validation, AttachThreadInput, SetForegroundWindow, and AllowSetForegroundWindow fallback. The call site in window.rs (lines 104-118) reads `previous_hwnd` and calls `restore_focus`. However, since `previous_hwnd` is never populated (gap above), the `if let Some(hwnd) = prev_hwnd` branch never triggers and focus is never restored. |

**Score:** 7/9 truths verified (2 failed)

---

## Required Artifacts

### Plan 01 Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/Cargo.toml` | Platform-gated dependencies | VERIFIED | Three dependency sections present: `[dependencies]` (cross-platform), `[target.'cfg(target_os = "macos")'.dependencies]`, `[target.'cfg(target_os = "windows")'.dependencies]` |
| `src-tauri/src/lib.rs` | Platform-branching with cfg-gated NSPanel imports | VERIFIED | `#[cfg(target_os = "macos")]` gates on lines 21, 25, 31, 45, 66, 77, 82 |
| `src-tauri/src/state.rs` | AppState with previous_hwnd field | VERIFIED | `previous_hwnd: Mutex<Option<isize>>` at line 93, initialized at line 107 |

### Plan 02 Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/lib.rs` | Windows setup with apply_acrylic and WS_EX_TOOLWINDOW | VERIFIED | Lines 121-155: complete Windows block with apply_acrylic, set_always_on_top, WS_EX_TOOLWINDOW via SetWindowLongPtrW |
| `src-tauri/src/commands/window.rs` | Cross-platform show/hide with Windows path | VERIFIED | Both `show_overlay` and `hide_overlay` have `#[cfg(target_os = "macos")]` and `#[cfg(not(target_os = "macos"))]` paths |

### Plan 03 Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/commands/hotkey.rs` | GetForegroundWindow + restore_focus | PARTIAL | `get_foreground_hwnd()` (lines 65-78) and `restore_focus()` (lines 89-146) are correctly implemented. However HWND capture is dead code -- nested inside `if let Some(pid) = pid` which never matches on Windows. |
| `src-tauri/src/commands/window.rs` | restore_focus call on hide_overlay Windows path | PARTIAL | Code exists and reads `previous_hwnd` correctly (lines 104-118), but `previous_hwnd` is always None because hotkey.rs never writes to it on Windows. |
| `src-tauri/src/state.rs` | previous_hwnd field | VERIFIED | Already confirmed above. |

---

## Key Link Verification

### Plan 01 Key Links

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src-tauri/Cargo.toml` | `src-tauri/src/lib.rs` | Platform-gated dependency availability (`cfg(target_os`) | VERIFIED | Cargo.toml has correct cfg-gated sections; lib.rs uses cfg-gated imports matching those deps |
| `src-tauri/src/lib.rs` | `tauri_nspanel` | `#[cfg(target_os = "macos")]` import gate | VERIFIED | lib.rs lines 21-24: `#[cfg(target_os = "macos")] use tauri_nspanel::{...}` |

### Plan 02 Key Links

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src-tauri/src/lib.rs` | `window-vibrancy` | `apply_acrylic` call | VERIFIED | lib.rs line 123: `use window_vibrancy::apply_acrylic;` inside Windows block; line 128: `apply_acrylic(&window, ...)` |
| `src-tauri/src/lib.rs` | `windows-sys` | WS_EX_TOOLWINDOW style | VERIFIED | lib.rs lines 141-154: `use windows_sys::Win32::UI::WindowsAndMessaging::*;` with GetWindowLongPtrW/SetWindowLongPtrW |
| `src-tauri/src/commands/window.rs` | `src-tauri/src/lib.rs` | `get_webview_window("main")` | VERIFIED | window.rs line 35 (show path) and line 73 (hide path): `app.get_webview_window("main")` |

### Plan 03 Key Links

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src-tauri/src/commands/hotkey.rs` | `src-tauri/src/state.rs` | Stores HWND in `AppState.previous_hwnd` | FAILED | Code at lines 258-262 correctly writes to `state.previous_hwnd.lock()`, but this code is unreachable because it is inside `if let Some(pid) = pid` (always None on Windows) |
| `src-tauri/src/commands/window.rs` | `src-tauri/src/state.rs` | Reads `previous_hwnd` on hide_overlay | PARTIAL | Code reads `state.previous_hwnd` at line 107, but the value is always None -- the read is live but functionally disconnected |
| `src-tauri/src/commands/hotkey.rs` | `GetForegroundWindow` | FFI call inside get_foreground_hwnd | VERIFIED (function) / UNREACHABLE (call site) | `get_foreground_hwnd()` correctly calls `GetForegroundWindow` inside Windows cfg block. The function itself is correct but is called inside the dead `if let Some(pid) = pid` branch. |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| WBLD-01 | 11-01 | Cargo.toml platform-gates macOS-only deps and adds Windows-only deps | SATISFIED | Cargo.toml has three dependency sections: cross-platform, macOS-only, Windows-only. window-vibrancy 0.7 is cross-platform; tauri-nspanel/accessibility-sys/core-foundation-sys are macOS-only; windows-sys 0.59 is Windows-only. |
| WBLD-02 | 11-01 | Project compiles on both macOS and Windows without regressions | SATISFIED (macOS verified, Windows inferred) | Cargo check passed on macOS (confirmed by clean commits). Cross-compilation to Windows cannot be verified on macOS without target installed, but all imports are correctly cfg-gated. |
| WOVL-01 | 11-02 | Overlay window appears with Acrylic (Win10) or Mica (Win11) frosted glass vibrancy | SATISFIED (code) / NEEDS HUMAN (visual) | lib.rs lines 123-130: `apply_acrylic(&window, Some((18, 18, 18, 125)))` inside Windows cfg block. |
| WOVL-02 | 11-02 | Overlay floats above all windows with always-on-top and skip-taskbar behavior | SATISFIED (code) | lib.rs line 133-135: `window.set_always_on_top(true)`. tauri.conf.json line 23: `"skipTaskbar": true`. Also WS_EX_TOOLWINDOW removes from taskbar via Win32 API. |
| WOVL-03 | 11-02 | Overlay does not appear in Alt+Tab or taskbar (WS_EX_TOOLWINDOW) | SATISFIED (code) | lib.rs lines 137-154: WS_EX_TOOLWINDOW applied directly via GetWindowLongPtrW/SetWindowLongPtrW, removing WS_EX_APPWINDOW simultaneously. |
| WOVL-04 | 11-03 | Previous window HWND captured before overlay shows for focus restoration | BLOCKED | HWND capture code exists but is unreachable on Windows (inside `if let Some(pid) = pid` which is always false on Windows). `previous_hwnd` is never populated. |
| WOVL-05 | 11-03 | Focus returns to previous terminal window on overlay dismiss (SetForegroundWindow) | BLOCKED | `restore_focus()` implementation is correct but never called with a valid HWND because `previous_hwnd` is always None. Depends on WOVL-04 being fixed. |
| WOVL-06 | 11-03 | Ctrl+Shift+K default hotkey triggers overlay system-wide (configurable) | SATISFIED | lib.rs lines 165-168: `#[cfg(target_os = "macos")] let default_hotkey = "Super+K"; #[cfg(not(target_os = "macos"))] let default_hotkey = "Ctrl+Shift+K";`. Used in register_hotkey call at line 171. |
| WOVL-07 | 11-02 | Escape dismisses overlay without executing | SATISFIED (frontend handles, backend verified) | Summary states Escape dismiss works via frontend Overlay.tsx calling hide_overlay IPC. hide_overlay Windows path is correctly implemented in window.rs. |

**Coverage:** 7/9 requirements satisfied (WOVL-04 and WOVL-05 blocked)

No orphaned requirements: all 9 phase-11 requirements (WOVL-01 through WOVL-07, WBLD-01, WBLD-02) are claimed in the three plans and accounted for in this report.

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src-tauri/src/commands/paste.rs` | 140 | `// TODO(Phase 13): Implement Windows clipboard write via win32 API or arboard crate` | INFO | Expected -- this is a known deferred stub for Phase 13. Does not block Phase 11 goal. |
| `src-tauri/src/commands/hotkey.rs` | 250-263 | Windows HWND capture inside `if let Some(pid) = pid` -- unreachable on Windows | BLOCKER | Blocks WOVL-04 and WOVL-05. The block `#[cfg(target_os = "windows")]` at line 251 is inside the `if let Some(pid) = pid` guard at line 243. Since `get_frontmost_pid()` always returns `None` on non-macOS (line 55-58), this block never runs on Windows. |

---

## Human Verification Required

### 1. Acrylic Vibrancy Visual Check

**Test:** Build and launch the app on Windows 10 1903+ or Windows 11 hardware. Press Ctrl+Shift+K.
**Expected:** The overlay panel appears with a dark frosted glass (Acrylic blur) background matching the macOS HudWindow appearance.
**Why human:** Visual effect quality cannot be verified from code. Acrylic requires Windows 10 1903+ -- earlier versions will crash at `.expect("Acrylic vibrancy requires Windows 10 1903+")`.

### 2. Alt+Tab and Taskbar Exclusion

**Test:** On Windows, open the overlay with Ctrl+Shift+K, then press Alt+Tab.
**Expected:** The CMD+K overlay does NOT appear in the Alt+Tab switcher or the Windows taskbar.
**Why human:** WS_EX_TOOLWINDOW behavior requires runtime verification on actual Windows hardware.

### 3. Focus Restoration (After Gap Fix)

**Test:** After the HWND capture bug is fixed, press Ctrl+Shift+K while a terminal is focused, then press Escape.
**Expected:** Focus returns to the terminal automatically without the user clicking.
**Why human:** Focus management behavior depends on Windows window manager state, which varies by Windows version and application.

---

## Gaps Summary

Two gaps block the phase goal:

**Gap 1 (WOVL-04): HWND Capture is Dead Code on Windows**

In `src-tauri/src/commands/hotkey.rs`, the Windows HWND capture block (lines 250-263) is correctly written but placed inside an `if let Some(pid) = pid` guard (line 243). The `pid` variable comes from `get_frontmost_pid()` which returns `None` on all non-macOS platforms (the stub at lines 55-58). As a result, on a real Windows build, `pid` is always `None`, the `if let Some(pid) = pid` never matches, and the entire HWND capture block -- including `get_foreground_hwnd()` and the `previous_hwnd` state write -- is never reached.

The fix is straightforward: move the `#[cfg(target_os = "windows")] { let hwnd = get_foreground_hwnd(); ... }` block OUT of the `if let Some(pid) = pid` branch and place it at the outer `if !is_currently_visible` scope level, alongside the `let pid = get_frontmost_pid()` call.

**Gap 2 (WOVL-05): Focus Restoration Silent No-Op on Windows**

Because `previous_hwnd` is never populated (Gap 1), the focus restoration logic in `hide_overlay` at `src-tauri/src/commands/window.rs` lines 104-118 always reads `None` from `state.previous_hwnd`. The `if let Some(hwnd) = prev_hwnd` check never triggers, so `restore_focus()` is never called. The implementation of `restore_focus()` itself (AttachThreadInput + SetForegroundWindow + IsWindow + AllowSetForegroundWindow fallback) is complete and correct -- it just needs the HWND data to be fed into it.

**Root Cause:** Both gaps share a single root cause: the Windows HWND capture is gated behind a macOS-specific control flow path (`if let Some(pid) = pid` where pid is always None on Windows). Fixing Gap 1 will resolve both.

---

*Verified: 2026-03-02T16:30:00Z*
*Verifier: Claude (gsd-verifier)*
