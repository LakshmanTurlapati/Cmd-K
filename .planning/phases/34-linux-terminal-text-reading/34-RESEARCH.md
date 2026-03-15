# Phase 34: Linux Terminal Text Reading - Research

**Researched:** 2026-03-15
**Domain:** Linux accessibility APIs, terminal emulator IPC, D-Bus
**Confidence:** MEDIUM

## Summary

This phase adds terminal text reading on Linux by implementing three strategies: AT-SPI2 D-Bus for VTE-based and Qt terminals, kitty remote control subprocess, and WezTerm CLI subprocess. The architecture directly parallels the existing `ax_reader.rs` (macOS) and `uia_reader.rs` (Windows) modules.

AT-SPI2 is the most complex strategy. It requires connecting to the accessibility bus (separate from the session bus), enumerating applications to find the terminal by PID, walking the accessible tree to find a widget with role `Terminal` (value 60), then calling `GetText(0, character_count)` on the `org.a11y.atspi.Text` interface. The `zbus` crate (v5.x) provides a pure-Rust blocking D-Bus client that avoids C library dependencies. The kitty and WezTerm strategies are simple subprocess calls with timeouts.

**Primary recommendation:** Use `zbus` (with `blocking` feature) directly for AT-SPI2 D-Bus calls. Do NOT use the higher-level `atspi` crate -- it is async-only (requires tokio runtime), adds heavy dependencies, and our use case is a simple blocking request-response pattern. Raw zbus `blocking::Connection` with manual D-Bus method calls is simpler, lighter, and matches the sync call pattern established by macOS/Windows readers.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Use `zbus` crate for pure Rust async D-Bus client (no C deps, no libdbus/libatspi linking)
- Sync D-Bus calls with 500ms timeout -- matches macOS AX reader pattern
- Request visible screen text only (not scrollback) -- consistent with macOS/Windows behavior
- Find terminal widget by AT-SPI2 role-based match (VTE widgets report role "terminal") -- targeted, fast
- If AT-SPI2 is unavailable or times out, return None silently
- kitty: shell out to `kitty @ get-text --extent screen` -- uses kitty's built-in IPC socket
- WezTerm: shell out to `wezterm cli get-text` -- reads visible pane text
- Screen-only text for both (no scrollback) -- consistent with AT-SPI2 decision
- 500ms timeout for both subprocess calls -- keeps hotkey response snappy
- If kitty remote control is disabled, return None silently -- zero setup principle
- Process name match from /proc/PID/exe determines which strategy to use
- Routing map: gnome-terminal/tilix/terminator/mate-terminal/xfce4-terminal/konsole -> AT-SPI2, kitty -> kitty @, wezterm-gui -> wezterm cli, everything else -> None
- No fallback chain -- if matched strategy fails, return None immediately
- Reader function signature: `read_terminal_text_linux(pid, exe_name)` -- accepts terminal process name from caller
- Single `terminal/linux_reader.rs` module paralleling `ax_reader.rs` and `uia_reader.rs`
- One pub fn `read_terminal_text_linux()` dispatches to internal AT-SPI2/kitty/WezTerm functions
- Wire into `detect_inner_linux()` in `mod.rs` to populate `visible_output` (currently `None` at line 323)
- zbus as `cfg(target_os = "linux")` dependency -- only compiles on Linux

### Claude's Discretion
- Exact zbus API calls for AT-SPI2 text extraction
- Process name matching list (additional VTE-based terminals beyond the ones listed)
- Error handling granularity (which errors to log vs silently ignore)
- Test strategy and mocking approach for D-Bus and subprocess calls
- Internal data structures for routing

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LTXT-01 | AT-SPI2 D-Bus integration reads terminal text from VTE-based terminals (GNOME Terminal, Tilix, Terminator) | AT-SPI2 architecture documented: connect to a11y bus, find app by PID, walk tree for role=Terminal (60), call GetText on org.a11y.atspi.Text interface |
| LTXT-02 | kitty remote control reads terminal text from kitty | `kitty @ get-text --extent screen` documented; requires `allow_remote_control` in kitty.conf |
| LTXT-03 | WezTerm CLI reads terminal text from WezTerm | `wezterm cli get-text` documented; reads visible pane text by default |
| LTXT-04 | Graceful None return for terminals without text reading support | Routing map with fallthrough to None established in CONTEXT.md |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| zbus | 5.x | Pure Rust D-Bus client for AT-SPI2 communication | No C deps, blocking API, actively maintained, 2M+ downloads |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::process::Command | stdlib | Subprocess calls for kitty/WezTerm CLI | For `kitty @ get-text` and `wezterm cli get-text` |
| std::time::Duration | stdlib | 500ms timeout for all strategies | Subprocess timeout + D-Bus call timeout |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| zbus (raw) | atspi crate | atspi is async-only, heavy deps (tokio runtime), overkill for simple GetText call |
| zbus | dbus-rs (libdbus) | dbus-rs requires libdbus C library at runtime -- breaks zero-dep principle |
| Command subprocess | kitty IPC socket directly | Direct socket is faster but requires parsing kitty's binary protocol -- not worth complexity |

