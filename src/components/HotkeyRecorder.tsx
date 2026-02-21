import { useEffect, useRef, useState } from "react";

interface HotkeyRecorderProps {
  onCapture: (shortcut: string) => void;
  onCancel: () => void;
}

interface CapturedKeys {
  meta: boolean;
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  key: string;
}

function codeToDisplayName(code: string): string {
  if (code === "Space") return "Space";
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  // Arrow keys, function keys, etc.
  return code.replace("Arrow", "").replace("Bracket", "]").replace("Semicolon", ";");
}

function keysToDisplayString(keys: CapturedKeys): string {
  const parts: string[] = [];
  if (keys.meta) parts.push("Cmd");
  if (keys.ctrl) parts.push("Ctrl");
  if (keys.alt) parts.push("Option");
  if (keys.shift) parts.push("Shift");
  if (keys.key) parts.push(codeToDisplayName(keys.key));
  return parts.join(" + ");
}

function keysToTauriString(keys: CapturedKeys): string {
  const parts: string[] = [];
  if (keys.meta) parts.push("Super");
  if (keys.ctrl) parts.push("Control");
  if (keys.alt) parts.push("Alt");
  if (keys.shift) parts.push("Shift");
  if (keys.key) parts.push(keys.key);
  return parts.join("+");
}

export function HotkeyRecorder({ onCapture, onCancel }: HotkeyRecorderProps) {
  const [displayText, setDisplayText] = useState("Press a key combination...");
  const containerRef = useRef<HTMLDivElement>(null);
  // Use a ref to track current captured keys without triggering re-renders
  const capturedRef = useRef<CapturedKeys>({
    meta: false,
    ctrl: false,
    alt: false,
    shift: false,
    key: "",
  });

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      const modifierCodes = ["MetaLeft", "MetaRight", "ControlLeft", "ControlRight", "AltLeft", "AltRight", "ShiftLeft", "ShiftRight"];
      const isModifier = modifierCodes.includes(e.code);

      capturedRef.current = {
        meta: e.metaKey,
        ctrl: e.ctrlKey,
        alt: e.altKey,
        shift: e.shiftKey,
        key: isModifier ? "" : e.code,
      };

      const keys = capturedRef.current;
      const hasModifier = keys.meta || keys.ctrl || keys.alt || keys.shift;

      if (hasModifier && keys.key) {
        setDisplayText(keysToDisplayString(keys));
      } else if (hasModifier) {
        setDisplayText(keysToDisplayString({ ...keys, key: "" }) + " + ...");
      } else {
        setDisplayText("Press a key combination...");
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      const prev = capturedRef.current;
      const hasModifier = prev.meta || prev.ctrl || prev.alt || prev.shift;

      if (hasModifier && prev.key) {
        // Finalize: call onCapture with the Tauri-format shortcut
        onCapture(keysToTauriString(prev));
      }

      // Update ref with current modifier state after keyup
      capturedRef.current = {
        meta: e.metaKey,
        ctrl: e.ctrlKey,
        alt: e.altKey,
        shift: e.shiftKey,
        key: "",
      };
    };

    window.addEventListener("keydown", handleKeyDown, true);
    window.addEventListener("keyup", handleKeyUp, true);

    return () => {
      window.removeEventListener("keydown", handleKeyDown, true);
      window.removeEventListener("keyup", handleKeyUp, true);
    };
  }, [onCapture]);

  useEffect(() => {
    containerRef.current?.focus();
  }, []);

  const injectSpace = () => {
    // Manually inject Space as the key since macOS NSPanel filters Space keydown events
    const current = capturedRef.current;
    const withSpace: CapturedKeys = { ...current, key: "Space" };
    capturedRef.current = withSpace;

    const hasModifier = withSpace.meta || withSpace.ctrl || withSpace.alt || withSpace.shift;
    if (hasModifier) {
      setDisplayText(keysToDisplayString(withSpace));
      onCapture(keysToTauriString(withSpace));
    } else {
      setDisplayText("Hold a modifier key first, then tap Space");
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div
        ref={containerRef}
        tabIndex={-1}
        className={[
          "rounded-lg",
          "border-2",
          "border-white/40",
          "bg-black/40",
          "px-4 py-3",
          "text-center",
          "text-white/80",
          "text-sm",
          "outline-none",
          "cursor-default",
          "focus:border-white/70",
          "focus:bg-black/60",
          "transition-colors",
        ].join(" ")}
      >
        {displayText}
      </div>
      <div className="flex items-center justify-between">
        <button
          type="button"
          onMouseDown={(e) => e.preventDefault()}
          onClick={injectSpace}
          className="px-2.5 py-1 rounded-md text-xs text-white/40 hover:text-white/70 bg-white/5 hover:bg-white/10 border border-white/10 transition-colors cursor-default"
        >
          Space
        </button>
        <button
          type="button"
          onClick={onCancel}
          className="px-3 py-1.5 rounded-md text-xs text-white/60 hover:text-white/90 hover:bg-white/10 transition-colors cursor-default"
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
