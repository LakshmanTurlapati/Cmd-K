import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Store } from "@tauri-apps/plugin-store";
import { useOverlayStore } from "@/store";
import { HotkeyRecorder } from "./HotkeyRecorder";

interface Preset {
  label: string;
  shortcut: string;
  note?: string;
}

const PRESETS: Preset[] = [
  { label: "Cmd + K", shortcut: "Super+KeyK", note: "default" },
  { label: "Cmd + Shift + K", shortcut: "Super+Shift+KeyK" },
  { label: "Ctrl + Space", shortcut: "Control+Space" },
  { label: "Option + Space", shortcut: "Alt+Space" },
  { label: "Cmd + Shift + Space", shortcut: "Super+Shift+Space" },
];

type Status = "idle" | "applying" | "success" | { error: string };

export function HotkeyConfig() {
  const closeHotkeyConfig = useOverlayStore((s) => s.closeHotkeyConfig);
  const currentHotkey = useOverlayStore((s) => s.currentHotkey);
  const setCurrentHotkey = useOverlayStore((s) => s.setCurrentHotkey);

  const [selected, setSelected] = useState<string>(currentHotkey);
  const [showRecorder, setShowRecorder] = useState(false);
  const [status, setStatus] = useState<Status>("idle");

  const handleCapture = (shortcut: string) => {
    setSelected(shortcut);
    setShowRecorder(false);
  };

  const handleApply = async () => {
    setStatus("applying");
    try {
      const result = await invoke<string | null>("register_hotkey", {
        shortcutStr: selected,
      });

      if (result && result.length > 0) {
        // Rust returned an error string
        setStatus({
          error: `Could not register ${selected}. It may conflict with another app. Try a different shortcut.`,
        });
        return;
      }

      // Success: persist to store
      const store = await Store.load("settings.json");
      await store.set("hotkey", selected);
      await store.save();

      // Update Zustand state
      setCurrentHotkey(selected);

      setStatus("success");
      setTimeout(() => {
        closeHotkeyConfig();
      }, 500);
    } catch (err) {
      const msg = typeof err === "string" ? err : "Unknown error";
      setStatus({
        error: `Could not register ${selected}. ${msg} Try a different shortcut.`,
      });
    }
  };

  const handleCancel = () => {
    closeHotkeyConfig();
  };

  const tauriToDisplay = (shortcut: string): string => {
    return shortcut
      .split("+")
      .map((part) => {
        if (part === "Super") return "Cmd";
        if (part === "Control") return "Ctrl";
        if (part === "Alt") return "Option";
        if (part.startsWith("Key")) return part.slice(3);
        if (part.startsWith("Digit")) return part.slice(5);
        return part;
      })
      .join(" + ");
  };

  const currentDisplayLabel =
    PRESETS.find((p) => p.shortcut === currentHotkey)?.label ?? tauriToDisplay(currentHotkey);

  const selectedDisplayLabel =
    PRESETS.find((p) => p.shortcut === selected)?.label ?? tauriToDisplay(selected);

  return (
      <div className="flex flex-col gap-3">
        {/* Header */}
        <div className="flex flex-col gap-1">
          <h2 className="text-white text-base font-semibold tracking-tight">
            Change Hotkey
          </h2>
          <p className="text-white/50 text-xs">
            Current: {currentDisplayLabel}
          </p>
        </div>

        {/* Preset list */}
        <div className="flex flex-col gap-1.5">
          <p className="text-white/40 text-xs uppercase tracking-wider mb-1">
            Presets
          </p>
          {PRESETS.map((preset) => (
            <button
              key={preset.shortcut}
              type="button"
              onClick={() => {
                setSelected(preset.shortcut);
                setShowRecorder(false);
              }}
              className={[
                "flex items-center justify-between",
                "rounded-lg px-3 py-2",
                "text-sm text-left",
                "cursor-default transition-colors",
                selected === preset.shortcut
                  ? "bg-white/15 text-white"
                  : "text-white/70 hover:bg-white/8 hover:text-white",
              ].join(" ")}
            >
              <span>{preset.label}</span>
              {preset.note && (
                <span className="text-white/30 text-xs">{preset.note}</span>
              )}
              {selected === preset.shortcut && selected !== currentHotkey && (
                <span className="text-white/60 text-xs ml-auto mr-0">
                  selected
                </span>
              )}
            </button>
          ))}
        </div>

        {/* Custom recorder section */}
        <div className="flex flex-col gap-2">
          {/* Show the current custom shortcut if it's not a preset */}
          {!PRESETS.some((p) => p.shortcut === selected) && !showRecorder && selected && (
            <button
              type="button"
              onClick={() => setShowRecorder(true)}
              className="flex items-center justify-between rounded-lg px-3 py-2 text-sm text-left cursor-default transition-colors bg-white/15 text-white"
            >
              <span>{selectedDisplayLabel}</span>
              <span className="text-white/30 text-xs">custom</span>
            </button>
          )}
          {!showRecorder ? (
            <button
              type="button"
              onClick={() => setShowRecorder(true)}
              className="flex items-center gap-2 text-sm text-white/60 hover:text-white/90 hover:bg-white/8 rounded-lg px-3 py-2 transition-colors cursor-default text-left"
            >
              <span className="text-white/40">+</span>
              Record Custom Shortcut
            </button>
          ) : (
            <div className="flex flex-col gap-2">
              <p className="text-white/40 text-xs uppercase tracking-wider">
                Custom
              </p>
              <HotkeyRecorder
                onCapture={handleCapture}
                onCancel={() => setShowRecorder(false)}
              />
            </div>
          )}
        </div>

        {/* Status area */}
        <div className="min-h-[20px] text-xs">
          {status === "applying" && (
            <span className="text-white/50">Applying...</span>
          )}
          {status === "success" && (
            <span className="text-green-400/80">Hotkey updated successfully</span>
          )}
          {typeof status === "object" && "error" in status && (
            <span className="text-red-400/80">{status.error}</span>
          )}
          {status === "idle" && selected !== currentHotkey && (
            <span className="text-white/30">Will apply: {selectedDisplayLabel}</span>
          )}
        </div>

        {/* Action buttons */}
        <div className="flex gap-2 justify-end pt-1">
          <button
            type="button"
            onClick={handleCancel}
            className="px-4 py-1.5 rounded-lg text-sm text-white/60 hover:text-white/90 hover:bg-white/8 transition-colors cursor-default"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={handleApply}
            disabled={status === "applying" || status === "success"}
            className={[
              "px-4 py-1.5 rounded-lg text-sm transition-colors cursor-default",
              status === "applying" || status === "success"
                ? "bg-white/10 text-white/30"
                : "bg-white/20 text-white hover:bg-white/30",
            ].join(" ")}
          >
            Apply
          </button>
        </div>
    </div>
  );
}
