import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

function App() {
  const panelRef = useRef<HTMLDivElement>(null);

  // Click outside dismisses overlay
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
        invoke("hide_overlay").catch(console.error);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  // Listen for overlay-shown event to auto-focus
  useEffect(() => {
    const unlisten = listen("overlay-shown", () => {
      // Input components will handle their own focus via useEffect
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  return (
    <div
      style={{
        width: "100vw",
        height: "100vh",
        background: "transparent",
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "center",
        paddingTop: "0px",
      }}
    >
      <div
        ref={panelRef}
        style={{
          width: "640px",
          minHeight: "80px",
          borderRadius: "12px",
          overflow: "hidden",
        }}
      >
        {/* Overlay content - Phase 2 will replace this placeholder */}
        <div
          style={{
            padding: "16px",
            color: "rgba(255,255,255,0.6)",
            fontSize: "14px",
            textAlign: "center",
          }}
        >
          CMD+K overlay ready
        </div>
      </div>
    </div>
  );
}

export default App;
