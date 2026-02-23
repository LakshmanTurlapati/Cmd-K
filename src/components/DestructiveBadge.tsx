import { Tooltip } from "radix-ui";
import { invoke, Channel } from "@tauri-apps/api/core";
import { useOverlayStore } from "@/store";
import { useEffect, useState } from "react";

export function DestructiveBadge() {
  const streamingText = useOverlayStore((s) => s.streamingText);
  const destructiveExplanation = useOverlayStore((s) => s.destructiveExplanation);
  const dismissDestructiveBadge = useOverlayStore((s) => s.dismissDestructiveBadge);
  const setDestructiveExplanation = useOverlayStore((s) => s.setDestructiveExplanation);
  const selectedModel = useOverlayStore((s) => s.selectedModel);

  const [visible, setVisible] = useState(false);

  // Trigger fade-in on mount
  useEffect(() => {
    const id = setTimeout(() => setVisible(true), 0);
    return () => clearTimeout(id);
  }, []);

  // Eagerly load explanation on mount
  useEffect(() => {
    const ch = new Channel<string>();
    ch.onmessage = (explanation: string) => {
      setDestructiveExplanation(explanation);
    };

    invoke("get_destructive_explanation", {
      command: streamingText,
      model: selectedModel ?? "grok-3-mini",
      onResult: ch,
    }).catch(() => {
      setDestructiveExplanation("This command makes irreversible changes.");
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <Tooltip.Provider delayDuration={200}>
      <Tooltip.Root>
        <Tooltip.Trigger asChild>
          <span
            className={[
              "text-[11px] font-mono px-1.5 py-0.5 rounded",
              "bg-red-500/20 text-red-400 border border-red-500/30 ml-auto",
              "cursor-pointer transition-opacity duration-200",
              visible ? "opacity-100" : "opacity-0",
            ].join(" ")}
            onClick={dismissDestructiveBadge}
          >
            Destructive
          </span>
        </Tooltip.Trigger>
        <Tooltip.Portal>
          <Tooltip.Content
            side="top"
            sideOffset={4}
            className="max-w-[220px] text-[11px] text-white/80 bg-black/80 border border-white/10 rounded px-2 py-1.5 font-sans shadow-lg z-50"
          >
            {destructiveExplanation !== null ? (
              destructiveExplanation
            ) : (
              <span className="inline-block w-3 h-3 border border-white/30 border-t-white/70 rounded-full animate-spin" />
            )}
            <Tooltip.Arrow className="fill-black/80" />
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip.Root>
    </Tooltip.Provider>
  );
}
