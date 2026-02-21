import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Check, ExternalLink, RefreshCw } from "lucide-react";

interface StepAccessibilityProps {
  onNext: () => void;
}

export function StepAccessibility({ onNext }: StepAccessibilityProps) {
  const [granted, setGranted] = useState<boolean | null>(null);
  const [checking, setChecking] = useState(false);

  const checkPermission = async () => {
    setChecking(true);
    try {
      const result = await invoke<boolean>("check_accessibility_permission");
      setGranted(result);
      if (result) {
        // Auto-advance after 1s if already granted
        setTimeout(() => {
          onNext();
        }, 1000);
      }
    } catch {
      setGranted(false);
    } finally {
      setChecking(false);
    }
  };

  useEffect(() => {
    checkPermission();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const handleOpenSettings = async () => {
    try {
      await invoke("open_accessibility_settings");
    } catch {
      // Best-effort
    }
  };

  const handleCheckAgain = async () => {
    await checkPermission();
  };

  return (
    <div className="flex flex-col gap-4">
      {granted === null && (
        <div className="flex items-center justify-center py-4">
          <div className="w-4 h-4 border border-white/30 border-t-white/80 rounded-full animate-spin" />
        </div>
      )}

      {granted === true && (
        <div className="flex flex-col items-center gap-3 py-2">
          <div className="w-10 h-10 rounded-full bg-green-400/15 flex items-center justify-center">
            <Check size={20} className="text-green-400" />
          </div>
          <p className="text-green-400 text-sm font-medium">
            Accessibility access granted
          </p>
          <p className="text-white/40 text-xs text-center">
            Continuing automatically...
          </p>
        </div>
      )}

      {granted === false && (
        <div className="flex flex-col gap-3">
          <p className="text-white/70 text-sm leading-relaxed">
            CMD+K needs Accessibility access to read terminal context and inject
            commands.
          </p>

          <div className="flex flex-col gap-2">
            <button
              type="button"
              onClick={handleOpenSettings}
              className={[
                "flex items-center justify-center gap-2",
                "w-full px-3 py-2 rounded-lg text-sm",
                "bg-white/10 hover:bg-white/15 border border-white/15",
                "text-white transition-colors cursor-default",
              ].join(" ")}
            >
              <ExternalLink size={14} />
              Open System Settings
            </button>

            <button
              type="button"
              onClick={handleCheckAgain}
              disabled={checking}
              className={[
                "flex items-center justify-center gap-2",
                "w-full px-3 py-2 rounded-lg text-sm",
                "bg-white/5 hover:bg-white/10 border border-white/10",
                "text-white/70 transition-colors cursor-default",
                checking ? "opacity-50" : "",
              ].join(" ")}
            >
              {checking ? (
                <div className="w-3 h-3 border border-white/30 border-t-white/80 rounded-full animate-spin" />
              ) : (
                <RefreshCw size={13} />
              )}
              Check Again
            </button>
          </div>

          <p className="text-white/30 text-xs text-center">
            You can grant this later in System Settings
          </p>
        </div>
      )}

      {/* Continue button (always available) */}
      {granted !== null && (
        <button
          type="button"
          onClick={onNext}
          className={[
            "w-full px-3 py-2 rounded-lg text-sm font-medium",
            "bg-white/10 hover:bg-white/15 border border-white/15",
            "text-white transition-colors cursor-default",
          ].join(" ")}
        >
          Continue
        </button>
      )}
    </div>
  );
}
