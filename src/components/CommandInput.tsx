import { useEffect, useRef, useMemo } from "react";
import { useOverlayStore } from "@/store";

const COMMANDS = ["/settings"];

interface CommandInputProps {
  onSubmit: (value: string) => void;
}

export function CommandInput({ onSubmit }: CommandInputProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const inputValue = useOverlayStore((state) => state.inputValue);
  const setInputValue = useOverlayStore((state) => state.setInputValue);
  const visible = useOverlayStore((state) => state.visible);

  const suggestion = useMemo(() => {
    if (!inputValue.startsWith("/") || inputValue.length < 1) return "";
    const match = COMMANDS.find((cmd) => cmd.startsWith(inputValue) && cmd !== inputValue);
    return match ?? "";
  }, [inputValue]);

  // Auto-focus when component mounts
  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  // Re-focus when overlay becomes visible
  useEffect(() => {
    if (visible) {
      // Small delay to ensure the animation has started and panel is ready
      const timer = setTimeout(() => {
        textareaRef.current?.focus();
      }, 50);
      return () => clearTimeout(timer);
    }
  }, [visible]);

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInputValue(e.target.value);
    // Auto-grow: reset height then expand to scrollHeight
    const el = e.target;
    el.style.height = "auto";
    el.style.height = `${el.scrollHeight}px`;
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Tab" && suggestion) {
      e.preventDefault();
      setInputValue(suggestion);
      return;
    }
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      if (inputValue.trim()) {
        onSubmit(inputValue);
      }
    }
    // Shift+Enter: default textarea behavior inserts a newline
    // Escape: handled by useKeyboard hook in App.tsx
  };

  return (
    <div className="relative w-full">
      {/* Ghost suggestion layer */}
      {suggestion && (
        <div
          className="absolute inset-0 pointer-events-none text-sm leading-relaxed text-white/20"
          style={{ minHeight: "24px" }}
          aria-hidden="true"
        >
          {suggestion}
        </div>
      )}
      <textarea
        ref={textareaRef}
        rows={1}
        value={inputValue}
        onChange={handleChange}
        onKeyDown={handleKeyDown}
        placeholder="Describe a task or type a command..."
        className={[
          "w-full",
          "bg-transparent",
          "text-white",
          "text-sm",
          "leading-relaxed",
          "resize-none",
          "outline-none",
          "ring-0",
          "border-none",
          "max-h-[200px]",
          "overflow-y-auto",
          "placeholder:text-white/40",
          "scrollbar-thin",
          "relative",
        ].join(" ")}
        style={{ minHeight: "24px" }}
      />
    </div>
  );
}
