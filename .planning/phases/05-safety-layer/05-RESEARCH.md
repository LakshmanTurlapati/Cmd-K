# Phase 5: Safety Layer - Research

**Researched:** 2026-02-23
**Domain:** Regex pattern matching (Rust), React badge UI, Tooltip with async loading, Zustand state, xAI API secondary call
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Warning presentation
- Red "Destructive" badge positioned on the right side of the results header, next to the terminal detection badge
- Badge only -- no changes to command text appearance (no tint, no background change)
- Badge appears after command generation is complete (not during streaming)
- Subtle fade-in animation (~200ms) when badge appears
- Badge is dismissable -- user can click it to hide for that command
- Tooltip on hover shows AI-generated plain-language explanation of what the command does

#### Confirmation flow
- No confirmation required -- copy shortcut works the same for destructive and non-destructive commands
- No extra behavior after copying a destructive command (no toast, no flash)
- Badge serves as informational warning only, not a gate
- Update roadmap success criteria: change "User must explicitly confirm" to "Warning badge informs user of destructive commands"

#### Detection scope and sensitivity
- Curated pattern list for detection (not AI-assisted classification)
- Comprehensive list covering:
  - File destruction: rm -rf, rm -r, shred, unlink, rmdir with contents
  - Git force operations: git push --force, git reset --hard, git clean -fd, git branch -D
  - Database mutations: DROP TABLE, DROP DATABASE, TRUNCATE, DELETE without WHERE
  - System/permission changes: chmod 777, chown, sudo rm, mkfs, dd if=, shutdown, reboot
  - Any other clearly destructive patterns identified during research
- On/off toggle only -- no sensitivity levels
- Toggle lives in the Settings panel (not tray menu)

#### Plain-language explanations
- AI-generated per command via embedded system prompt (not pattern-based templates)
- Explanation is command-specific (e.g., "Recursively deletes all files in /home/user/projects")
- Explanation appears in the badge tooltip on hover
- Eager loading: separate API call fires as soon as destructive command is detected
- If user hovers before explanation loads, tooltip shows a spinner, then swaps to text when ready

#### Claude's Discretion
- Exact curated pattern list completeness (researcher can expand during investigation)
- Tooltip styling and spinner design
- API call structure for the explanation request
- Badge dismiss interaction details (click vs X button)

### Deferred Ideas (OUT OF SCOPE)

- Syntax highlighting for generated commands (Atom IDE-style) -- applies to all commands, not safety-specific. Add to backlog as a UI enhancement phase or standalone task.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| AICG-03 | Destructive commands (rm -rf, format, etc.) are flagged with a warning before paste | Pattern detection runs in Rust post-stream; badge rendered in ResultsArea/Overlay footer; secondary xAI call generates plain-language explanation; Zustand stores isDestructive + explanation state |
</phase_requirements>

---

## Summary

Phase 5 is an additive layer on top of the Phase 4 streaming pipeline. Detection happens in pure Rust using compiled regex patterns applied to the final `streamingText` value after streaming completes. No new Tauri commands are strictly needed for detection -- the frontend can call a new lightweight `check_destructive` IPC command and receive a boolean, OR the detection can happen in the frontend itself via a regex list. Given the project already has `regex = "1"` and `once_cell = "1"` in `Cargo.toml`, the Rust approach is zero-cost-dependency and preferred for accuracy and reusability.

The badge UI slots into the existing results header row in `Overlay.tsx` -- the same `div` that currently shows the shell-type badge. The destructive badge renders to the right of the shell badge when `displayMode === "result"` and `isDestructive === true`. Dismissal is tracked per-result via a local boolean in component state (not persisted). The on/off toggle is a new row in `PreferencesTab.tsx` persisted to `settings.json` via `tauri-plugin-store`.

