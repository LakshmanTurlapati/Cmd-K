import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useOverlayStore, resolveBadge } from "@/store";
import { CommandInput } from "./CommandInput";
import { ResultsArea } from "./ResultsArea";
import { HotkeyConfig } from "./HotkeyConfig";
import { SettingsPanel } from "./Settings/SettingsPanel";
import { OnboardingWizard } from "./Onboarding/OnboardingWizard";

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
            <div
              className="flex items-center gap-2 px-3 py-2 bg-amber-900/40 border border-amber-500/30 rounded-lg text-xs text-amber-200 cursor-pointer"
              onClick={() => {
                invoke("open_accessibility_settings");
              }}
            >
              <span>Enable Accessibility for terminal context</span>
            </div>
          )}
          {hotkeyConfigOpen ? (
            <HotkeyConfig />
          ) : (
            <>
              {/* Input and output alternate in the same slot based on displayMode */}
              {displayMode === "input" ? (
                <CommandInput onSubmit={onSubmit} />
              ) : (
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
                </div>
              )}
            </>
          )}
        </>
      )}
    </div>
  );
}
