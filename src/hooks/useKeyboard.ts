import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useOverlayStore } from "@/store";

export function useKeyboard(): void {
  const hide = useOverlayStore((state) => state.hide);
  const show = useOverlayStore((state) => state.show);

  useEffect(() => {
    // Handle Escape key globally to dismiss the overlay
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        invoke("hide_overlay").catch(console.error);
        hide();
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
