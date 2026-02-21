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

    const balanced = availableModels.find(
      (m) => m.label === "Balanced" || m.label === "Recommended"
    );
    const defaultModel = balanced ?? availableModels[0];
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

  const handleChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    setSelectedModel(value);
    await persistModel(value);
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1.5">
        <p className="text-white/40 text-xs uppercase tracking-wider">
          Grok Model
        </p>

        {hasModels ? (
          <select
            value={selectedModel ?? ""}
            onChange={handleChange}
            className={[
              "w-full bg-white/8 border border-white/10 rounded-lg",
              "px-3 py-2 text-sm text-white cursor-pointer",
              "focus:outline-none focus:border-white/25 transition-colors",
            ].join(" ")}
          >
            {availableModels.map((model) => (
              <option key={model.id} value={model.id}>
                {model.id}
                {model.label ? ` (${model.label})` : ""}
              </option>
            ))}
          </select>
        ) : (
          <div className="flex flex-col gap-2">
            <select
              disabled
              className={[
                "w-full bg-white/8 border border-white/10 rounded-lg",
                "px-3 py-2 text-sm text-white/30 cursor-not-allowed",
                "focus:outline-none transition-colors",
              ].join(" ")}
            >
              <option value="">No models available</option>
            </select>
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
