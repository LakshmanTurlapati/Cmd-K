import { useOverlayStore } from "@/store";
import { AccountTab } from "./AccountTab";
import { ModelTab } from "./ModelTab";
import { PreferencesTab } from "./PreferencesTab";

const TABS = [
  { id: "account", label: "Account" },
  { id: "model", label: "Model" },
  { id: "preferences", label: "Preferences" },
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
    </div>
  );
}
