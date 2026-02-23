import { Store } from "@tauri-apps/plugin-store";
import { HotkeyConfig } from "@/components/HotkeyConfig";
import { useOverlayStore } from "@/store";

export function PreferencesTab() {
  const destructiveDetectionEnabled = useOverlayStore(
    (state) => state.destructiveDetectionEnabled
  );
  const setDestructiveDetectionEnabled = useOverlayStore(
    (state) => state.setDestructiveDetectionEnabled
  );

  const handleToggleDestructive = async () => {
    const newValue = !destructiveDetectionEnabled;
    setDestructiveDetectionEnabled(newValue);
    try {
      const store = await Store.load("settings.json");
      await store.set("destructiveDetectionEnabled", newValue);
      await store.save();
    } catch (err) {
      console.error("[preferences] Failed to persist destructiveDetectionEnabled:", err);
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <p className="text-white/40 text-xs uppercase tracking-wider">
        Keyboard Shortcut
      </p>
      <HotkeyConfig />

      <p className="text-white/40 text-xs uppercase tracking-wider mt-2">
        Safety
      </p>
      <div className="flex items-center justify-between">
        <span className="text-white/70 text-xs">Destructive command warnings</span>
        <button
          aria-label="Toggle destructive command detection"
          onClick={handleToggleDestructive}
          className={`relative w-8 h-4 rounded-full transition-colors duration-200 ${
            destructiveDetectionEnabled ? "bg-red-500/60" : "bg-white/10"
          }`}
        >
          <div
            className={`absolute top-0.5 w-3 h-3 rounded-full bg-white transition-transform duration-200 ${
              destructiveDetectionEnabled ? "translate-x-4" : "translate-x-0.5"
            }`}
          />
        </button>
      </div>
    </div>
  );
}
