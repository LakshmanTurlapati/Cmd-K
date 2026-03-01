# Phase 7: Fix Accessibility Permission Detection and Move Accessibility Step to Second-to-Last in Onboarding - Research

**Researched:** 2026-02-26
**Domain:** macOS Accessibility API (AX), TCC permission system, React/Tauri onboarding UI
**Confidence:** HIGH (for onboarding reorder — full codebase read), MEDIUM (for AX probe fallback — based on API behavior analysis + community sources)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Detection Fix**
- AXIsProcessTrusted/AXIsProcessTrustedWithOptions returns false in production builds even when the app is listed and toggled ON in System Settings > Accessibility
- Works correctly in dev mode (cargo tauri dev), fails in unsigned production builds
- Re-adding the app to the Accessibility list does not help
- Fix approach: Research and fix the root cause (likely unsigned build identity mismatch with macOS trust database) AND add a fallback detection method regardless
- Fallback: Attempt an actual AX API call as a silent probe -- if the AX call succeeds, permission is granted even if AXIsProcessTrusted reports false
- Fallback probe should be completely silent -- no UI indication that a fallback is being used

**Skip and Degraded UX**
- Users CAN skip the accessibility step during onboarding
- When skipped, a compact warning badge appears in the overlay (not a full banner -- something concise like a small warning icon with brief text)
- Badge behavior: tooltip on hover explaining what's limited and how to fix, click opens System Settings directly
- Badge auto-disappears: periodically check permission status in the background and hide the badge as soon as accessibility is granted (no app restart needed)

**Re-check Flow (During Onboarding)**
- After user clicks "Open System Settings", auto-poll every 1-2 seconds to detect when permission is granted
- Polling runs indefinitely while the accessibility step is visible (user may take time in System Settings)
- Polling uses dual approach: try fixed AXIsProcessTrusted first, if false try fallback probe
- On detection of grant: show green checkmark / "Granted!" for ~1 second, then auto-advance to next step

**Onboarding Reorder**
- New step order: API Key (step 0) -> Model (step 1) -> Accessibility (step 2) -> Done (step 3)
- Step labels update accordingly

### Claude's Discretion

- Exact polling interval (1-2 second range)
- Specific fallback AX API call to use as probe
- Warning badge visual design (icon choice, text, positioning in overlay)
- Technical approach to fixing the unsigned build detection issue

### Deferred Ideas (OUT OF SCOPE)

None -- discussion stayed within phase scope
</user_constraints>

---

## Summary

This phase has two parts: (1) fixing the macOS accessibility permission detection that consistently returns false for unsigned production builds, and (2) reordering the onboarding wizard steps so accessibility is second-to-last.

The root cause of false-negative permission detection is well-understood: macOS TCC (Transparency, Consent, and Control) stores app permissions keyed by code-signing identity. Unsigned builds produced by `cargo tauri build` without a Developer ID certificate use ad-hoc signing, which generates a new identity on every build. TCC therefore treats each build as a new app and does not recognize previously-granted permissions. The fix has two independent parts: attempt to coerce macOS to find the correct TCC entry (by calling AXIsProcessTrustedWithOptions with prompt: false), AND implement a live AX API probe that empirically tests whether the permission actually works regardless of what the flag reports.

The onboarding reorder is a straightforward surgical change: swap step indices in OnboardingWizard.tsx, update step labels, adjust the effectiveStep advancement logic in App.tsx's startup check, and remove the skip button from the accessibility step (it has its own "Continue" button and users can still advance).

The existing codebase already has the AX FFI infrastructure (ax_reader.rs, accessibility_sys crate) to implement an elegant probe. The probe should call AXUIElementCreateApplication with a known safe PID (the current process itself, or the SystemUIServer), then attempt AXUIElementCopyAttributeValue. If the call returns kAXErrorSuccess (0), permission is actually working even if AXIsProcessTrusted lied.

