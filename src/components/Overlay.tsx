import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import * as Tooltip from "@radix-ui/react-tooltip";
import { ShieldAlert } from "lucide-react";
import { useOverlayStore, resolveBadge } from "@/store";
import { CommandInput } from "./CommandInput";
import { ResultsArea } from "./ResultsArea";
import { HotkeyConfig } from "./HotkeyConfig";
import { SettingsPanel } from "./Settings/SettingsPanel";
import { OnboardingWizard } from "./Onboarding/OnboardingWizard";
import { DestructiveBadge } from "./DestructiveBadge";

type AnimationPhase = "entering" | "visible" | "exiting" | "hidden";

interface OverlayProps {
  onSubmit: (value: string) => void;
}

export function Overlay({ onSubmit }: OverlayProps) {
  const visible = useOverlayStore((state) => state.visible);
  const hotkeyConfigOpen = useOverlayStore((state) => state.hotkeyConfigOpen);
  const mode = useOverlayStore((state) => state.mode);
  const displayMode = useOverlayStore((state) => state.displayMode);
  const appContext = useOverlayStore((s) => s.appContext);
  const isDetecting = useOverlayStore((s) => s.isDetectingContext);
  const accessibilityGranted = useOverlayStore((s) => s.accessibilityGranted);
  const isDestructive = useOverlayStore((s) => s.isDestructive);
  const destructiveDismissed = useOverlayStore((s) => s.destructiveDismissed);
  const badgeText = resolveBadge(appContext);
  const [animPhase, setAnimPhase] = useState<AnimationPhase>("hidden");

  useEffect(() => {
    if (visible) {
      setAnimPhase("entering");
    } else if (animPhase === "visible" || animPhase === "entering") {
      setAnimPhase("exiting");
    }
  }, [visible]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleAnimationEnd = () => {
    if (animPhase === "entering") {
      setAnimPhase("visible");
    } else if (animPhase === "exiting") {
      setAnimPhase("hidden");
    }
  };

  // Background polling: auto-hide accessibility badge when permission is granted
  useEffect(() => {
    if (accessibilityGranted) return;
    const intervalId = setInterval(async () => {
      try {
        const result = await invoke<boolean>("check_accessibility_permission");
        if (result) {
          useOverlayStore.getState().setAccessibilityGranted(true);
        }
      } catch {
        // Silent
      }
    }, 5000);
    return () => clearInterval(intervalId);
  }, [accessibilityGranted]);

  if (animPhase === "hidden") {
    return null;
  }

  const animClass =
    animPhase === "entering"
      ? "animate-[overlay-in_120ms_ease-out]"
      : animPhase === "exiting"
        ? "animate-[overlay-out_100ms_ease-in]"
        : "";

  const panelWidth =
    mode === "settings" || mode === "onboarding" ? "w-[380px]" : "w-[320px]";

  return (
    <div
      className={[
        panelWidth,
        "rounded-xl",
        "shadow-2xl",
        "bg-black/60",
        "border border-white/10",
        "p-4",
        "flex flex-col gap-2",
        animClass,
      ]
        .filter(Boolean)
        .join(" ")}
      onAnimationEnd={handleAnimationEnd}
    >
      {mode === "settings" ? (
        <SettingsPanel />
      ) : mode === "onboarding" ? (
        <OnboardingWizard />
      ) : (
        // command mode (default)
        <>
          {mode === "command" && !accessibilityGranted && !isDetecting && (
            <Tooltip.Provider delayDuration={300}>
              <Tooltip.Root>
                <Tooltip.Trigger asChild>
                  <button
                    type="button"
                    onClick={() => invoke("open_accessibility_settings")}
                    className="flex items-center gap-1 text-amber-400/70 hover:text-amber-400 transition-colors cursor-default bg-transparent border-none p-0"
                  >
                    <ShieldAlert size={12} />
                    <span className="text-[11px] font-mono">No AX access</span>
                  </button>
                </Tooltip.Trigger>
                <Tooltip.Portal>
                  <Tooltip.Content
                    className="bg-black/90 text-white/70 text-xs px-2 py-1.5 rounded border border-white/10 max-w-[220px] leading-relaxed"
                    sideOffset={4}
                  >
                    Terminal context and paste require Accessibility permission. Click to open System Settings.
                    <Tooltip.Arrow className="fill-black/90" />
                  </Tooltip.Content>
                </Tooltip.Portal>
              </Tooltip.Root>
            </Tooltip.Provider>
          )}
          {hotkeyConfigOpen ? (
            <HotkeyConfig />
          ) : (
            <>
              <CommandInput onSubmit={onSubmit} />
              {(displayMode === "streaming" || displayMode === "result") && (
                <ResultsArea />
              )}
              {/* Badge stays visible in ALL display modes below input/output */}
              {mode === "command" && (
                <div className="flex items-center gap-2 min-h-[20px]">
                  {isDetecting ? (
                    <div className="w-3 h-3 border border-white/30 border-t-white/70 rounded-full animate-spin" />
                  ) : badgeText ? (
                    <span className="text-[11px] text-white/40 font-mono">
                      {badgeText}
                    </span>
                  ) : null}
                  {!destructiveDismissed && isDestructive && displayMode === "result" && (
                    <DestructiveBadge />
                  )}
                </div>
              )}
            </>
          )}
        </>
      )}
    </div>
  );
}
