import { useOverlayStore } from "@/store";

interface StepDoneProps {
  onComplete: () => void;
}

export function StepDone({ onComplete }: StepDoneProps) {
  const currentHotkey = useOverlayStore((s) => s.currentHotkey);
  const selectedModel = useOverlayStore((s) => s.selectedModel);

  const formatHotkey = (hotkey: string) => {
    return hotkey
      .replace("Super", "Cmd")
      .replace("Key", "")
      .split("+")
      .join(" + ");
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col items-center gap-2 py-2">
        <h2 className="text-white text-lg font-semibold">You are all set!</h2>
        <p className="text-white/50 text-sm text-center">
          CMD+K is ready to use. Here is your configuration summary.
        </p>
      </div>

      <div className="flex flex-col gap-2 bg-white/5 rounded-lg p-3 border border-white/8">
        <div className="flex items-center justify-between">
          <span className="text-white/40 text-xs uppercase tracking-wider">
            Hotkey
          </span>
          <span className="text-white/80 text-sm font-mono">
            {formatHotkey(currentHotkey)}
          </span>
        </div>
        <div className="h-px bg-white/8" />
        <div className="flex items-center justify-between">
          <span className="text-white/40 text-xs uppercase tracking-wider">
            Model
          </span>
          <span className="text-white/80 text-sm font-mono">
            {selectedModel ?? "Not configured"}
          </span>
        </div>
      </div>

      <button
        type="button"
        onClick={onComplete}
        className={[
          "w-full px-3 py-2.5 rounded-lg text-sm font-semibold",
          "bg-white/15 hover:bg-white/20 border border-white/20",
          "text-white transition-colors cursor-default",
        ].join(" ")}
      >
        Start using CMD+K
      </button>
    </div>
  );
}