The plain-language explanation requires a second xAI API call. The ideal implementation fires this call from the Rust side immediately after `stream_ai_response` completes (or from the frontend's `submitQuery` success block) using the same Keychain API key, `reqwest`, and a short non-streaming POST. The result is delivered to the frontend via a new `get_destructive_explanation` IPC command or via a small Tauri Channel. Tooltip rendering uses Radix UI's `Tooltip` component (already available via `radix-ui` in `package.json`) with a spinner state while the explanation loads.

**Primary recommendation:** Detect in Rust via a new `check_destructive(command: String) -> bool` command, fetch explanation via a new `get_destructive_explanation(command: String, on_result: Channel<String>) -> Result<(), String>` command, and render badge + tooltip in the existing Overlay footer row using Radix UI Tooltip.

---

## Standard Stack

### Core (already in project -- no new installs needed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `regex` (Rust) | `1` (pinned in Cargo.toml) | Compiled regex for destructive pattern matching | Already a direct dep; `once_cell` lazy static compilation means zero per-call overhead |
| `once_cell` (Rust) | `1` (pinned in Cargo.toml) | Lazy-static `Regex` initialization | Already a direct dep; preferred over `lazy_static` in modern Rust |
| `keyring` (Rust) | `3` (pinned in Cargo.toml) | Read xAI API key for explanation call | Already used in `ai.rs`; same SERVICE/ACCOUNT constants apply |
| `tauri-plugin-http` / `reqwest` | `2` (pinned) | HTTP POST for explanation API call | Already used for streaming; non-streaming call is simpler subset |
| `tauri-plugin-store` (JS) | `2.4.2` | Persist the destructive detection on/off toggle | Already used in `App.tsx` for hotkey + model persistence |
| `radix-ui` (Tooltip) | `1.4.3` | Badge tooltip with accessible hover behavior | Already installed; zero new npm deps |
| `zustand` | `5.0.11` | Store `isDestructive`, `destructiveExplanation`, `destructiveDismissed`, `destructiveDetectionEnabled` | Already the state manager for all overlay state |
| Tailwind CSS | `4.2.0` | Red badge styling, fade-in animation | Already the CSS layer; `text-red-400`, `bg-red-500/20` available |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde_json` (Rust) | `1` | No additional use needed; explanation is a plain string | Already in project |
| `tw-animate-css` | `1.4.0` | Fade-in keyframe for badge appearance | Already imported in `styles.css`; add `@keyframes badge-in` there |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Rust regex detection | Frontend JS regex | JS works but Rust is already set up, patterns are more testable in Rust, and avoids shipping regex bundle to renderer |
| Radix UI Tooltip | Custom tooltip div | Custom tooltip is already the pattern for small labels in this project (shell badge is plain `<span>`), but Radix Tooltip handles hover delay, positioning, and accessibility correctly for this more complex case |
| Second xAI API call for explanation | Embedded in original system prompt | Embedding in original prompt risks command output contamination (the system prompt says output ONLY the command); second call is cleanest separation |

**Installation:** No new packages needed. All dependencies already present.

---

## Architecture Patterns

### Recommended Project Structure

```
src-tauri/src/commands/
├── ai.rs                  (existing -- stream_ai_response)
├── safety.rs              (NEW -- check_destructive, get_destructive_explanation)
└── mod.rs                 (add safety module)

src/components/
├── Overlay.tsx            (MODIFY -- add DestructiveBadge next to shell badge)
├── ResultsArea.tsx        (no changes needed -- badge lives in Overlay footer)
├── DestructiveBadge.tsx   (NEW -- badge + tooltip + dismiss button)
└── Settings/
    └── PreferencesTab.tsx (MODIFY -- add on/off toggle row)

src/store/index.ts         (MODIFY -- add 4 new state fields + 3 new actions)
```

### Pattern 1: Rust Lazy-Static Regex Set

All patterns compiled once at process start, zero allocation on each check call.

```rust
// src-tauri/src/commands/safety.rs
use once_cell::sync::Lazy;
use regex::RegexSet;

static DESTRUCTIVE_PATTERNS: Lazy<RegexSet> = Lazy::new(|| {
    RegexSet::new([
        // File destruction
        r"(?i)\brm\s+(-[a-z]*r[a-z]*f|--recursive.*--force|--force.*--recursive)\b",
        r"(?i)\brm\s+-[a-z]*r\b",          // rm -r (without -f, still destructive)
        r"(?i)\bshred\b",
        r"(?i)\bunlink\b",
        r"(?i)\brmdir\b",
        // Git force operations
        r"(?i)\bgit\s+push\s+.*--force\b",
        r"(?i)\bgit\s+push\s+.*-f\b",
        r"(?i)\bgit\s+reset\s+--hard\b",
        r"(?i)\bgit\s+clean\s+.*-[a-z]*f\b",
        r"(?i)\bgit\s+branch\s+.*-D\b",
        // Database mutations
        r"(?i)\bDROP\s+TABLE\b",
        r"(?i)\bDROP\s+DATABASE\b",
        r"(?i)\bTRUNCATE\b",
        r"(?i)\bDELETE\s+FROM\s+\w+\s*(?!WHERE)\b",  // DELETE without WHERE
        // System/permission changes
        r"(?i)\bchmod\s+777\b",
        r"(?i)\bsudo\s+rm\b",
        r"(?i)\bmkfs\b",
        r"(?i)\bdd\s+if=",
        r"(?i)\bshutdown\b",
        r"(?i)\breboot\b",
        r"(?i)\bpkill\s+-9\b",
        r"(?i)\bkillall\b",
        r"(?i)\b>\s*/dev/sd[a-z]\b",   // overwrite disk device
        r"(?i)\bformat\s+[a-z]:\b",    // Windows-style format (cross-platform future)
    ]).expect("destructive pattern set is valid")
});

#[tauri::command]
pub fn check_destructive(command: String) -> bool {
    DESTRUCTIVE_PATTERNS.is_match(&command)
}
```

**Confidence:** HIGH -- `regex::RegexSet` is the canonical Rust API for multi-pattern matching against a single input string. `is_match` returns on first match without full scan.

### Pattern 2: Non-Streaming Explanation API Call

The explanation endpoint reuses the exact same reqwest client and auth pattern from `ai.rs` but sends a short non-streaming POST and reads the full response body.

```rust
// src-tauri/src/commands/safety.rs
#[tauri::command]
pub async fn get_destructive_explanation(
    command: String,
    on_result: tauri::ipc::Channel<String>,
) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT)
        .map_err(|e| format!("Keyring error: {}", e))?;
    let api_key = entry
        .get_password()
        .map_err(|_| "No API key configured".to_string())?;

    let body = serde_json::json!({
        "model": "grok-3-mini",   // cheaper/faster model for short explanation
        "messages": [
            {
                "role": "system",
                "content": "You are a safety assistant. In one plain-English sentence (max 20 words), \
                             explain what the following terminal command does and why it is destructive. \
                             Be specific about what data or state it will permanently change or delete. \
                             No markdown, no code fences."
            },
            {
                "role": "user",
                "content": &command
            }
        ],
        "stream": false,
        "temperature": 0.0
    }).to_string();

    let client = tauri_plugin_http::reqwest::Client::new();
    let response = client
        .post("https://api.x.ai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let bytes = response.bytes().await.map_err(|e| format!("Read error: {}", e))?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Parse error: {}", e))?;

    let explanation = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("This command makes irreversible changes.")
        .to_string();

    on_result.send(explanation).map_err(|e| format!("Channel error: {}", e))?;
    Ok(())
}
```

**Confidence:** HIGH -- pattern verified against existing `ai.rs` in this codebase. `tauri_plugin_http::reqwest` re-export + `.bytes()` + `serde_json::from_slice` is the documented workaround for the missing `.json()` feature (already used in `xai.rs`).

### Pattern 3: Zustand State Extension

Four new fields added to `OverlayState`, three new actions. All initialized to safe defaults in `show()` reset block.

```typescript
// New fields to add to OverlayState interface
isDestructive: boolean;
destructiveExplanation: string | null;   // null = not yet loaded
destructiveDismissed: boolean;
destructiveDetectionEnabled: boolean;    // loaded from settings.json on startup

