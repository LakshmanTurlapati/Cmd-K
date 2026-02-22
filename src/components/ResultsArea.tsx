import { useOverlayStore } from "@/store";

export function ResultsArea() {
  const showApiWarning = useOverlayStore((state) => state.showApiWarning);
  const submitted = useOverlayStore((state) => state.submitted);
  const openSettings = useOverlayStore((state) => state.openSettings);

  if (!submitted) {
    return null;
  }

  if (showApiWarning) {
    return (
      <div className="border-t border-white/10 pt-3 mt-1">
        <p className="text-white/50 text-xs text-center">
          API not configured.{" "}
          <button
            type="button"
            className="text-white/70 underline underline-offset-2 hover:text-white/90 transition-colors cursor-pointer bg-transparent border-none p-0"
            onClick={() => openSettings("account")}
          >
            Set up in Settings
          </button>
        </p>
      </div>
    );
  }

  // Phase 4 will replace this with actual AI responses
  return (
    <div className="border-t border-white/10 pt-3 mt-1">
      <p className="text-white/30 text-xs text-center">
        AI responses coming soon
      </p>
    </div>
  );
}
