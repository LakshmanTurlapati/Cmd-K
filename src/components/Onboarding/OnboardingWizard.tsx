import { Store } from "@tauri-apps/plugin-store";
import { useOverlayStore } from "@/store";
import { StepAccessibility } from "./StepAccessibility";
import { StepApiKey } from "./StepApiKey";
import { StepModelSelect } from "./StepModelSelect";
import { StepDone } from "./StepDone";

const TOTAL_STEPS = 4;

export function OnboardingWizard() {
  const onboardingStep = useOverlayStore((s) => s.onboardingStep);
  const setOnboardingStep = useOverlayStore((s) => s.setOnboardingStep);
  const setOnboardingComplete = useOverlayStore((s) => s.setOnboardingComplete);
  const setMode = useOverlayStore((s) => s.setMode);

  const persistStep = async (step: number) => {
    try {
      const store = await Store.load("settings.json");
      await store.set("onboardingStep", step);
      await store.save();
    } catch {
      // Non-fatal: progress will reset on next launch but data is safe
    }
  };

  const handleNext = async () => {
    const nextStep = onboardingStep + 1;
    setOnboardingStep(nextStep);
    await persistStep(nextStep);
  };

  const handleComplete = async () => {
    try {
      const store = await Store.load("settings.json");
      await store.set("onboardingComplete", true);
      await store.save();
    } catch {
      // Non-fatal: onboarding will re-show on next launch
    }
    setOnboardingComplete(true);
    setMode("command");
  };

  const stepLabels = ["Accessibility", "API Key", "Model", "Done"];

  return (
    <div className="flex flex-col gap-4">
      {/* Header */}
      <div className="flex flex-col items-center gap-3 pt-1">
        <h1 className="text-white text-base font-semibold tracking-tight">
          Welcome to CMD+K
        </h1>

        {/* macOS-style stepper */}
        <div className="relative w-[300px] h-[36px] flex items-center">
          {/* Track background */}
          <div className="absolute inset-x-0 h-[12px] bg-white/8 rounded-full shadow-[inset_0_1px_3px_rgba(0,0,0,0.2)]">
            {/* Track fill */}
            <div
              className="h-full rounded-full transition-all duration-400 ease-[cubic-bezier(0.25,1,0.5,1)]"
              style={{
                width: `${12 + (onboardingStep / (TOTAL_STEPS - 1)) * 76}%`,
                background: "rgba(255,255,255,0.12)",
              }}
            />
          </div>

          {/* Step nodes */}
          {stepLabels.map((_, index) => {
            const position = 12 + (index / (TOTAL_STEPS - 1)) * 76;
            const isCompleted = index < onboardingStep;
            const isActive = index === onboardingStep;

            return (
              <div
                key={index}
                className="absolute -translate-x-1/2 z-10"
                style={{ left: `${position}%` }}
              >
                <div
                  className={[
                    "w-[30px] h-[30px] rounded-full flex items-center justify-center",
                    "text-[13px] font-medium transition-all duration-300",
                    isCompleted
                      ? "bg-[#48484A]"
                      : isActive
                        ? "bg-white/15 backdrop-blur-[12px] shadow-[inset_0_0_0_1px_rgba(255,255,255,0.2),0_4px_12px_rgba(0,0,0,0.15)] text-white/90"
                        : "bg-transparent text-white/30",
                  ].join(" ")}
                >
                  {isCompleted ? (
                    <svg
                      className="w-[14px] h-[14px] transition-all duration-300 ease-[cubic-bezier(0.175,0.885,0.32,1.275)]"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="white"
                      strokeWidth="2.5"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    >
                      <polyline points="5 12 10 17 19 8" />
                    </svg>
                  ) : (
                    <span>{index + 1}</span>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Step content */}
      <div className="flex flex-col">
        {onboardingStep === 0 && <StepAccessibility onNext={handleNext} />}
        {onboardingStep === 1 && <StepApiKey onNext={handleNext} />}
        {onboardingStep === 2 && <StepModelSelect onNext={handleNext} />}
        {onboardingStep === 3 && <StepDone onComplete={handleComplete} />}
      </div>

      {/* Skip link for steps 0-2 */}
      {onboardingStep < 3 && (
        <div className="flex justify-center">
          <button
            type="button"
            onClick={handleNext}
            className="text-white/25 text-xs hover:text-white/50 transition-colors cursor-default"
          >
            Skip this step
          </button>
        </div>
      )}
    </div>
  );
}
