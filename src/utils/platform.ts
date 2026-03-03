/// Detect if the current platform is Windows via navigator.userAgent.
/// Works in Tauri's WebView2 (Windows) and WKWebView (macOS) runtimes.
export function isWindows(): boolean {
  return navigator.userAgent.includes("Windows");
}

/// Display the platform-appropriate modifier key label.
/// "Super" → "Cmd" on macOS, "Ctrl" on Windows.
/// Other modifiers pass through unchanged.
export function displayModifier(key: string): string {
  if (key === "Super") {
    return isWindows() ? "Ctrl" : "Cmd";
  }
  if (key === "Alt") {
    return isWindows() ? "Alt" : "Option";
  }
  return key;
}
