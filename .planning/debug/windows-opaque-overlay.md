---
status: awaiting_human_verify
trigger: "Windows overlay window background is opaque instead of transparent"
created: 2026-03-03T01:12:00Z
updated: 2026-03-03T01:25:00Z
---

## Current Focus

hypothesis: CONFIRMED -- Two independent causes create opaque window
test: cargo check passes, tsc --noEmit passes
expecting: user launches app and overlay floats transparently
next_action: awaiting human verification

## Symptoms

expected: Window background fully transparent; only rounded overlay panel visible floating above desktop
actual: Entire Tauri window is a dark opaque rectangle; overlay panel visible inside it with border
errors: none (visual bug)
reproduction: Launch app on Windows, trigger Ctrl+K overlay
started: Windows port (always broken on Windows; works on macOS)

## Eliminated

## Evidence

- timestamp: 2026-03-03T01:12:00Z
  checked: Screenshot
  found: Dark rectangle fills entire window area; overlay panel with border visible inside
  implication: Window-level effect (acrylic) is painting entire window dark

- timestamp: 2026-03-03T01:12:00Z
  checked: lib.rs Windows setup block
  found: apply_acrylic(&window, Some((18, 18, 18, 125))) applied at window level
  implication: Acrylic vibrancy paints ENTIRE window background with RGBA(18,18,18,125) frosted tint

- timestamp: 2026-03-03T01:12:00Z
  checked: Overlay.tsx CSS classes
  found: bg-black/60 on overlay div provides its own dark background
  implication: Panel already has its own dark translucent background; acrylic is redundant

- timestamp: 2026-03-03T01:12:00Z
  checked: styles.css body rule
  found: body has @apply bg-background which in :root is oklch(1 0 0) (white, opaque)
  implication: CSS body bg-background overrides inline transparent style; may conflict

- timestamp: 2026-03-03T01:18:00Z
  checked: macOS implementation comparison
  found: macOS uses apply_vibrancy with HudWindow material + NSPanel with corner radius 12.0. The vibrancy applies to the panel via NSVisualEffectView, which macOS clips to the corner radius. On Windows, apply_acrylic uses DWM composition which paints the ENTIRE window surface.
  implication: Fundamental platform difference -- macOS vibrancy is clipped to panel shape, Windows acrylic is full-window

- timestamp: 2026-03-03T01:19:00Z
  checked: Tauri issues #12804, #12437, #10064
  found: CSS backdrop-filter: blur() does NOT work correctly on Tauri transparent windows on Windows (WebView2 limitation). It fails to blur behind-window content.
  implication: Cannot use CSS backdrop-blur as a 1:1 replacement for behind-window blur; must use opaque dark background instead

- timestamp: 2026-03-03T01:20:00Z
  checked: styles.css @layer base rule
  found: body { @apply bg-background text-foreground } sets body background to oklch(1 0 0) = opaque white in light mode
  implication: Even without acrylic, the body has an opaque white background that prevents window transparency

- timestamp: 2026-03-03T01:24:00Z
  checked: cargo check and tsc --noEmit
  found: Both compile cleanly (0 errors, pre-existing warnings only)
  implication: Fix is syntactically valid

## Resolution

root_cause: |
  TWO independent causes create the opaque window on Windows:

  1. RUST: apply_acrylic(&window, Some((18, 18, 18, 125))) paints the ENTIRE window surface
     with a dark frosted glass effect at the DWM (compositor) level. Unlike macOS's
     NSVisualEffectView which is clipped to the panel's corner radius, Windows Acrylic
     applies uniformly to the full window rectangle. This creates the dark opaque background
     visible in the screenshot.

  2. CSS: In styles.css, the @layer base rule `body { @apply bg-background }` sets body
     background to oklch(1 0 0) (opaque white). This overrides the inline
     style={{ background: "transparent" }} on the root div. Even if acrylic were removed,
     the body background would still be opaque.

fix: |
  1. src-tauri/src/lib.rs: Removed the apply_acrylic() call entirely on Windows. Kept
     DwmExtendFrameIntoClientArea (enables transparency) and all other Window setup code.
     Added detailed comment explaining WHY acrylic is not used.

  2. src/styles.css: Changed body rule from `@apply bg-background text-foreground` to
     `@apply text-foreground` plus `background: transparent !important`. Added comment
     explaining the constraint.

  The Overlay div's existing bg-black/60 class provides the dark translucent panel appearance.
  No changes needed to Overlay.tsx.

verification: |
  - cargo check: PASS (0 errors)
  - tsc --noEmit: PASS (0 errors)
  - Runtime verification: PENDING (user must launch app and visually confirm)

files_changed:
  - src-tauri/src/lib.rs
  - src/styles.css
