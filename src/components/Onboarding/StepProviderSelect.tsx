import { useState } from "react";
import { Store } from "@tauri-apps/plugin-store";
import { useOverlayStore, PROVIDERS } from "@/store";
import { ProviderIcon } from "@/components/icons/ProviderIcon";

interface StepProviderSelectProps {
  onNext: () => void;
}

export function StepProviderSelect({ onNext }: StepProviderSelectProps) {
  const setSelectedProvider = useOverlayStore((s) => s.setSelectedProvider);
  const [chosen, setChosen] = useState<string | null>(null);

  const handleSelect = async (providerId: string) => {
    setChosen(providerId);
    setSelectedProvider(providerId);

    try {
      const store = await Store.load("settings.json");
      await store.set("selectedProvider", providerId);
      await store.save();
    } catch {
      // Non-fatal: provider will default to xai on next launch
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-col gap-1.5">
        <p className="text-white/60 text-sm">
          Choose your AI provider to get started.
        </p>

        <div className="flex flex-col gap-1.5 mt-1">
          {PROVIDERS.map((provider) => {
            const isSelected = chosen === provider.id;
            return (
              <button
                key={provider.id}
                type="button"
                onClick={() => handleSelect(provider.id)}
                className={[
                  "flex items-center gap-3 px-3 py-2 rounded-lg text-sm",
                  "border transition-colors cursor-default",
                  isSelected
                    ? "bg-white/15 text-white border-white/20"
                    : "bg-white/5 hover:bg-white/8 text-white/70 border-transparent",
                ].join(" ")}
              >
                <div className="w-8 h-8 rounded-full bg-white/10 flex items-center justify-center">
                  <ProviderIcon provider={provider.id} size={16} className="text-white/70" />
                </div>
                <span>{provider.name}</span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Next button */}
      <button
        type="button"
        onClick={onNext}
        disabled={!chosen}
        className={[
          "w-full px-3 py-2 rounded-lg text-sm font-medium",
          "border transition-colors cursor-default",
          chosen
            ? "bg-white/10 hover:bg-white/15 border-white/15 text-white"
            : "bg-white/5 border-white/8 text-white/30 cursor-not-allowed",
        ].join(" ")}
      >
        Next
      </button>
    </div>
  );
}
