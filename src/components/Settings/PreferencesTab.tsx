import { HotkeyConfig } from "@/components/HotkeyConfig";

export function PreferencesTab() {
  return (
    <div className="flex flex-col gap-3">
      <p className="text-white/40 text-xs uppercase tracking-wider">
        Keyboard Shortcut
      </p>
      <HotkeyConfig />
    </div>
  );
}
