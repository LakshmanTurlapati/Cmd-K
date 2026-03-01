import { Store } from "@tauri-apps/plugin-store";
import { invoke } from "@tauri-apps/api/core";
import { useOverlayStore } from "@/store";
import { Trash2 } from "lucide-react";
import { HotkeyConfig } from "@/components/HotkeyConfig";

export function PreferencesTab() {
  const turnLimit = useOverlayStore((state) => state.turnLimit);
  const setTurnLimit = useOverlayStore((state) => state.setTurnLimit);

  const handleTurnLimitChange = async (value: number) => {
    setTurnLimit(value);
    try {
      const store = await Store.load("settings.json");
      await store.set("turnLimit", value);
      await store.save();
    } catch (err) {
      console.error("[preferences] Failed to persist turnLimit:", err);
    }
  };

  const handleClearHistory = async () => {
    try {
      await invoke("clear_all_history");
      // Reset frontend state
      useOverlayStore.getState().setWindowHistory([]);
      useOverlayStore.getState().setTurnHistory([]);
    } catch (err) {
      console.error("[preferences] Failed to clear history:", err);
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <p className="text-white/40 text-xs uppercase tracking-wider">
        Keyboard Shortcut
      </p>
      <HotkeyConfig />

      {/* AI Memory section */}
      <p className="text-white/40 text-xs uppercase tracking-wider mt-3">
        AI Memory
      </p>
      <div className="flex flex-col gap-2">
        <div className="flex items-center justify-between">
          <span className="text-white/70 text-xs">Conversation memory</span>
          <span className="text-white/40 text-xs font-mono">{turnLimit} turns</span>
        </div>
        <input
          type="range"
          min={5}
          max={50}
          value={turnLimit}
          onChange={(e) => handleTurnLimitChange(Number(e.target.value))}
          className="w-full accent-blue-500 h-1"
        />
        <p className="text-white/30 text-xs">
          How many prior exchanges the AI remembers per window
        </p>
      </div>

      {/* Clear history section */}
      <div className="mt-1">
        <button
          type="button"
          onClick={handleClearHistory}
          className="flex items-center gap-2 text-white/60 hover:text-red-400/80 text-xs transition-colors cursor-default bg-transparent border-none p-0"
        >
          <Trash2 size={12} />
          Clear conversation history
        </button>
        <p className="text-white/30 text-xs mt-1">
          Clears all AI context and command history for all windows
        </p>
      </div>
    </div>
  );
}
