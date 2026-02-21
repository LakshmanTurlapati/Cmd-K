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

    const recommended = availableModels.find((m) => m.label === "Recommended")
      ?? availableModels.find((m) => m.label === "Balanced");
    const defaultModel = recommended ?? availableModels[0];
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

  return (
    <div className="flex flex-col gap-4">
      {/* Model selection */}
      <div className="flex flex-col gap-1.5">
        <p className="text-white/40 text-xs uppercase tracking-wider">
          xAI Model
        </p>
        {!isEnabled ? (
          <div className="w-full bg-white/8 border border-white/10 rounded-lg px-3 py-2 text-sm text-white/30">
            Validate API key first
          </div>
        ) : (
          <div className="flex flex-col gap-1">
            {availableModels.map((model) => (
              <button
                key={model.id}
                type="button"
                onClick={async () => {
                  setSelectedModel(model.id);
                  await persistModel(model.id);
                }}
                className={[
                  "flex items-center justify-between w-full rounded-lg px-3 py-2 text-sm text-left",
                  "cursor-default transition-colors",
                  selectedModel === model.id
                    ? "bg-white/15 text-white"
                    : "text-white/70 hover:bg-white/8 hover:text-white",
                ].join(" ")}
              >
                <span>{model.id}</span>
                {model.label && (
                  <span className="text-white/30 text-xs">{model.label}</span>
                )}
              </button>
            ))}
          </div>
        )}
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
