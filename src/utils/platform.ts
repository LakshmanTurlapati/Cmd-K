/// Detect if the current platform is Windows via navigator.userAgent.
/// Works in Tauri's WebView2 (Windows) and WKWebView (macOS) runtimes.
export function isWindows(): boolean {
  return navigator.userAgent.includes("Windows");
}

/// Detect if the current platform is Linux via navigator.userAgent.
/// Excludes Android which also reports "Linux" in its UA string.
export function isLinux(): boolean {
  return navigator.userAgent.includes("Linux") && !navigator.userAgent.includes("Android");
}

/// Display the platform-appropriate modifier key label.
/// "Super" → "Cmd" on macOS, "Ctrl" on Windows/Linux.
/// Other modifiers pass through unchanged.
export function displayModifier(key: string): string {
  if (key === "Super") {
    return (isWindows() || isLinux()) ? "Ctrl" : "Cmd";
  }
  if (key === "Alt") {
    return (isWindows() || isLinux()) ? "Alt" : "Option";
  }
  return key;
}
