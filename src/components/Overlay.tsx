import { useEffect, useState } from "react";
import { useOverlayStore } from "@/store";
import { CommandInput } from "./CommandInput";
import { ResultsArea } from "./ResultsArea";

type AnimationPhase = "entering" | "visible" | "exiting" | "hidden";

interface OverlayProps {
  onSubmit: (value: string) => void;
}

export function Overlay({ onSubmit }: OverlayProps) {
  const visible = useOverlayStore((state) => state.visible);
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

  return (
    <div
      className={[
        "w-[640px]",
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
      <CommandInput onSubmit={onSubmit} />
      <ResultsArea />
    </div>
  );
}