**Installation:**
```toml
[target.'cfg(target_os = "linux")'.dependencies]
zbus = { version = "5", default-features = false, features = ["blocking"] }
```

## Architecture Patterns

### Recommended Module Structure
```
src-tauri/src/terminal/
  linux_reader.rs    # NEW - parallels ax_reader.rs / uia_reader.rs
  mod.rs             # Wire linux_reader into detect_inner_linux()
  detect_linux.rs    # Already has exe name lists (extend routing)
```

### Pattern 1: Dispatch by Process Name
**What:** Single pub fn routes to strategy based on exe_name string match
**When to use:** Always -- this is the entry point
**Example:**
```rust
#[cfg(target_os = "linux")]
pub fn read_terminal_text_linux(pid: i32, exe_name: &str) -> Option<String> {
    match exe_name {
        // VTE-based terminals (GTK)
        "gnome-terminal-server" | "tilix" | "terminator"
        | "mate-terminal" | "xfce4-terminal" | "guake"
        | "tilda" | "sakura" | "lxterminal" | "terminology"
        // Qt-based terminals (also use AT-SPI2 via qt-atspi bridge)
        | "konsole" => read_via_atspi(pid),
        // Direct API terminals
        "kitty" => read_via_kitty(),
        "wezterm-gui" => read_via_wezterm(),
        // Everything else: no text reading support
        _ => None,
    }
}
```

### Pattern 2: AT-SPI2 D-Bus Connection Flow
**What:** Multi-step D-Bus calls to read terminal text via accessibility bus
**When to use:** For VTE and Qt terminals
**Example flow:**
```rust
fn read_via_atspi(pid: i32) -> Option<String> {
    // Step 1: Connect to session bus
    let session = zbus::blocking::Connection::session().ok()?;

    // Step 2: Get AT-SPI2 bus address via org.a11y.Bus.GetAddress
    let a11y_addr: String = session
        .call_method(
            Some("org.a11y.Bus"),           // destination
            "/org/a11y/bus",                // path
            Some("org.a11y.Bus"),           // interface
            "GetAddress",                    // method
            &(),                            // no args
        ).ok()?
        .body().deserialize().ok()?;

    // Step 3: Connect to accessibility bus
    let a11y_conn = zbus::blocking::Connection::builder()
        .address(a11y_addr.as_str())?
        .build()
        .ok()?;

    // Step 4: Get desktop children (list of apps) from registry
    // Desktop object: bus_name=org.a11y.atspi.Registry, path=/org/a11y/atspi/accessible/root
    // Call GetChildren on org.a11y.atspi.Accessible interface
    // Returns Vec<(String, OwnedObjectPath)> -- (bus_name, obj_path) pairs

    // Step 5: For each app, check if its PID matches our target
    // Call GetChildren, then GetRole on children to find role=60 (Terminal)

    // Step 6: Once terminal widget found, read text:
    // Call CharacterCount property on org.a11y.atspi.Text
    // Call GetText(0, char_count) on org.a11y.atspi.Text
    // Return the text

    None // placeholder
}
```

### Pattern 3: Subprocess with Timeout
**What:** Shell out to terminal-specific CLI with 500ms timeout
**When to use:** For kitty and WezTerm
**Example:**
```rust
fn read_via_kitty() -> Option<String> {
    let output = std::process::Command::new("kitty")
        .args(["@", "get-text", "--extent", "screen"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None; // Remote control likely disabled
    }

    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    if text.trim().is_empty() { None } else { Some(text) }
}

fn read_via_wezterm() -> Option<String> {
    let output = std::process::Command::new("wezterm")
        .args(["cli", "get-text"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    if text.trim().is_empty() { None } else { Some(text) }
}
```