// New actions
setIsDestructive: (value: boolean) => void;
setDestructiveExplanation: (explanation: string | null) => void;
dismissDestructiveBadge: () => void;
```

Reset in `show()` and `hide()`:
```typescript
isDestructive: false,
destructiveExplanation: null,
destructiveDismissed: false,
```

`destructiveDetectionEnabled` is NOT reset on show/hide -- it is a user preference persisted to `settings.json`.

### Pattern 4: Badge Placement in Overlay.tsx

The badge sits in the existing footer row alongside the shell badge. The footer `div` currently renders:

```tsx
<div className="flex items-center gap-2 min-h-[20px]">
  {isDetecting ? <spinner/> : badgeText ? <span>{badgeText}</span> : null}
</div>
```

Extended to:

```tsx
<div className="flex items-center gap-2 min-h-[20px]">
  {isDetecting ? <spinner/> : badgeText ? <span className="text-[11px] text-white/40 font-mono">{badgeText}</span> : null}
  {!destructiveDismissed && isDestructive && displayMode === "result" && (
    <DestructiveBadge />
  )}
</div>
```

The badge renders only when `displayMode === "result"` to honor the "badge appears after generation is complete" decision.

### Pattern 5: Radix UI Tooltip for Badge

`radix-ui` is already installed. The `Tooltip` primitive is available at `radix-ui/react-tooltip`.

```tsx
// src/components/DestructiveBadge.tsx
import * as Tooltip from "@radix-ui/react-tooltip";
import { invoke, Channel } from "@tauri-apps/api/core";
import { useOverlayStore } from "@/store";
import { useEffect, useState } from "react";

