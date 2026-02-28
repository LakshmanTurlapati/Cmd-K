import { Store } from "@tauri-apps/plugin-store";
import { useOverlayStore } from "@/store";
import { RotateCcw } from "lucide-react";

export function AdvancedTab() {
  const destructiveDetectionEnabled = useOverlayStore(
    (state) => state.destructiveDetectionEnabled
  );
  const setDestructiveDetectionEnabled = useOverlayStore(
    (state) => state.setDestructiveDetectionEnabled
  );

  const autoPasteEnabled = useOverlayStore(
    (state) => state.autoPasteEnabled
  );
  const setAutoPasteEnabled = useOverlayStore(
    (state) => state.setAutoPasteEnabled
  );

  const handleToggleDestructive = async () => {
    const newValue = !destructiveDetectionEnabled;
    setDestructiveDetectionEnabled(newValue);
    try {
      const store = await Store.load("settings.json");
      await store.set("destructiveDetectionEnabled", newValue);
      await store.save();
    } catch (err) {
      console.error("[advanced] Failed to persist destructiveDetectionEnabled:", err);
    }
  };

  const handleToggleAutoPaste = async () => {
    const newValue = !autoPasteEnabled;
    setAutoPasteEnabled(newValue);
    try {
      const store = await Store.load("settings.json");
      await store.set("autoPasteEnabled", newValue);
      await store.save();
    } catch (err) {
      console.error("[advanced] Failed to persist autoPasteEnabled:", err);
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <p className="text-white/40 text-xs uppercase tracking-wider">
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

      <p className="text-white/40 text-xs uppercase tracking-wider mt-2">
        Terminal
      </p>
      <div className="flex items-center justify-between">
        <span className="text-white/70 text-xs">Auto-paste to terminal</span>
        <button
          aria-label="Toggle auto-paste to terminal"
          onClick={handleToggleAutoPaste}
          className={`relative w-8 h-4 rounded-full transition-colors duration-200 ${
            autoPasteEnabled ? "bg-blue-500/60" : "bg-white/10"
          }`}
        >
          <div
            className={`absolute top-0.5 w-3 h-3 rounded-full bg-white transition-transform duration-200 ${
              autoPasteEnabled ? "translate-x-4" : "translate-x-0.5"
            }`}
          />
        </button>
      </div>
      {!autoPasteEnabled && (
        <p className="text-amber-400/60 text-xs mt-1">Commands will not be pasted automatically</p>
      )}

      <p className="text-white/40 text-xs uppercase tracking-wider mt-2">
        Setup
      </p>
      <button
        type="button"
        onClick={async () => {
          try {
            const store = await Store.load("settings.json");
            await store.set("onboardingComplete", false);
            await store.set("onboardingStep", 0);
            await store.save();
          } catch (err) {
            console.error("[advanced] Failed to reset onboarding in store:", err);
          }
          const s = useOverlayStore.getState();
          s.setOnboardingComplete(false);
          s.setOnboardingStep(0);
          s.setMode("onboarding");
        }}
        className="flex items-center gap-2 text-white/60 hover:text-white/80 text-xs transition-colors cursor-default bg-transparent border-none p-0"
      >
        <RotateCcw size={12} />
        Re-run onboarding wizard
      </button>
    </div>
  );
}