### Pattern 4: Non-macOS/non-Linux Stub
**What:** Cross-compilation stub that returns None
**When to use:** Always provide for non-Linux targets
**Example:**
```rust
#[cfg(not(target_os = "linux"))]
pub fn read_terminal_text_linux(_pid: i32, _exe_name: &str) -> Option<String> {
    None
}
```

### Anti-Patterns to Avoid
- **Using the `atspi` crate:** Pulls in tokio, async runtime complexity, and 10+ transitive deps for a single blocking call. Use raw zbus.
- **Linking libdbus or libatspi:** C library deps break on systems without dev packages installed. zbus is pure Rust.
- **Cascading fallback strategies:** If AT-SPI2 fails for gnome-terminal, do NOT try kitty or wezterm APIs. The routing map is deterministic.
- **Blocking the main thread indefinitely:** All strategies MUST have 500ms timeout. D-Bus and subprocess calls can hang.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| D-Bus protocol | Custom socket parser | zbus crate | D-Bus wire protocol is complex (endianness, type signatures, auth) |
| AT-SPI2 bus discovery | Manual xprop parsing | org.a11y.Bus.GetAddress D-Bus call | Standard D-Bus method, works regardless of X11/Wayland |
| kitty text reading | Parse kitty IPC socket protocol | `kitty @ get-text` subprocess | kitty IPC protocol is undocumented/internal, CLI is stable API |
| WezTerm text reading | Parse WezTerm mux protocol | `wezterm cli get-text` subprocess | Mux protocol is internal, CLI is stable |
| Subprocess timeout | Manual thread+kill | `Command::output()` with thread timeout wrapper | Standard pattern from existing codebase |

**Key insight:** Terminal emulators provide stable CLI interfaces for text extraction. Only AT-SPI2 requires D-Bus protocol knowledge, and zbus handles that entirely.

## Common Pitfalls

### Pitfall 1: AT-SPI2 Bus Not Running
**What goes wrong:** `org.a11y.Bus` service not available on session bus -- D-Bus call fails
**Why it happens:** AT-SPI2 is not universally enabled. Minimal desktops (i3, sway, dwm) may not start the accessibility bus. Some distros disable it by default for performance.
**How to avoid:** Wrap entire AT-SPI2 flow in early-return Option chain. If session bus call to GetAddress fails, return None immediately. No logging spam.
**Warning signs:** Getting D-Bus errors like "ServiceUnknown" or "NameHasNoOwner"

### Pitfall 2: AT-SPI2 App Not Found by PID
**What goes wrong:** Walking all applications in the registry finds no match for the terminal PID
**Why it happens:** AT-SPI2 applications register with their own bus names, not PIDs. The PID is exposed as a property on the application accessible, but not all apps set it correctly. Also, gnome-terminal-server may have a different PID than the window manager thinks.
**How to avoid:** When iterating apps, try to match by PID. But also consider that the app may use a different PID (e.g., gnome-terminal uses a server process). If no match, return None.
**Warning signs:** Finding 0 or many applications but none matching PID

### Pitfall 3: Qt Accessibility Not Enabled
**What goes wrong:** Konsole and other KDE apps don't expose AT-SPI2 tree
**Why it happens:** Qt only loads the AT-SPI2 bridge plugin when `QT_ACCESSIBILITY=1` environment variable is set. Many KDE setups don't set this by default.
**How to avoid:** Accept that Konsole AT-SPI2 may not work for all users. Return None gracefully. This is within the "zero setup" principle.
**Warning signs:** Konsole appears in process list but not in AT-SPI2 registry

### Pitfall 4: kitty Remote Control Disabled by Default
**What goes wrong:** `kitty @ get-text` returns error because remote control is off
**Why it happens:** kitty ships with `allow_remote_control no` by default for security. Users must explicitly enable it in kitty.conf.
**How to avoid:** Check exit status of kitty command. If non-zero, return None silently. Never prompt users to change their config.
**Warning signs:** kitty @ commands return exit code 1 with error message about remote control

