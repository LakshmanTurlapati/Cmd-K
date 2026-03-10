import { useEffect } from "react";
import { Store } from "@tauri-apps/plugin-store";
import { Loader2 } from "lucide-react";
import { useOverlayStore, PROVIDERS } from "@/store";

const TIER_ORDER = [
  { key: "fast", label: "Fast" },
  { key: "balanced", label: "Balanced" },
  { key: "capable", label: "Most Capable" },
] as const;

export function ModelTab() {
  const apiKeyStatus = useOverlayStore((s) => s.apiKeyStatus);
  const availableModels = useOverlayStore((s) => s.availableModels);
  const selectedModel = useOverlayStore((s) => s.selectedModel);
  const setSelectedModel = useOverlayStore((s) => s.setSelectedModel);
  const selectedProvider = useOverlayStore((s) => s.selectedProvider);
  const selectedModels = useOverlayStore((s) => s.selectedModels);
  const setSelectedModels = useOverlayStore((s) => s.setSelectedModels);

  const isEnabled =
    apiKeyStatus === "valid" && availableModels.length > 0;

  const providerName =
    PROVIDERS.find((p) => p.id === selectedProvider)?.name ?? selectedProvider;

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
    if (!isEnabled) return;

    // Check per-provider memory first
    const rememberedModel = selectedModels[selectedProvider];
    if (rememberedModel && availableModels.some((m) => m.id === rememberedModel)) {
      setSelectedModel(rememberedModel);
      return;
    }

    if (selectedModel && availableModels.some((m) => m.id === selectedModel)) return;

    const recommended = availableModels.find((m) => m.label === "Recommended")
      ?? availableModels.find((m) => m.label === "Balanced");
    const defaultModel = recommended ?? availableModels[0];
    if (defaultModel) {
      setSelectedModel(defaultModel.id);
      handleModelSelect(defaultModel.id);
    }
  }, [isEnabled, availableModels, selectedProvider]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleModelSelect = async (modelId: string) => {
    setSelectedModel(modelId);
    const provider = useOverlayStore.getState().selectedProvider;
    const currentMap = useOverlayStore.getState().selectedModels;
    const updatedMap = { ...currentMap, [provider]: modelId };
    setSelectedModels(updatedMap);
    try {
      const store = await Store.load("settings.json");
      await store.set("selectedModels", updatedMap);
      await store.set("selectedModel", modelId); // backward compat
      await store.save();
    } catch {
      // Non-fatal persistence failure
    }
  };

  const renderModelButton = (model: { id: string; label: string }) => (
    <button
      key={model.id}
      type="button"
      onClick={() => handleModelSelect(model.id)}
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
  );

  return (
    <div className="flex flex-col gap-4">
      {/* Model selection */}
      <div className="flex flex-col gap-1.5">
        <p className="text-white/40 text-xs uppercase tracking-wider">
          {providerName} Model
        </p>
        {!isEnabled ? (
          <div className="w-full bg-white/8 border border-white/10 rounded-lg px-3 py-2 text-sm text-white/30">
            {apiKeyStatus === "validating" ? (
              <span className="flex items-center gap-2">
                <Loader2 size={14} className="animate-spin" />
                Validating...
              </span>
            ) : (
              "Validate API key first"
            )}
          </div>
        ) : (
          <div className="flex flex-col gap-1 max-h-64 overflow-y-auto scrollbar-thin pr-1">
            {/* Tier sections */}
            {TIER_ORDER.map((tier) => {
              const tierModels = availableModels.filter((m) => m.tier === tier.key);
              if (tierModels.length === 0) return null;
              return (
                <div key={tier.key}>
                  <p className="text-white/30 text-xs uppercase tracking-wider mb-1 mt-2">
                    {tier.label}
                  </p>
                  {tierModels.map(renderModelButton)}
                </div>
              );
            })}

            {/* All Models section */}
            <p className="text-white/30 text-xs uppercase tracking-wider mb-1 mt-3">
              All Models
            </p>
            {availableModels.map(renderModelButton)}
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