export function DestructiveBadge() {
  const streamingText = useOverlayStore((s) => s.streamingText);
  const destructiveExplanation = useOverlayStore((s) => s.destructiveExplanation);
  const dismissDestructiveBadge = useOverlayStore((s) => s.dismissDestructiveBadge);
  const setDestructiveExplanation = useOverlayStore((s) => s.setDestructiveExplanation);
  const [visible, setVisible] = useState(false);

  // Fade-in on mount
  useEffect(() => {
    const t = setTimeout(() => setVisible(true), 0);
    return () => clearTimeout(t);
  }, []);

  // Eager-load explanation when badge first mounts (not on hover)
  useEffect(() => {
    const ch = new Channel<string>();
    ch.onmessage = (explanation: string) => {
      setDestructiveExplanation(explanation);
    };
    invoke("get_destructive_explanation", { command: streamingText, onResult: ch })
      .catch(() => setDestructiveExplanation("This command makes irreversible changes."));
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <Tooltip.Provider delayDuration={200}>
      <Tooltip.Root>
        <Tooltip.Trigger asChild>
          <span
            className={[
              "text-[11px]",
              "font-mono",
              "px-1.5 py-0.5",
              "rounded",
              "bg-red-500/20",
              "text-red-400",
              "border border-red-500/30",
              "cursor-pointer",
              "transition-opacity duration-200",
              visible ? "opacity-100" : "opacity-0",
            ].join(" ")}
            onClick={dismissDestructiveBadge}
          >
            Destructive
          </span>
        </Tooltip.Trigger>
        <Tooltip.Portal>
          <Tooltip.Content
            side="top"
            sideOffset={4}
            className="max-w-[220px] text-[11px] text-white/80 bg-black/80 border border-white/10 rounded px-2 py-1.5 font-sans shadow-lg"
          >
            {destructiveExplanation ?? (
              <span className="inline-block w-3 h-3 border border-white/30 border-t-white/70 rounded-full animate-spin" />
            )}
            <Tooltip.Arrow className="fill-black/80" />
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip.Root>
    </Tooltip.Provider>
  );
}
```

**Confidence:** HIGH -- Radix UI Tooltip API verified from package presence and standard Radix pattern. The `radix-ui` umbrella package exposes `@radix-ui/react-tooltip` internally.

### Pattern 6: Triggering Detection in submitQuery

The detection call happens in the `submitQuery` success block (in `store/index.ts`), immediately after `stream_ai_response` resolves:

```typescript
// Inside submitQuery success block, after set({ isStreaming: false, displayMode: "result", ... }):
const state = useOverlayStore.getState();
if (state.destructiveDetectionEnabled && finalText) {
  invoke<boolean>("check_destructive", { command: finalText })
    .then((isDestructive) => {
      if (isDestructive) {
        useOverlayStore.getState().setIsDestructive(true);
        // Explanation is loaded eagerly by DestructiveBadge on mount
      }
    })
    .catch(console.error);
}
```

This keeps detection outside the streaming hot path and fires synchronously after the result is ready.

### Pattern 7: Settings Toggle in PreferencesTab

```tsx
// PreferencesTab.tsx addition
const destructiveDetectionEnabled = useOverlayStore((s) => s.destructiveDetectionEnabled);

const handleToggle = async (enabled: boolean) => {
  useOverlayStore.getState().setDestructiveDetectionEnabled(enabled);
  const store = await Store.load("settings.json");
  await store.set("destructiveDetectionEnabled", enabled);
  await store.save();
};
```

Toggle rendered as a simple `<button>` using the existing pattern (no new UI library needed):

```tsx
<div className="flex items-center justify-between">
  <span className="text-white/70 text-xs">Destructive command warnings</span>
  <button
    type="button"
    onClick={() => handleToggle(!destructiveDetectionEnabled)}
    className={[
      "w-8 h-4 rounded-full transition-colors",
      destructiveDetectionEnabled ? "bg-red-500/60" : "bg-white/10",
    ].join(" ")}
    aria-label="Toggle destructive command detection"
  />
</div>
```

### Anti-Patterns to Avoid

- **Detecting during streaming:** Only run detection after `displayMode` transitions to `"result"`. Mid-stream text is incomplete and will produce false positives.
- **Blocking clipboard copy:** The user decided copy is never blocked. Do not add confirmation logic.
- **Fetching explanation on every tooltip open:** Fetch eagerly on badge mount, not on hover -- avoids flicker if user hovers immediately.
- **Using `.json()` on reqwest response:** This project's `tauri_plugin_http` reqwest re-export lacks the `json` feature. Use `.bytes()` + `serde_json::from_slice` (same pattern as `xai.rs`).
- **Persisting `destructiveDismissed` across sessions:** Dismiss is per-result only. Do not write it to `settings.json`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tooltip with accessible positioning | Custom `position: absolute` div | `@radix-ui/react-tooltip` | Handles viewport overflow, pointer events, aria-describedby, exit animation |
| Multi-pattern string matching | Sequential `.contains()` checks | `regex::RegexSet` | Single-pass matching, compiled once, handles word boundaries correctly |
| Badge fade-in | setTimeout style manipulation | CSS `transition-opacity` + `useEffect` visibility toggle | Zero-JS animation cost, composable with Tailwind |

**Key insight:** The entire detection and explanation pipeline reuses zero new dependencies -- all capabilities are already in the project.

---

## Common Pitfalls

### Pitfall 1: `radix-ui` umbrella package import path

**What goes wrong:** Importing `from "radix-ui/react-tooltip"` fails at build time; Vite/TypeScript cannot resolve it.
**Why it happens:** The `radix-ui` umbrella package (v1.4.3 installed) re-exports all primitives but may have different internal resolution than the individual `@radix-ui/react-tooltip` package.
**How to avoid:** Import as `import * as Tooltip from "@radix-ui/react-tooltip"` -- check if this path resolves from the umbrella, or install `@radix-ui/react-tooltip` directly if needed. Run `node -e "require('radix-ui/react-tooltip')"` in the project root to verify before coding.
**Warning signs:** TypeScript error "Cannot find module 'radix-ui/react-tooltip'" during dev server start.

**Resolution:** If umbrella import fails, install the individual package:
```bash
npm install @radix-ui/react-tooltip
```
This is a single small package, no bundling concern.

### Pitfall 2: Regex false positives on substring matches

**What goes wrong:** Pattern `rm -r` matches inside `npm run` or `yarn remove` when written naively.
**Why it happens:** Naive string matching without word boundaries.
**How to avoid:** All patterns use `\b` word boundaries and require whitespace after the command name. The `RegexSet` patterns above include these guards. Test each pattern against both positive and negative examples before shipping.
**Warning signs:** Shell badge shows "Destructive" for `npm run dev`.

### Pitfall 3: Explanation call races with badge visibility

**What goes wrong:** User hovers tooltip before the explanation IPC call resolves; tooltip flickers or shows empty string.
**Why it happens:** Network latency on the explanation call (typically 200-800ms).
**How to avoid:** Tooltip shows a spinner (`animate-spin` border div) when `destructiveExplanation === null`. Switch to text when non-null. This is already in the code pattern above.
**Warning signs:** Tooltip appears empty or closes unexpectedly on hover.

### Pitfall 4: Badge persists after result is replaced by new query

**What goes wrong:** User submits a new query; previous destructive badge stays visible during new streaming.
**Why it happens:** `isDestructive` is not reset when streaming begins.
**How to avoid:** Reset `isDestructive: false`, `destructiveExplanation: null`, `destructiveDismissed: false` at the start of `submitQuery` (when transitioning to `displayMode: "streaming"`).
**Warning signs:** Badge flashes briefly at start of a new non-destructive query.

### Pitfall 5: Settings toggle not loaded on startup

**What goes wrong:** `destructiveDetectionEnabled` always defaults to `true` even if user toggled it off.
**Why it happens:** Zustand initializes from hardcoded defaults; `settings.json` not read on startup.
**How to avoid:** In `App.tsx`'s startup `useEffect` (the one that loads hotkey + model), also read `destructiveDetectionEnabled` from the store and set it into Zustand state.
**Warning signs:** Toggle appears to reset on every app restart.

---

## Code Examples

### Registering safety.rs commands in lib.rs

```rust
// src-tauri/src/commands/mod.rs  -- add:
pub mod safety;

// src-tauri/src/lib.rs  -- in .invoke_handler:
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::safety::check_destructive,
    commands::safety::get_destructive_explanation,
])
```

### Complete destructive pattern list (expanded from CONTEXT.md)

```rust
static DESTRUCTIVE_PATTERNS: Lazy<RegexSet> = Lazy::new(|| {
    RegexSet::new([
        // File destruction
        r"(?i)\brm\s+(-[a-z]*r[a-z]*f|-[a-z]*f[a-z]*r|--recursive.*--force|--force.*--recursive)\b",
        r"(?i)\brm\s+-[a-z]*r\b",
        r"(?i)\bshred\b",
        r"(?i)\bunlink\b",
        r"(?i)\brmdir\b",
        // Git force operations
        r"\bgit\s+push\s+.*--force\b",
        r"\bgit\s+push\s+.*\s-f\b",
        r"\bgit\s+reset\s+--hard\b",
        r"\bgit\s+clean\s+.*-[a-zA-Z]*f\b",
        r"\bgit\s+branch\s+.*-D\b",
        r"\bgit\s+rebase\s+.*--force\b",
        // Database mutations
        r"(?i)\bDROP\s+(TABLE|DATABASE|SCHEMA|INDEX)\b",
        r"(?i)\bTRUNCATE\s+TABLE\b",
        r"(?i)\bDELETE\s+FROM\s+\w+\s*;",   // DELETE without WHERE (ends with semicolon)
        r"(?i)\bDELETE\s+FROM\s+\w+\s*$",   // DELETE without WHERE (end of string)
        // System-level destructive operations
        r"(?i)\bsudo\s+rm\b",
        r"(?i)\bchmod\s+777\b",
        r"(?i)\bmkfs\b",
        r"(?i)\bdd\s+if=",
        r"(?i)\bshutdown\b",
        r"(?i)\breboot\b",
        r"(?i)\bpkill\s+-9\b",
        r"(?i)\bkillall\b",
        r"(?i)\bfdisk\b",
        r"(?i)\bdiskutil\s+erase\b",
        r"(?i)\bformat\s+[a-zA-Z]:\b",       // Windows format (future cross-platform)
        r"(?i)>\s*/dev/sd[a-z]",             // Overwrite disk device
        r"(?i)>\s*/dev/disk[0-9]",           // macOS disk device overwrite
    ]).expect("destructive pattern set must be valid")
});
```

### Loading toggle preference on startup (App.tsx addition)

```typescript
// Inside the checkOnboarding useEffect, after loading savedModel:
const destructiveEnabled = await store.get<boolean>("destructiveDetectionEnabled");
// undefined means never set = default to true
useOverlayStore.getState().setDestructiveDetectionEnabled(destructiveEnabled ?? true);
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `lazy_static!` macro for compiled regex | `once_cell::sync::Lazy` | ~2022 | `once_cell` is already a dep; standard in modern Rust |
| Individual `@radix-ui/*` packages | `radix-ui` umbrella package | 2024 | Single install, all primitives included; import paths may differ |