### Pitfall 5: Subprocess Hangs
**What goes wrong:** `kitty @` or `wezterm cli` blocks indefinitely
**Why it happens:** If the IPC socket is in a bad state, or the terminal process is hung, subprocess never returns
**How to avoid:** Implement 500ms timeout using `std::thread` + `Command::output()` pattern, or use `wait_timeout` from child process. Kill child if timeout exceeded.
**Warning signs:** Hotkey response becomes very slow for specific terminals

### Pitfall 6: zbus Async Sandwich
**What goes wrong:** Using zbus blocking API from within tokio async context causes panic
**Why it happens:** zbus blocking module runs its own `block_on` internally, which conflicts with an already-running tokio runtime
**How to avoid:** The terminal detection path is called synchronously (via `mpsc::channel` with timeout in `detect_app_context`). This is safe. Do NOT refactor to async. The sync path is intentional.
**Warning signs:** Runtime panic about "Cannot start a runtime from within a runtime"

### Pitfall 7: ANSI Escape Sequences in Output
**What goes wrong:** Terminal text contains ANSI codes that pollute AI context
**Why it happens:** AT-SPI2 may return raw text with escapes. `wezterm cli get-text` returns raw text by default (without `--escapes` flag, it strips them). kitty may include some escape sequences.
**How to avoid:** Phase 33's `context.rs` already handles ANSI stripping downstream. The reader should return raw text and let the pipeline handle cleanup.
**Warning signs:** AI responses reference "[0m" or other escape artifacts

## Code Examples

### AT-SPI2 D-Bus Method Call via zbus Blocking
```rust
// Source: zbus docs + AT-SPI2 spec
use zbus::blocking::Connection;

fn get_atspi_bus() -> Option<Connection> {
    // Step 1: session bus
    let session = Connection::session().ok()?;

    // Step 2: Get accessibility bus address
    let reply = session.call_method(
        Some("org.a11y.Bus"),
        "/org/a11y/bus",
        Some("org.a11y.Bus"),
        "GetAddress",
        &(),
    ).ok()?;
    let addr: String = reply.body().deserialize().ok()?;

    // Step 3: Connect to AT-SPI2 bus
    Connection::builder()
        .address(addr.as_str())
        .ok()?
        .build()
        .ok()
}
```

### AT-SPI2 Role Check (Terminal = 60)
```rust
// Source: AT-SPI2 spec, atspi crate enum
const ATSPI_ROLE_TERMINAL: u32 = 60;

fn get_role(conn: &Connection, bus_name: &str, path: &str) -> Option<u32> {
    let reply = conn.call_method(
        Some(bus_name),
        path,
        Some("org.a11y.atspi.Accessible"),
        "GetRole",
        &(),
    ).ok()?;
    reply.body().deserialize::<u32>().ok()
}
```

### AT-SPI2 GetText Call
```rust
// Source: AT-SPI2 org.a11y.atspi.Text spec
fn get_text(conn: &Connection, bus_name: &str, path: &str) -> Option<String> {
    // Get character count first
    let count_reply = conn.call_method(
        Some(bus_name),
        path,
        Some("org.freedesktop.DBus.Properties"),
        "Get",
        &("org.a11y.atspi.Text", "CharacterCount"),
    ).ok()?;
    // CharacterCount is returned as a Variant containing i32
    let count: i32 = /* deserialize from variant */ 0;

    if count <= 0 { return None; }

    let text_reply = conn.call_method(
        Some(bus_name),
        path,
        Some("org.a11y.atspi.Text"),
        "GetText",
        &(0i32, count),
    ).ok()?;
    let text: String = text_reply.body().deserialize().ok()?;

    if text.trim().is_empty() { None } else { Some(text) }
}
```

