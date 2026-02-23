import { useState } from "react";
import { useOverlayStore } from "@/store";

export function ResultsArea() {
  const streamingText = useOverlayStore((state) => state.streamingText);
  const isStreaming = useOverlayStore((state) => state.isStreaming);
  const displayMode = useOverlayStore((state) => state.displayMode);
  const streamError = useOverlayStore((state) => state.streamError);
  const openSettings = useOverlayStore((state) => state.openSettings);

  const [copiedVisible, setCopiedVisible] = useState(false);

  // Only render when in streaming or result mode
  if (displayMode !== "streaming" && displayMode !== "result") {
    return null;
  }

  const handleClick = () => {
    // Click-to-copy only available in result mode (not during streaming)
    if (displayMode !== "result" || !streamingText) return;
    navigator.clipboard.writeText(streamingText).then(() => {
      setCopiedVisible(true);
      setTimeout(() => setCopiedVisible(false), 1500);
    }).catch((err) => {
      console.error("[ResultsArea] clipboard copy failed:", err);
    });
  };

  return (
    <div className="border-t border-white/10 pt-3 mt-1">
      {streamError ? (
        <div className="text-red-400/70 text-xs font-mono">
          {streamError}
          {streamError.includes("API key") && (
            <>
              {" "}
              <button
                type="button"
                className="text-white/50 underline underline-offset-2 hover:text-white/70 transition-colors cursor-pointer bg-transparent border-none p-0"
                onClick={() => openSettings("account")}
              >
                Open Settings
              </button>
            </>
          )}
        </div>
      ) : (
        <div className="relative">
          <div
            className={[
              "max-h-[60vh]",
              "overflow-y-auto",
              displayMode === "result" ? "cursor-pointer" : "",
              displayMode === "result" ? "hover:bg-white/5" : "",
              "transition-colors",
              "rounded",
            ]
              .filter(Boolean)
              .join(" ")}
            onClick={handleClick}
          >
            <pre className="font-mono text-sm text-white/90 whitespace-pre-wrap break-words m-0">
              {streamingText}
              {isStreaming && (
                <span className="inline-block w-[0.6em] h-[1.1em] bg-white/80 animate-pulse align-text-bottom ml-px" />
              )}
            </pre>
          </div>
          {copiedVisible && (
            <span className="absolute bottom-0 right-0 text-[10px] text-white/50 pointer-events-none">
              Copied to clipboard
            </span>
          )}
        </div>
      )}
    </div>
  );
}