**Deprecated/outdated:**
- `lazy_static` crate: Superseded by `once_cell` (already in Cargo.toml). Do not add `lazy_static` as a dep.

---

## Open Questions

1. **`radix-ui` umbrella tooltip import path**
   - What we know: `radix-ui@1.4.3` is installed as an umbrella package; Radix tooltip is definitely available somewhere in the tree
   - What's unclear: Whether `import * as Tooltip from "radix-ui/react-tooltip"` resolves, or if `@radix-ui/react-tooltip` must be installed individually
   - Recommendation: First task of planning phase should be a 5-line verification: `import * as Tooltip from "radix-ui/react-tooltip"` in a test file and check TypeScript resolution. If it fails, `npm install @radix-ui/react-tooltip` is the fix (no design change needed).

2. **`grok-3-mini` availability and model name**
   - What we know: `grok-3` is validated and working; xAI likely has cheaper/faster variants
   - What's unclear: Exact model ID string for the mini variant; whether it exists at all on xAI
   - Recommendation: Fallback gracefully -- use `selectedModel` from Zustand state if `grok-3-mini` fails validation. The explanation call is non-blocking so a failure just leaves the tooltip in spinner state with a fallback string.

3. **DELETE without WHERE regex reliability**
   - What we know: Matching "DELETE FROM table" without "WHERE" via regex in a single-line command is straightforward
   - What's unclear: Multi-line SQL in a heredoc would need multiline matching
   - Recommendation: Single-line matching is sufficient for the v1 scope (AI-generated commands are single-line). Flag as a known limitation.

