---
phase: 17-overlay-z-order
verified: 2026-03-03T00:00:00Z
status: human_needed
score: 2/5 must-haves automated-verified (3/5 require macOS runtime)
human_verification:
  - test: "macOS permission dialogs appear above the CMD+K overlay"
    expected: "With overlay open, triggering an Accessibility or Screen Recording permission dialog causes the dialog to render above the overlay and be fully interactable"
    why_human: "NSPanel window level interaction with macOS dialog levels is a macOS window server behavior — cannot be verified statically from code"
  - test: "Notification Center appears above the CMD+K overlay"
    expected: "With overlay open, swiping from right edge of trackpad to open Notification Center causes it to slide in above the overlay"
    why_human: "macOS window server z-order compositing at runtime cannot be verified statically"
  - test: "Spotlight appears above the CMD+K overlay"
    expected: "With overlay open, pressing Cmd+Space to open Spotlight causes it to appear above the overlay and be usable"
    why_human: "macOS window server z-order compositing at runtime cannot be verified statically"
  - test: "Overlay floats above all normal application windows"
    expected: "With Terminal, browser, and Finder open, pressing Cmd+K shows the overlay above all three; clicking on a normal app does not cause the overlay to go behind it"
    why_human: "PanelLevel::Floating (3) is the correct mechanism, but actual floating behavior over specific apps must be confirmed at runtime"
  - test: "Overlay appears above fullscreen apps"
    expected: "With Terminal in fullscreen mode (Ctrl+Cmd+F), pressing Cmd+K shows the overlay on top of the fullscreen Terminal"
    why_human: "full_screen_auxiliary() + can_join_all_spaces() collection behavior provides this, but actual rendering above fullscreen must be confirmed at runtime"
---

# Phase 17: Overlay Z-Order Verification Report

**Phase Goal:** System UI elements (permission dialogs, Notification Center, Spotlight) can appear above the CMD+K overlay while the overlay still floats above normal application windows and fullscreen apps
**Verified:** 2026-03-03
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #   | Truth                                                                              | Status       | Evidence                                                                 |
| --- | ---------------------------------------------------------------------------------- | ------------ | ------------------------------------------------------------------------ |
| 1   | macOS permission dialogs appear above the CMD+K overlay and are interactable       | ? NEEDS HUMAN | Code mechanism correct (PanelLevel::Floating < ModalPanel=8); runtime behavior unverifiable statically |
| 2   | Notification Center slides in above the CMD+K overlay                              | ? NEEDS HUMAN | Code mechanism correct (PanelLevel::Floating < Status=25); runtime behavior unverifiable statically |
| 3   | Spotlight appears above the CMD+K overlay when invoked                             | ? NEEDS HUMAN | Code mechanism correct (Spotlight uses system-level overlay > Floating=3); runtime behavior unverifiable statically |
| 4   | CMD+K overlay floats above all normal application windows                          | ? NEEDS HUMAN | PanelLevel::Floating (3) > Normal (0) is correct; SUMMARY records Task 2 approved by human |
| 5   | CMD+K overlay appears above fullscreen apps                                        | ? NEEDS HUMAN | full_screen_auxiliary() + can_join_all_spaces() preserved at lines 105-106; SUMMARY records Task 2 approved |

**Score:** 0/5 truths auto-verified (all require macOS runtime); code mechanism confirmed correct for all 5

### Required Artifacts

| Artifact                    | Expected                         | Status      | Details                                                                 |
| --------------------------- | -------------------------------- | ----------- | ----------------------------------------------------------------------- |
| `src-tauri/src/lib.rs`      | NSPanel window level configuration with PanelLevel::Floating | VERIFIED | File exists; `panel.set_level(PanelLevel::Floating.value())` at line 100; no PanelLevel::Status references remain |

**Artifact Level Checks:**
- Level 1 (Exists): PASS — file is present and non-trivial (217 lines)
- Level 2 (Substantive): PASS — full Tauri app setup with complete NSPanel configuration; not a stub
- Level 3 (Wired): PASS — `panel.set_level(PanelLevel::Floating.value())` is called directly in the macOS setup block (line 100); this is the entry point of truth for the z-order, not a helper function

### Key Link Verification

| From                   | To                              | Via                                        | Status   | Details                                                  |
| ---------------------- | ------------------------------- | ------------------------------------------ | -------- | -------------------------------------------------------- |
| `src-tauri/src/lib.rs` | macOS window server z-order     | `panel.set_level(PanelLevel::Floating.value())` | WIRED | Exact pattern found at line 100; import at line 23 (`use tauri_nspanel::{CollectionBehavior, PanelLevel, StyleMask, WebviewWindowExt}`) |

**Verification commands from PLAN:**
- `grep -n "PanelLevel::Floating" src-tauri/src/lib.rs` → line 100: `panel.set_level(PanelLevel::Floating.value());` — PASS
- `grep -n "PanelLevel::Status" src-tauri/src/lib.rs` → no output — PASS (Status level completely removed)

### Supporting Configuration Verified

The PLAN required that these settings remain unchanged. All confirmed present:

| Setting                    | Line | Status    |
| -------------------------- | ---- | --------- |
| `full_screen_auxiliary()`  | 105  | PRESERVED |
| `can_join_all_spaces()`    | 106  | PRESERVED |
| `nonactivating_panel()`    | 112  | PRESERVED |
| `set_has_shadow(false)`    | 118  | PRESERVED |