**Primary recommendation:** Implement dual-check as a new Rust function `check_accessibility_permission_dual()` that replaces all call sites for `check_accessibility_permission`. The dual check: (1) call AXIsProcessTrusted(), return true immediately if true; (2) call the AX probe on our own PID (std::process::id()), return true if probe succeeds (kAXErrorSuccess), false otherwise.

---

## Standard Stack

### Core (already in project -- no new dependencies needed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| accessibility-sys | 0.2 | AXIsProcessTrusted binding | Already in Cargo.toml |
| core-foundation-sys | 0.8 | CFRelease, CFStringRef, etc. | Already in Cargo.toml |
| @tauri-apps/plugin-store | 2 | Persist onboarding step | Already in use |
| @radix-ui/react-tooltip | 1.2.8 | Tooltip for warning badge | Already installed (Phase 05-02) |

### Supporting

No new dependencies required. All work uses existing codebase infrastructure.

**Installation:** No additional packages needed.

---

## Architecture Patterns

### Recommended Project Structure (no changes needed)

The phase modifies existing files only:

```
src-tauri/src/commands/permissions.rs   # Add dual-check probe function
src/components/Onboarding/OnboardingWizard.tsx  # Reorder steps + labels
src/components/Onboarding/StepAccessibility.tsx # Add polling, skip-badge trigger
src/components/Overlay.tsx              # Replace full banner with compact badge
src/App.tsx                             # Fix effectiveStep advancement logic
src/store/index.ts                      # Add accessibilitySkipped state (optional)
```

### Pattern 1: Dual-Check Rust Function

**What:** A single exported Rust function that wraps both AXIsProcessTrusted and an AX probe, returning the logical OR.

**When to use:** Replaces every call site that previously called `check_accessibility_permission` or `request_accessibility_permission`.

**Example (Rust, permissions.rs):**

```rust
/// Dual-check: AXIsProcessTrusted first, then AX probe fallback.
/// Returns true if either check confirms accessibility is granted.
#[tauri::command]
#[cfg(target_os = "macos")]
pub fn check_accessibility_permission() -> bool {
    // Primary check: the standard flag
    let trusted = unsafe { accessibility_sys::AXIsProcessTrusted() };
    if trusted {
        return true;
    }
    // Fallback probe: attempt an actual AX call on our own process.
    // If it succeeds (kAXErrorSuccess = 0), the OS actually allows AX access
    // even though the flag lied (unsigned build identity mismatch in TCC).
    ax_probe_self()
}

#[cfg(target_os = "macos")]
fn ax_probe_self() -> bool {
    use std::ffi::c_void;

    extern "C" {
        fn AXUIElementCreateApplication(pid: i32) -> *const c_void;
        fn AXUIElementCopyAttributeValue(
            element: *const c_void,
            attribute: *const c_void,
            value: *mut *const c_void,
        ) -> i32;
        fn CFRelease(cf: *const c_void);
    }

    // Use our own PID as probe target -- we can always create an element for ourselves.
    // If accessibility is granted, CopyAttributeValue returns 0 (kAXErrorSuccess).
    // If not granted, it returns -25211 (kAXErrorNotTrusted) or -25212 (kAXErrorCannotComplete).
    let pid = std::process::id() as i32;
    unsafe {
        let app_elem = AXUIElementCreateApplication(pid);
        if app_elem.is_null() {
            return false;
        }
        // We need a CFString for the attribute name. Reuse cf_string pattern from ax_reader.rs:
        // Use core_foundation_sys to create the string for "AXRole" (lightweight, always present).
        use core_foundation_sys::string::{CFStringCreateWithCString, kCFStringEncodingUTF8};
        use core_foundation_sys::base::kCFAllocatorDefault;
        use std::ffi::CString;
        let attr_name = CString::new("AXRole").expect("CString ok");
        let attr_cf = CFStringCreateWithCString(kCFAllocatorDefault, attr_name.as_ptr(), kCFStringEncodingUTF8);
        if attr_cf.is_null() {
            CFRelease(app_elem);
            return false;
        }
        let mut value: *const c_void = std::ptr::null();
        let err = AXUIElementCopyAttributeValue(app_elem, attr_cf as *const c_void, &mut value);
        if !value.is_null() {
            CFRelease(value);
        }
        CFRelease(attr_cf as *const c_void);
        CFRelease(app_elem);
        // kAXErrorSuccess = 0. Any non-zero means either permission denied or element has no role (still "success" from permission standpoint).
        // kAXErrorNotTrusted = -25211 means definitely not granted.
        // kAXErrorAttributeUnsupported / kAXErrorNoValue = permission is granted but attribute absent (treat as granted).
        // So: return true if NOT kAXErrorNotTrusted (-25211) AND NOT kAXErrorCannotComplete (-25212 when no permission).
        const AX_ERROR_NOT_TRUSTED: i32 = -25211;
        err != AX_ERROR_NOT_TRUSTED
    }
}
```