---

## Sources

### Primary (HIGH confidence)
- Codebase: `/src-tauri/src/commands/ai.rs` -- reqwest pattern, Keychain constants, Channel usage
- Codebase: `/src-tauri/Cargo.toml` -- confirmed `regex`, `once_cell`, `keyring`, `tauri-plugin-http` already present
- Codebase: `/src/store/index.ts` -- Zustand state shape, `submitQuery` async pattern
- Codebase: `/src/components/Overlay.tsx` -- badge row placement point confirmed
- Codebase: `/src/components/Settings/PreferencesTab.tsx` -- toggle will be added here
- Codebase: `/package.json` -- `radix-ui@1.4.3` confirmed installed
- `regex::RegexSet` Rust docs (standard library; HIGH confidence from training data + codebase usage)
- `once_cell::sync::Lazy` Rust docs (standard library; HIGH confidence from Cargo.toml)

### Secondary (MEDIUM confidence)
- Radix UI Tooltip API pattern (verified via radix-ui.com docs structure; import path question remains)
- xAI API non-streaming POST pattern (inferred from streaming pattern in ai.rs + xAI docs consistency)

### Tertiary (LOW confidence)
- `grok-3-mini` model ID -- unverified; xAI model lineup may differ; fallback to `selectedModel` recommended

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in project, no new deps required
- Architecture: HIGH -- patterns derived directly from existing codebase (ai.rs, store/index.ts, Overlay.tsx)
- Pitfalls: HIGH -- grounded in known codebase constraints (reqwest re-export limitation, streaming reset patterns)
- Radix tooltip import: MEDIUM -- package installed, exact subpath needs 1-line verification

**Research date:** 2026-02-23
**Valid until:** 2026-03-23 (stable stack; only risk is xAI model name changes)
