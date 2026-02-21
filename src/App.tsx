import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useOverlayStore } from "@/store";
import { useKeyboard } from "@/hooks/useKeyboard";
import { Overlay } from "@/components/Overlay";

function App() {
  const hide = useOverlayStore((state) => state.hide);
  const show = useOverlayStore((state) => state.show);
  const submit = useOverlayStore((state) => state.submit);
  const panelRef = useRef<HTMLDivElement>(null);

  // Register keyboard handler (Escape dismiss + event sync)
  useKeyboard();

  // Click outside dismisses overlay
  const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
      invoke("hide_overlay").catch(console.error);
      hide();
    }
  };

  // Listen for overlay-shown event from Rust backend to sync state
  useEffect(() => {
    const unlisten = listen("overlay-shown", () => {
      show();
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, [show]);

  const handleSubmit = (value: string) => {
    if (value.trim()) {
      submit();
    }
  };

  return (
    <div
      className="w-screen h-screen flex items-start justify-center select-none"
      style={{ background: "transparent" }}
      onMouseDown={handleMouseDown}
    >
      <div ref={panelRef} className="select-text">
        <Overlay onSubmit={handleSubmit} />
      </div>
    </div>
  );
}

export default App;