**NOTE:** The existing `request_accessibility_permission(prompt: bool)` function can remain for the onboarding "prompt" case (showing the system dialog). The dual-check replaces all background/non-prompting checks.

### Pattern 2: Frontend Polling with useEffect + setInterval

**What:** React useEffect that starts a polling interval when the accessibility step becomes visible, clears on unmount or on grant detection.

**When to use:** Onboarding step 2 (Accessibility), polling after user clicks "Open System Settings".

**Example (TypeScript):**

```typescript
useEffect(() => {
  // Start polling only after user has opened settings (pollingActive state)
  if (!pollingActive) return;

  const intervalId = setInterval(async () => {
    try {
      const result = await invoke<boolean>("check_accessibility_permission");
      if (result) {
        clearInterval(intervalId);
        setGranted(true);
        useOverlayStore.getState().setAccessibilityGranted(true);
        // Show granted state for 1 second, then auto-advance
        setTimeout(() => onNext(), 1000);
      }
    } catch {
      // Silent -- keep polling
    }
  }, 1500); // 1.5s interval (within the 1-2s range at Claude's discretion)

  return () => clearInterval(intervalId);
}, [pollingActive]); // eslint-disable-line react-hooks/exhaustive-deps
```

### Pattern 3: Background Badge Polling in Overlay

**What:** A lightweight `setInterval` that runs in command mode when `!accessibilityGranted`, periodically rechecking permissions and clearing itself on grant.

**When to use:** After onboarding completes with accessibility skipped, keeps checking until granted.

**Example (TypeScript, inside Overlay.tsx or a custom hook):**

```typescript
useEffect(() => {
  if (accessibilityGranted) return; // already granted, no need to poll

  const intervalId = setInterval(async () => {
    try {
      const result = await invoke<boolean>("check_accessibility_permission");
      if (result) {
        useOverlayStore.getState().setAccessibilityGranted(true);
        // Badge will disappear reactively via accessibilityGranted state
      }
    } catch {
      // Silent
    }
  }, 5000); // Poll every 5s in background (less aggressive than onboarding)

  return () => clearInterval(intervalId);
}, [accessibilityGranted]);
```

### Pattern 4: Compact Warning Badge in Overlay

**What:** Replace the current multi-line text block (`!accessibilityGranted` banner) with a small inline badge using Radix Tooltip (already installed).

**Current code to replace in Overlay.tsx (lines 83-114):**

```tsx
{mode === "command" && !accessibilityGranted && !isDetecting && (
  <div className="text-xs font-mono flex flex-col gap-1">
    <span className="text-amber-400/80">Accessibility permission not detected.</span>
    ...multi-line banner...
  </div>
)}
```

**New compact badge approach:**

