import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useOverlayStore } from "@/store";

export function useKeyboard(): void {
  const hide = useOverlayStore((state) => state.hide);
  const show = useOverlayStore((state) => state.show);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Escape always closes the overlay regardless of display mode
      if (e.key === "Escape") {
        invoke("hide_overlay").catch(console.error);
        hide();
        return;
      }

      // Enter in result mode: execute the pasted command in terminal and dismiss.
      // Guards: must have actual AI content (streamingText), no error, not destructive.
      if (e.key === "Enter" && !e.shiftKey) {
        const state = useOverlayStore.getState();
        if (
          state.displayMode === "result" &&
          state.autoPasteEnabled &&
          !state.isDestructive &&
          !state.streamError &&
          state.streamingText.length > 0
        ) {
          e.preventDefault();
          invoke("confirm_terminal_command").catch((err) => {
            console.error("[useKeyboard] confirm failed:", err);
          });
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
