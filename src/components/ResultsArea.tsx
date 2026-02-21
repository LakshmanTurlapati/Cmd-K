import { useOverlayStore } from "@/store";

export function ResultsArea() {
  const showApiWarning = useOverlayStore((state) => state.showApiWarning);
  const submitted = useOverlayStore((state) => state.submitted);

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
            onClick={() => {
              // Placeholder for Phase 2 settings navigation
              console.log("Open settings requested");
            }}
          >
            Set up in Settings
          </button>
        </p>
      </div>
    );
  }

  // Placeholder for Phase 4 AI output
  return <div className="min-h-[4px]" />;
}