```tsx
import * as Tooltip from "@radix-ui/react-tooltip";
import { ShieldAlert } from "lucide-react";

// Inside command mode section, replacing the full banner:
{mode === "command" && !accessibilityGranted && !isDetecting && (
  <Tooltip.Provider delayDuration={300}>
    <Tooltip.Root>
      <Tooltip.Trigger asChild>
        <button
          type="button"
          onClick={() => invoke("open_accessibility_settings")}
          className="flex items-center gap-1 text-amber-400/70 hover:text-amber-400 transition-colors cursor-default bg-transparent border-none p-0"
        >
          <ShieldAlert size={12} />
          <span className="text-[11px] font-mono">No AX access</span>
        </button>
      </Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content
          className="bg-black/90 text-white/70 text-xs px-2 py-1 rounded border border-white/10 max-w-[200px]"
          sideOffset={4}
        >
          Terminal context and paste require Accessibility. Click to open System Settings.
          <Tooltip.Arrow className="fill-black/90" />
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  </Tooltip.Provider>
)}
```

### Pattern 5: Onboarding Step Reorder

**What:** Swap step component render order in OnboardingWizard.tsx and update the step label array. Update effectiveStep logic in App.tsx.

**Current step mapping (OnboardingWizard.tsx lines 44, 115-118):**

```typescript
// OLD
const stepLabels = ["Accessibility", "API Key", "Model", "Done"];
// ...
{onboardingStep === 0 && <StepAccessibility onNext={handleNext} />}
{onboardingStep === 1 && <StepApiKey onNext={handleNext} />}
{onboardingStep === 2 && <StepModelSelect onNext={handleNext} />}
{onboardingStep === 3 && <StepDone onComplete={handleComplete} />}
```

**New step mapping:**

```typescript
// NEW
const stepLabels = ["API Key", "Model", "Accessibility", "Done"];
// ...
{onboardingStep === 0 && <StepApiKey onNext={handleNext} />}
{onboardingStep === 1 && <StepModelSelect onNext={handleNext} />}
{onboardingStep === 2 && <StepAccessibility onNext={handleNext} />}
{onboardingStep === 3 && <StepDone onComplete={handleComplete} />}
```

**App.tsx effectiveStep logic must also update (lines 78-82):**

```typescript
// OLD: effectiveStep logic was for apikey at step 1
const effectiveStep =
  onboardingStep <= 1
    ? Math.max(onboardingStep, 1)
    : onboardingStep;

// NEW: API key is now step 0, so the advancement logic changes:
// If API key is already present (we're in this branch), skip to at least step 1 (Model)
const effectiveStep =
  onboardingStep <= 0
    ? 1  // skip past API key step since key exists
    : onboardingStep;
```

### Anti-Patterns to Avoid

- **Caching the probe result aggressively:** The probe must be called each time (or at polling interval) because the user may grant permission after launch. Do not cache the AX probe result permanently; cache it only for the duration of an overlay session (re-checked each `show()`).
- **Using AXIsProcessTrustedWithOptions(prompt: true) as the dual-check:** The prompt version triggers a system dialog which is intrusive. The dual-check (probe) should be completely silent. Only use prompt: true in the onboarding step UI explicitly.
- **Making the AX probe too expensive:** Probing with a minimal attribute like "AXRole" on our own PID is fast (sub-millisecond). Do not walk AX trees in the probe.
- **Calling the probe on a target app PID:** The probe should use our own PID (`std::process::id()`). Probing another app's AX tree is more expensive and not necessary to detect our own trust status.
- **Forgetting to clear polling intervals:** React `useEffect` cleanup must always `clearInterval`. Orphaned intervals will accumulate across onboarding step changes.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tooltip for badge | Custom CSS tooltip with `::before` pseudo-element | @radix-ui/react-tooltip (already installed v1.2.8) | Already in the codebase from Phase 05-02, handles z-index, portal, accessibility |
| Permission polling | Custom event bus / observable | setInterval + useEffect cleanup | Simplest approach, existing pattern in Phase 03 banner re-check |
| AX string creation | New cf_string helper | Reuse pattern from ax_reader.rs `cf_string_from_str` or inline with core-foundation-sys directly | Already proven in codebase |

**Key insight:** The AX probe uses the exact same FFI patterns already established in `ax_reader.rs`. No new AX infrastructure is needed -- the probe is just `AXUIElementCreateApplication(own_pid)` + `AXUIElementCopyAttributeValue` for "AXRole", both already declared as extern "C" in ax_reader.rs.

---

