import { useEffect, useState } from "react";
import { Store } from "@tauri-apps/plugin-store";
import { invoke } from "@tauri-apps/api/core";
import { Loader2 } from "lucide-react";
import { useOverlayStore, PROVIDERS, ModelWithMeta } from "@/store";

interface UsageStatEntry {
  provider: string;
  model: string;
  input_tokens: number;
  output_tokens: number;
  query_count: number;
  estimated_cost: number | null;
  pricing_available: boolean;
}

interface UsageStatsResponse {
  entries: UsageStatEntry[];
  session_total_cost: number | null;
  query_costs: (number | null)[];
}

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
  const setModels = useOverlayStore((s) => s.setModels);

  const [usageStats, setUsageStats] = useState<UsageStatsResponse | null>(null);

  const currentProv = PROVIDERS.find((p) => p.id === selectedProvider);
  const isLocal = currentProv?.local ?? false;

  const fetchUsage = async () => {
    try {
      const stats = await invoke<UsageStatsResponse>("get_usage_stats");
      setUsageStats(stats);
    } catch {
      // Non-fatal: leave display empty
    }
  };

  // Fetch usage stats on mount (tab open)
  useEffect(() => {
    fetchUsage();
  }, []);

  // Refresh model list on mount for local providers (catches new model installs/unloads)
  useEffect(() => {
    if (!isLocal) return;
    const refreshModels = async () => {
      try {
        const models = await invoke<ModelWithMeta[]>(
          "fetch_models",
          { provider: selectedProvider, apiKey: "" }
        );
        setModels(models);
      } catch {
        // Graceful degradation -- keep existing model list
      }
    };
    refreshModels();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps
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
                {isLocal ? "Checking server..." : "Validating..."}
              </span>
            ) : isLocal && apiKeyStatus === "valid" ? (
              "No models found — check that models are loaded"
            ) : isLocal ? (
              "Connect to server first"
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

      {/* Session cost display */}
      <div className="flex flex-col gap-1.5">
        <p className="text-white/40 text-xs uppercase tracking-wider">
          Estimated Cost
        </p>
        {(() => {
          const totalQueries = usageStats?.entries.reduce((s, e) => s + e.query_count, 0) ?? 0;
          if (!usageStats || totalQueries === 0) {
            return (
              <p className="text-white/30 text-xs">No usage recorded yet</p>
            );
          }

          const totalInput = usageStats.entries.reduce((s, e) => s + e.input_tokens, 0);
          const totalOutput = usageStats.entries.reduce((s, e) => s + e.output_tokens, 0);
          const allUnpriced = usageStats.entries.every((e) => !e.pricing_available);
          const someUnpriced = usageStats.entries.some((e) => !e.pricing_available) && !allUnpriced;
          const allUnpricedAreLocal = allUnpriced && usageStats.entries.every((e) => {
            const prov = PROVIDERS.find((p) => p.name === e.provider);
            return prov?.local ?? false;
          });
          const unpricedAreLocal = someUnpriced && usageStats.entries
            .filter((e) => !e.pricing_available)
            .every((e) => {
              const prov = PROVIDERS.find((p) => p.name === e.provider);
              return prov?.local ?? false;
            });

          const formatCost = (cost: number): string => {
            if (cost >= 1) return `$${cost.toFixed(2)}`;
            return `$${cost.toFixed(4)}`;
          };

          const tokenStr = `${totalInput.toLocaleString()} in / ${totalOutput.toLocaleString()} out`;

          const handleReset = async () => {
            await invoke("reset_usage");
            fetchUsage();
          };

          // Sparkline data: filter to only render if there are meaningful costs
          const queryCosts = usageStats.query_costs;
          const hasCosts = queryCosts.some((c) => c !== null && c > 0);

          return (
            <>
              <div className="flex items-center gap-2">
                <p className="text-white/70 text-xs">
                  {allUnpricedAreLocal ? (
                    <span>
                      Free (local)
                      <span className="text-white/40"> &mdash; {tokenStr}</span>
                    </span>
                  ) : allUnpriced ? (
                    <span title="Pricing unavailable for this model">
                      {tokenStr} &mdash; $&mdash;
                    </span>
                  ) : (
                    <>
                      {formatCost(usageStats.session_total_cost ?? 0)}
                      {someUnpriced && !unpricedAreLocal && <span>*</span>}
                      <span className="text-white/40"> &mdash; {tokenStr}</span>
                    </>
                  )}
                </p>
                <button
                  type="button"
                  onClick={handleReset}
                  className="text-white/40 text-xs hover:text-white/60 cursor-default"
                >
                  Reset
                </button>
              </div>
              {someUnpriced && !unpricedAreLocal && (
                <p className="text-white/20 text-xs">*excludes queries without pricing</p>
              )}
              {hasCosts && (
                <div className="flex items-end gap-px h-8">
                  {queryCosts.map((cost, i) => {
                    const maxCost = Math.max(
                      ...queryCosts.map((c) => c ?? 0)
                    );
                    const height =
                      cost !== null && cost > 0 && maxCost > 0
                        ? Math.max(1, (cost / maxCost) * 32)
                        : 0;
                    return (
                      <div
                        key={i}
                        className="bg-white/20 rounded-none"
                        style={{
                          flex: 1,
                          maxWidth: 6,
                          minWidth: 1,
                          height: `${height}px`,
                        }}
                      />
                    );
                  })}
                </div>
              )}
            </>
          );
        })()}
      </div>
    </div>
  );
}