### Integration into detect_inner_linux
```rust
// In mod.rs, line ~320-327, replace:
//   visible_output: None, // Terminal text reading is Phase 34
// With:
#[cfg(target_os = "linux")]
{
    let visible_output = if is_terminal {
        linux_reader::read_terminal_text_linux(previous_app_pid, exe_str)
    } else {
        None
    };
    // ... use visible_output in TerminalContext
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| libdbus + libatspi C bindings | zbus pure Rust D-Bus | zbus 1.0 (2020) | No C deps, no pkg-config, no system lib requirements |
| dbus-rs crate | zbus | zbus 2.0+ (2022) | dbus-rs requires libdbus.so at runtime; zbus is pure Rust |
| atspi crate for text reading | Raw zbus calls | Current | atspi pulls tokio, too heavy for one blocking call |
| `kitten @` syntax | `kitty @` syntax | kitty 0.26+ | Both work; `kitty @` is canonical in current docs |

**Deprecated/outdated:**
- `dbus-rs` crate: Still works but requires C library. Not recommended for new projects.
- `at-spi2-atk`: The ATK adaptation layer is deprecated in favor of direct AT-SPI2 D-Bus.

## Open Questions

1. **AT-SPI2 application PID matching reliability**
   - What we know: Applications register on the AT-SPI2 bus with bus names. The PID can be read via `org.a11y.atspi.Application` interface property.
   - What's unclear: Do all VTE-based terminals reliably expose their PID? Does gnome-terminal-server's PID match what X11 reports as the window owner?
   - Recommendation: Implement PID matching as primary strategy. If it fails, return None. Can be improved later with bus-name heuristics if needed.

2. **zbus `blocking::Connection::builder().address()` API**
   - What we know: zbus 5.x has a builder pattern for connections. Address can be set.
   - What's unclear: The exact API for `address()` -- whether it takes `&str` or a parsed `Address` type. zbus 5.x may have changed API from 4.x.
   - Recommendation: Verify API against docs.rs during implementation. Fallback: use `Connection::builder().address()` or parse address string.

3. **Subprocess timeout implementation**
   - What we know: `std::process::Command::output()` blocks until completion. No built-in timeout.
   - What's unclear: Best pattern for 500ms timeout on subprocess in sync context.
   - Recommendation: Use `Command::spawn()` + `child.wait_timeout()` (unstable), or spawn a thread that calls `output()` with `recv_timeout` on the channel -- same pattern used in existing codebase's `detect_app_context` timeout wrapper.

## Sources

### Primary (HIGH confidence)
- [AT-SPI2 org.a11y.atspi.Text spec](https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/doc-org.a11y.atspi.Text.html) - GetText(startOffset, endOffset), CharacterCount property
- [AT-SPI2 org.a11y.atspi.Accessible spec](https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/doc-org.a11y.atspi.Accessible.html) - GetRole, GetChildren, ChildCount
- [atspi Role enum](https://docs.rs/atspi/latest/atspi/enum.Role.html) - Terminal role = 60
- [kitty remote control docs](https://sw.kovidgoyal.net/kitty/remote-control/) - `kitty @ get-text --extent screen`
- [WezTerm CLI get-text docs](https://wezterm.org/cli/cli/get-text.html) - `wezterm cli get-text`
- [zbus blocking API](https://docs.rs/zbus/latest/zbus/blocking/index.html) - zbus 5.14.0 blocking module

### Secondary (MEDIUM confidence)
- [AT-SPI2 freedesktop wiki](https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/) - Architecture overview
- [at-spi2-core bus README](https://github.com/GNOME/at-spi2-core/blob/main/bus/README.md) - Bus launcher architecture
- [KDE Qt AT-SPI bridge](https://community.kde.org/Accessibility/qt-atspi) - Qt needs QT_ACCESSIBILITY=1
- [PyAtSpi2Example](https://www.freedesktop.org/wiki/Accessibility/PyAtSpi2Example/) - Desktop/application tree walk pattern

### Tertiary (LOW confidence)
- zbus version: Docs show 5.14.0 but crates.io API was unreachable for direct verification
- Konsole AT-SPI2 text reliability: Qt AT-SPI bridge exists but terminal text quality untested

## Metadata

**Confidence breakdown:**
- Standard stack: MEDIUM - zbus is clearly the right choice but exact API for AT-SPI2 bus connection needs implementation-time verification
- Architecture: HIGH - Pattern is well-established by macOS/Windows readers, AT-SPI2 spec is stable
- Pitfalls: HIGH - AT-SPI2 availability issues are well-documented, kitty remote control disabled by default is confirmed in official docs
- Subprocess strategy: HIGH - kitty and WezTerm CLI APIs are simple and well-documented

**Research date:** 2026-03-15
**Valid until:** 2026-04-15 (stable domain, AT-SPI2 spec rarely changes)