## Common Pitfalls

### Pitfall 1: AX Probe on Own PID Returns Permission Error Even When Granted

**What goes wrong:** `AXUIElementCopyAttributeValue` for "AXRole" on our own PID may return `kAXErrorCannotComplete` (-25212) even with permission, because some apps don't expose their own AX tree well (e.g., the webview inside Tauri).

**Why it happens:** Tauri's webview (WKWebView) may not implement the AXRole attribute on the application element. The application element itself may not have a role defined.

**How to avoid:** Treat `kAXErrorNotTrusted` (-25211) as the definitive "no permission" code. All other error codes (including -25212 and attribute-absent codes like -25205 `kAXErrorAttributeUnsupported`, -25204 `kAXErrorNoValue`) indicate the OS allowed the call but the attribute wasn't available -- which means permission IS granted. Return `false` ONLY for error code -25211.

**Warning signs:** If probe always returns false even when permission is genuinely granted, the issue is interpreting non-permission errors as permission failures.

### Pitfall 2: Unsigned Build Identity Mismatch is NOT Fully Fixable at Runtime

**What goes wrong:** The CONTEXT.md notes the root cause is "unsigned build identity mismatch." There is no runtime workaround that fixes TCC's reading of `AXIsProcessTrusted` for truly unsigned builds. Apple's TCC database requires a stable code-signing identity.

**Why it happens:** macOS TCC stores the accessibility grant keyed to the code signature. Ad-hoc signed builds (produced by `cargo tauri build` without a Developer ID) get a new ephemeral identity each build. Even if the app is in the Accessibility list in System Settings UI, TCC may not match it to the current build's signature.

**How to avoid:** The CONTEXT.md decision is correct: fix the root cause AND add the fallback. The "root cause fix" means investigating whether `cargo tauri build --target aarch64-apple-darwin` with ad-hoc signing can be coaxed into stable identity. In practice, the fallback AX probe is the reliable path because it bypasses TCC lookup entirely and tests actual AX API access.

**Confidence:** MEDIUM -- confirmed by Tauri issue #11085 (FabianLars: "Permissions on macOS are bound to the signing identity/certificate"). The probe fallback is the correct path for unsigned builds.

**Warning signs:** If the fix is ONLY changing the signing approach (no fallback), it will break again on the next build. The fallback must be permanent.

### Pitfall 3: Polling in Onboarding Leaks After Step Advance

**What goes wrong:** If the polling interval in StepAccessibility is not properly cleared when `onNext()` fires (auto-advance), the interval fires after the component is unmounted and invokes the Rust command unnecessarily.

**Why it happens:** React functional component cleanup in `useEffect` return function only runs when the component unmounts OR when the effect dependencies change. If the interval fires at the exact moment the component is unmounting, it could call `setGranted` on an unmounted component.

**How to avoid:** Use a `mounted` ref pattern:
```typescript
const mountedRef = useRef(true);
useEffect(() => {
  return () => { mountedRef.current = false; };
}, []);
// In interval callback: if (!mountedRef.current) return;
```

