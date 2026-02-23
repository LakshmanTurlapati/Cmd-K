import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useOverlayStore } from "@/store";

export function useKeyboard(): void {
  const hide = useOverlayStore((state) => state.hide);
  const show = useOverlayStore((state) => state.show);

  useEffect(() => {
    // Handle Escape key globally -- two-Escape state machine:
    // streaming -> cancel (return to input)
    // result    -> returnToInput (restore previous query)
    // input     -> close overlay
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        const state = useOverlayStore.getState();
        if (state.displayMode === "streaming") {
          // During streaming: cancel and return to input
          state.cancelStreaming();
        } else if (state.displayMode === "result") {
          // In result mode: return to input with previous query
          state.returnToInput();
        } else {
          // In input mode: close the overlay
          invoke("hide_overlay").catch(console.error);
          hide();
        }
      }
    };

    document.addEventListener("keydown", handleKeyDown);

    // Listen for overlay-shown event from Rust backend
    const unlistenShown = listen("overlay-shown", () => {
      show();
    });

    // Listen for overlay-hidden event from Rust backend
    const unlistenHidden = listen("overlay-hidden", () => {
      hide();
    });

    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      unlistenShown.then((unlisten) => unlisten());
      unlistenHidden.then((unlisten) => unlisten());
    };
  }, [hide, show]);
}
