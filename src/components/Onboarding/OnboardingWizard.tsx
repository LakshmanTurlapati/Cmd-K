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

        {/* Progress indicator */}
        <div className="flex items-center gap-1.5">
          {stepLabels.map((label, index) => (
            <div key={label} className="flex items-center gap-1.5">
              <div className="flex flex-col items-center gap-0.5">
                <div
                  className={[
                    "w-2 h-2 rounded-full transition-colors",
                    index < onboardingStep
                      ? "bg-green-400"
                      : index === onboardingStep
                        ? "bg-white"
                        : "bg-white/20",
                  ].join(" ")}
                />
                <span className="text-white/30 text-[10px]">{label}</span>
              </div>
              {index < TOTAL_STEPS - 1 && (
                <div
                  className={[
                    "w-6 h-px mb-3 transition-colors",
                    index < onboardingStep ? "bg-green-400/50" : "bg-white/15",
                  ].join(" ")}
                />
              )}
            </div>
          ))}
        </div>

        <p className="text-white/40 text-xs">
          Step {onboardingStep + 1} of {TOTAL_STEPS}
        </p>
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