**Warning signs:** React "Can't perform state update on unmounted component" warning in console (though React 18 suppresses this, it's still a logic error).

### Pitfall 4: Onboarding Step Reorder Breaks Resume-from-Step Logic

**What goes wrong:** App.tsx has an `effectiveStep` calculation that assumes API key is at step 1. After reorder (API key at step 0), a user who was at step 1 during a previous interrupted onboarding would be fast-forwarded incorrectly.

**Why it happens:** The `effectiveStep` code explicitly compares `onboardingStep <= 1` to detect if the user was stuck before the API key step, then advances them past it. This index is now wrong.

**How to avoid:** Update the effectiveStep logic to use step 0 (new API key position): if API key exists and `onboardingStep <= 0`, advance to `1`. See Pattern 5 above.

**Warning signs:** User resumes onboarding from wrong step, or is advanced to wrong step on second launch with existing key.

### Pitfall 5: stepLabels Array Mismatch with Step Content Rendering

**What goes wrong:** The `stepLabels` array in OnboardingWizard.tsx is only used for the stepper display. If it is updated but the content rendering is not (or vice versa), the label shown does not match the displayed step.

**How to avoid:** Update both `stepLabels` AND the `onboardingStep === N` render conditions in the same commit/edit.

### Pitfall 6: Skip Button Logic After Reorder

**What goes wrong:** The current skip button renders for `onboardingStep < 3`. This remains correct after reorder (steps 0, 1, 2 all have skip, step 3 = Done doesn't). But the accessibility step (now step 2) ALSO has its own "Continue" button (when `granted !== null`) which serves as the skip mechanism. The step-level skip link is therefore redundant for step 2.

**How to avoid:** The skip link at `onboardingStep < 3` can remain as-is since it covers API Key and Model naturally. For accessibility (step 2), the "Continue" button in StepAccessibility already provides the skip path. No special handling needed unless the user wants to remove the redundant "Skip this step" link for step 2.

---

## Code Examples

Verified patterns from official sources and codebase inspection:

### AX Error Code Reference

```rust
// From ApplicationServices framework (stable public API)
const AX_ERROR_SUCCESS: i32 = 0;           // kAXErrorSuccess
const AX_ERROR_FAILURE: i32 = -25200;       // kAXErrorFailure
const AX_ERROR_NOT_TRUSTED: i32 = -25211;  // kAXErrorNotTrusted -- definitive "no permission"
const AX_ERROR_CANNOT_COMPLETE: i32 = -25212; // kAXErrorCannotComplete -- not a permission error
const AX_ERROR_ATTRIBUTE_UNSUPPORTED: i32 = -25205; // attribute absent -- not a permission error
```

### Checking Own PID

```rust
// std::process::id() returns u32; AXUIElementCreateApplication takes i32.
let pid = std::process::id() as i32;
```

### Radix Tooltip Pattern (from Phase 05-02 pattern, already verified)

```tsx
import * as Tooltip from "@radix-ui/react-tooltip";

<Tooltip.Provider delayDuration={300}>
  <Tooltip.Root>
    <Tooltip.Trigger asChild>
      <button ...>badge content</button>
    </Tooltip.Trigger>
    <Tooltip.Portal>
      <Tooltip.Content sideOffset={4} className="...">
        tooltip text
        <Tooltip.Arrow />
      </Tooltip.Content>
    </Tooltip.Portal>
  </Tooltip.Root>
</Tooltip.Provider>
```

### Current Accessibility Check Call Sites (must ALL be updated to dual-check)

| File | Line | Current call | Action |
|------|------|--------------|--------|
| `src/store/index.ts` | ~229 | `invoke<boolean>("check_accessibility_permission")` in `show()` | Uses dual-check automatically if Rust impl updated |
| `src/components/Overlay.tsx` | ~107 | `invoke<boolean>("check_accessibility_permission")` in "Check Again" button | Uses dual-check automatically |
| `src/App.tsx` | ~94 | `invoke<boolean>("check_accessibility_permission")` in startup check | Uses dual-check automatically |
| `src/components/Onboarding/StepAccessibility.tsx` | ~17 | `invoke<boolean>("request_accessibility_permission", { prompt: true })` | Keep prompt:true for onboarding; add polling that calls `check_accessibility_permission` (no prompt) |

**Key insight:** Since the Rust function name `check_accessibility_permission` is being updated in-place to include the dual-check probe, all existing frontend call sites get the fix automatically. No frontend call sites need to change their invoke name.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| AXIsProcessTrusted only | Dual-check: AXIsProcessTrusted + AX probe fallback | This phase | Handles unsigned build false negatives |
| Full multi-line accessibility banner in Overlay | Compact badge with Radix tooltip | This phase | Less intrusive, concise UX per user decision |
| Accessibility step first (step 0) | Accessibility step second-to-last (step 2) | This phase | API key + model configured before AX step |
| Manual "Check Again" button only | Auto-polling every ~1.5s after "Open System Settings" | This phase | Better UX, detects grant without user action |

**Deprecated/outdated:**
- `request_accessibility_permission(prompt: true)` as the background check mechanism: should only be used when actually prompting user, not for background checks. Background checks use `check_accessibility_permission` (no prompt, no dialog).

---

## Open Questions

1. **Which AX attribute gives the most reliable probe result on our own PID?**
   - What we know: "AXRole" is a standard attribute on most AX elements. For our own app, it may return kAXErrorAttributeUnsupported since a webview app may not declare a role.
   - What's unclear: Whether probing our own PID's AXFocusedWindow vs the application element gives a better signal.
   - Recommendation: Try `AXRole` on the application element first. If probe is unreliable, try `AXFocusedUIElement` instead (more likely to succeed on a webview that has key focus). The critical test is that the code returns `kAXErrorNotTrusted` when NOT granted and anything-else when granted.

2. **Background badge polling interval in command mode**
   - What we know: User wants badge to auto-disappear without restart. 5-second polling in command mode is reasonable.
   - What's unclear: Whether polling should only run when the overlay is visible, or also in background while overlay is hidden.
   - Recommendation: Poll only while the badge is visible (i.e., `!accessibilityGranted` state is true AND overlay is visible). This avoids unnecessary IPC when overlay is hidden. The existing `show()` action already re-checks on each open.

3. **Whether "Skip this step" link should be removed from accessibility step (step 2)**
   - What we know: StepAccessibility already has a "Continue" button that serves as skip. The outer wizard also shows "Skip this step" below.
   - What's unclear: User preference on redundant skip affordance.
   - Recommendation: Keep the outer "Skip this step" link for consistency with steps 0 and 1. The user can decide to remove it during planning if they want.

---

## Sources

### Primary (HIGH confidence)

- Codebase direct read: `src-tauri/src/commands/permissions.rs` -- existing AXIsProcessTrusted usage
- Codebase direct read: `src-tauri/src/terminal/ax_reader.rs` -- all AX FFI patterns including extern "C" declarations, CFRelease, CFStringCreateWithCString, kAXErrorSuccess
- Codebase direct read: `src/components/Onboarding/OnboardingWizard.tsx` -- step ordering, labels, render conditions
- Codebase direct read: `src/App.tsx` -- effectiveStep logic, startup check call sites
- Codebase direct read: `src/store/index.ts` -- all frontend call sites for check_accessibility_permission
- Codebase direct read: `src/components/Overlay.tsx` -- existing accessibility banner code to replace
- Codebase direct read: `src-tauri/Cargo.toml` -- confirms accessibility-sys 0.2 and core-foundation-sys 0.8 already present

### Secondary (MEDIUM confidence)

- [Tauri issue #11085](https://github.com/tauri-apps/tauri/issues/11085): Confirmed root cause -- "Permissions on macOS are bound to the signing identity/certificate" (FabianLars, Tauri team). Ad-hoc signing causes TCC permission loss on rebuild.
- [Apple Developer Forums thread 794253](https://developer.apple.com/forums/thread/794253): AXIsProcessTrusted can return true while AX calls fail (or vice versa). Error code `kAXErrorNotTrusted` (-25211) is the authoritative "not permitted" code.
- [Accessibility Permission in macOS (jano.dev, 2025)](https://jano.dev/apple/macos/swift/2025/01/08/Accessibility-Permission.html): Standard permission detection patterns on macOS Sequoia.

### Tertiary (LOW confidence)

- MacRumors forum: unsigned/self-signed app TCC behavior discussed by community, consistent with Tauri team's statement but not from Apple official source.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in Cargo.toml, no new dependencies
- Architecture: HIGH for onboarding reorder (pure code surgery with full codebase read), MEDIUM for AX probe (behavioral assumptions about error codes, need empirical validation)
- Pitfalls: HIGH for onboarding (derived from codebase), MEDIUM for AX probe (derived from error code semantics + community sources)

**Research date:** 2026-02-26
**Valid until:** 2026-05-26 (stable macOS APIs, 90-day validity estimate)
