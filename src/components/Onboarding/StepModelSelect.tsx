import { useEffect } from "react";
import { Store } from "@tauri-apps/plugin-store";
import { useOverlayStore } from "@/store";

interface StepModelSelectProps {
  onNext: () => void;
}

export function StepModelSelect({ onNext }: StepModelSelectProps) {
  const apiKeyStatus = useOverlayStore((s) => s.apiKeyStatus);
  const availableModels = useOverlayStore((s) => s.availableModels);
  const selectedModel = useOverlayStore((s) => s.selectedModel);
  const setSelectedModel = useOverlayStore((s) => s.setSelectedModel);

  const hasModels = apiKeyStatus === "valid" && availableModels.length > 0;

  // Auto-select balanced default when models load
  useEffect(() => {
    if (!hasModels || selectedModel) return;

    const recommended = availableModels.find((m) => m.label === "Recommended")
      ?? availableModels.find((m) => m.label === "Balanced");
    const defaultModel = recommended ?? availableModels[0];
    if (defaultModel) {
      setSelectedModel(defaultModel.id);
      persistModel(defaultModel.id);
    }
  }, [hasModels, availableModels]); // eslint-disable-line react-hooks/exhaustive-deps

  const persistModel = async (modelId: string) => {
    try {
      const store = await Store.load("settings.json");
      await store.set("selectedModel", modelId);
      await store.save();
    } catch {
      // Non-fatal
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1.5">
        <p className="text-white/40 text-xs uppercase tracking-wider">
          xAI Model
        </p>

        {hasModels ? (
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
        ) : (
          <div className="flex flex-col gap-2">
            <div className="w-full bg-white/8 border border-white/10 rounded-lg px-3 py-2 text-sm text-white/30">
              No models available
            </div>
            <p className="text-white/30 text-xs">
              Configure API key first to select a model
            </p>
          </div>
        )}
      </div>

      <button
        type="button"
        onClick={onNext}
        className={[
          "w-full px-3 py-2 rounded-lg text-sm font-medium",
          "bg-white/10 hover:bg-white/15 border border-white/15",
          "text-white transition-colors cursor-default",
        ].join(" ")}
      >
        Next
      </button>
    </div>
  );
}
