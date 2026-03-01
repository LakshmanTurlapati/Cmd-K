import { invoke } from "@tauri-apps/api/core";
import { useOverlayStore } from "@/store";
import { AccountTab } from "./AccountTab";
import { ModelTab } from "./ModelTab";
import { PreferencesTab } from "./PreferencesTab";
import { AdvancedTab } from "./AdvancedTab";

const TABS = [
  { id: "account", label: "Account" },
  { id: "model", label: "Model" },
  { id: "preferences", label: "Preferences" },
  { id: "advanced", label: "Advanced" },
] as const;

export function SettingsPanel() {
  const settingsTab = useOverlayStore((s) => s.settingsTab);
  const setSettingsTab = useOverlayStore((s) => s.setSettingsTab);
  const setMode = useOverlayStore((s) => s.setMode);

  return (
    <div className="flex flex-col gap-0 w-full">
      {/* Header row: back button + title */}
      <div className="flex items-center mb-3">
        <button
          type="button"
          onClick={() => setMode("command")}
          className="text-white/40 hover:text-white/70 transition-colors cursor-default p-0 mr-2 text-sm leading-none"
          aria-label="Back to command mode"
        >
          &#8592;
        </button>
        <span className="text-white/70 text-sm font-medium flex-1 text-center pr-4">
          Settings
        </span>
      </div>

      {/* Tab bar */}
      <div className="flex border-b border-white/10 mb-3">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            type="button"
            onClick={() => setSettingsTab(tab.id)}
            className={[
              "px-3 py-2 text-xs font-medium transition-colors cursor-default",
              settingsTab === tab.id
                ? "border-b-2 border-white/70 text-white"
                : "text-white/40 hover:text-white/60",
            ].join(" ")}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      {settingsTab === "account" && <AccountTab />}
      {settingsTab === "model" && <ModelTab />}
      {settingsTab === "preferences" && <PreferencesTab />}
      {settingsTab === "advanced" && <AdvancedTab />}

      {/* Footer */}
      <div className="flex flex-col items-center gap-1 mt-6 pt-3 border-t border-white/5">
        <img src="/K-white.png" alt="CMD+K" className="h-12 opacity-25" />
        <span className="text-white/25 text-[10px]">
          v{__APP_VERSION__} | Made by Lakshman Turlapati
        </span>
        <div className="flex gap-3 mt-1">
          <button
            onClick={() => invoke("open_url", { url: "https://www.cmd-k.site" })}
            className="text-white/30 hover:text-white/50 text-[10px] cursor-default transition-colors"
          >
            About
          </button>
          <span className="text-white/10">|</span>
          <button
            onClick={() => invoke("open_url", { url: "https://www.cmd-k.site/privacy" })}
            className="text-white/30 hover:text-white/50 text-[10px] cursor-default transition-colors"
          >
            Privacy
          </button>
        </div>
      </div>
    </div>
  );
}