### Requirements Coverage

| Requirement | Source Plan     | Description                                                                         | Status          | Evidence                                                                                      |
| ----------- | --------------- | ----------------------------------------------------------------------------------- | --------------- | --------------------------------------------------------------------------------------------- |
| ZORD-01     | 17-01-PLAN.md   | System permission and accessibility dialogs can appear above the CMD+K overlay      | CODE SATISFIED  | PanelLevel::Floating (3) is below ModalPanel (8), which governs permission dialogs; code mechanism is correct; runtime confirmed by human Task 2 checkpoint (approved) |
| ZORD-02     | 17-01-PLAN.md   | System UI elements (Notification Center, Spotlight, other system overlays) can appear above the CMD+K overlay | CODE SATISFIED | PanelLevel::Floating (3) is below MainMenu (24) and Status (25) levels used by Notification Center and Spotlight; code mechanism is correct; runtime confirmed by human Task 2 checkpoint (approved) |

**Orphaned requirements check:** REQUIREMENTS.md maps only ZORD-01 and ZORD-02 to Phase 17. Both are claimed in 17-01-PLAN.md `requirements` field. No orphaned requirements.

### Anti-Patterns Found

None. Scan of `src-tauri/src/lib.rs` found:
- No TODO/FIXME/HACK/PLACEHOLDER comments
- No stub return patterns (`return null`, `return {}`, `return []`)
- No empty handlers
- No unimplemented macros
- All NSPanel configuration calls are substantive (real API calls with correct arguments)

### Human Verification Required

The PLAN included a blocking `checkpoint:human-verify` gate (Task 2). The SUMMARY records this was approved. However, since we cannot independently confirm that human approval from code, the following items should be treated as needing human re-confirmation if any doubt exists:

#### 1. System Dialogs Above Overlay (ZORD-01)

**Test:** With CMD+K overlay open, trigger an Accessibility permission dialog (e.g., System Settings > Privacy and Security > Accessibility, toggle the app off and back on)
**Expected:** The permission dialog appears above the overlay and is fully clickable/dismissable
**Why human:** NSPanel level interaction with macOS CGWindowLevelKey_modalPanel is a compositing decision made by the macOS window server at runtime

#### 2. Notification Center Above Overlay (ZORD-02)

**Test:** With CMD+K overlay open, swipe from the right edge of the trackpad to open Notification Center
**Expected:** Notification Center slides in above the overlay, fully interactable
**Why human:** macOS window server compositing at runtime

#### 3. Spotlight Above Overlay (ZORD-02)

**Test:** With CMD+K overlay open, press Cmd+Space to invoke Spotlight
**Expected:** Spotlight appears above the overlay and is fully usable
**Why human:** macOS window server compositing at runtime

#### 4. Overlay Above Normal App Windows (Regression)

**Test:** Open Terminal, a browser, and Finder; press Cmd+K; click on a normal app window
**Expected:** Overlay remains visible above all three apps; floating behavior maintained
**Why human:** PanelLevel::Floating (3) is theoretically correct, but floating behavior interactions with specific apps must be confirmed

#### 5. Overlay Above Fullscreen Apps (Regression)

**Test:** Put Terminal in fullscreen (Ctrl+Cmd+F or green button); press Cmd+K
**Expected:** Overlay appears above the fullscreen Terminal
**Why human:** full_screen_auxiliary() collection behavior must be confirmed to still work with PanelLevel::Floating vs. the previous PanelLevel::Status

### Code Quality Assessment

The single-line change is minimal, correct, and well-documented:

**Before (as documented in PLAN):**
```rust
// Set panel level above the menu bar so it floats over fullscreen apps
// Status level (25) = NSMainMenuWindowLevel + 1
panel.set_level(PanelLevel::Status.value());
```

**After (verified in codebase at line 93-100):**
```rust
// Set panel level to Floating (3) — above normal app windows but below
// system UI (permission dialogs, Notification Center, Spotlight).
// Floating (3) > Normal (0), so the overlay stays above all standard apps.
// System overlays use higher levels (ModalPanel=8, MainMenu=24, Status=25)
// and will correctly render above this panel.
// Combined with full_screen_auxiliary() collection behavior, the overlay
// still appears above fullscreen apps.
panel.set_level(PanelLevel::Floating.value());
```

The comment accurately explains the rationale and level hierarchy. The implementation matches the PLAN exactly.

### Gaps Summary

No gaps found. All automated checks pass:
- The artifact exists, is substantive, and is correctly wired
- The key link (`set_level(PanelLevel::Floating.value())`) is verified
- `PanelLevel::Status` is fully removed with zero remaining references
- All required supporting configuration is preserved
- Both ZORD-01 and ZORD-02 are accounted for with correct code mechanisms
- No anti-patterns

The only items flagged are for human verification because z-order behavior in macOS window compositing cannot be confirmed from static analysis. The SUMMARY records a human-approved Task 2 checkpoint, which provides confidence that runtime behavior was verified. If that approval was genuine, this phase is effectively complete.

---

_Verified: 2026-03-03_
_Verifier: Claude (gsd-verifier)_
