import { useEffect } from "react";
import { Store } from "@tauri-apps/plugin-store";
import { useOverlayStore } from "@/store";

export function ModelTab() {
  const apiKeyStatus = useOverlayStore((s) => s.apiKeyStatus);
  const availableModels = useOverlayStore((s) => s.availableModels);
  const selectedModel = useOverlayStore((s) => s.selectedModel);
  const setSelectedModel = useOverlayStore((s) => s.setSelectedModel);

  const isEnabled =
    apiKeyStatus === "valid" && availableModels.length > 0;

  // On mount: load persisted selectedModel from settings.json
  useEffect(() => {
    const loadPersistedModel = async () => {
      try {
        const store = await Store.load("settings.json");
        const saved = await store.get<string>("selectedModel");
        if (saved) {
          setSelectedModel(saved);
        }
      } catch {
        // Non-fatal: leave selectedModel as null
      }
    };

    loadPersistedModel();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // Auto-select smart default when models load and none is selected
  useEffect(() => {
    if (!isEnabled || selectedModel) return;

    const balanced = availableModels.find(
      (m) => m.label === "Balanced" || m.label === "Recommended"
    );
    const defaultModel = balanced ?? availableModels[0];
    if (defaultModel) {
      setSelectedModel(defaultModel.id);
      persistModel(defaultModel.id);
    }
  }, [isEnabled, availableModels]); // eslint-disable-line react-hooks/exhaustive-deps

  const persistModel = async (modelId: string) => {
    try {
      const store = await Store.load("settings.json");
      await store.set("selectedModel", modelId);
      await store.save();
    } catch {
      // Non-fatal persistence failure
    }
  };

  const handleChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    setSelectedModel(value);
    await persistModel(value);
  };

  return (
    <div className="flex flex-col gap-4">
      {/* Model selection */}
      <div className="flex flex-col gap-1.5">
        <p className="text-white/40 text-xs uppercase tracking-wider">
          Grok Model
        </p>
        <select
          value={selectedModel ?? ""}
          onChange={handleChange}
          disabled={!isEnabled}
          className={[
            "w-full bg-white/8 border border-white/10 rounded-lg",
            "px-3 py-2 text-sm transition-colors",
            "focus:outline-none focus:border-white/25",
            isEnabled
              ? "text-white cursor-pointer"
              : "text-white/30 cursor-not-allowed",
          ].join(" ")}
        >
          {!isEnabled && (
            <option value="" disabled>
              Validate API key first
            </option>
          )}
          {isEnabled &&
            availableModels.map((model) => (
              <option key={model.id} value={model.id}>
                {model.id}
                {model.label ? ` (${model.label})` : ""}
              </option>
            ))}
        </select>
      </div>

      {/* Usage placeholder */}
      <div className="flex flex-col gap-1.5">
        <p className="text-white/40 text-xs uppercase tracking-wider">
          Estimated Cost
        </p>
        <p className="text-white/30 text-xs">No usage recorded yet</p>
        <p className="text-white/20 text-xs">
          (Available after AI commands are enabled)
        </p>
      </div>
    </div>
  );
}
