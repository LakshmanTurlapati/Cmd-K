import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
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
          {mode === "command" && !accessibilityGranted && (
            <div className="text-red-400/70 text-xs font-mono">
              Accessibility permission required.{" "}
              <button
                type="button"
                className="text-white/50 underline underline-offset-2 hover:text-white/70 transition-colors cursor-pointer bg-transparent border-none p-0"
                onClick={() => {
                  invoke("open_accessibility_settings");
                }}
              >
                Open Settings
              </button>
            </div>
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
