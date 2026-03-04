import { useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalPosition } from "@tauri-apps/api/dpi";
import { invoke } from "@tauri-apps/api/core";

/**
 * Enables drag-to-move on the overlay panel.
 *
 * Mousedown on the panel (but NOT on interactive elements like input, textarea,
 * button, a, [role="button"]) starts a drag. Mousemove updates the window
 * position in real-time. Mouseup ends the drag and persists the final position
 * to Rust AppState via set_overlay_position.
 *
 * Position is stored in logical coordinates (scale-factor aware).
 */
export function useDrag(ref: React.RefObject<HTMLElement | null>) {
  const isDragging = useRef(false);
  const dragStart = useRef({ x: 0, y: 0 });
  const windowStart = useRef({ x: 0, y: 0 });

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const handleMouseDown = async (e: MouseEvent) => {
      // Only left-click
      if (e.button !== 0) return;

      // Don't initiate drag on interactive elements — the user is clicking
      // an input, button, textarea, link, or other interactive widget
      const target = e.target as HTMLElement;
      if (
        target.closest(
          "input, textarea, button, a, [role='button'], [data-no-drag]"
        )
      ) {
        return;
      }

      isDragging.current = true;
      dragStart.current = { x: e.screenX, y: e.screenY };

      // Get current window position (physical pixels) and scale factor
      const win = getCurrentWindow();
      const physPos = await win.outerPosition();
      const scale = await win.scaleFactor();

      // Convert to logical coordinates
      windowStart.current = {
        x: physPos.x / scale,
        y: physPos.y / scale,
      };

      // Prevent text selection during drag
      e.preventDefault();
    };

    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging.current) return;

      const dx = e.screenX - dragStart.current.x;
      const dy = e.screenY - dragStart.current.y;

      const newX = windowStart.current.x + dx;
      const newY = windowStart.current.y + dy;

      // Move window in real-time (fire-and-forget — no await needed)
      getCurrentWindow().setPosition(new LogicalPosition(newX, newY));
    };

    const handleMouseUp = async (e: MouseEvent) => {
      if (!isDragging.current) return;
      isDragging.current = false;

      // Compute final position
      const dx = e.screenX - dragStart.current.x;
      const dy = e.screenY - dragStart.current.y;

      // Only persist if the overlay actually moved (ignore clicks)
      if (Math.abs(dx) > 2 || Math.abs(dy) > 2) {
        const finalX = windowStart.current.x + dx;
        const finalY = windowStart.current.y + dy;

        // Persist to Rust AppState for session-scoped memory
        invoke("set_overlay_position", { x: finalX, y: finalY }).catch(
          (err) =>
            console.error("[useDrag] set_overlay_position failed:", err)
        );
      }
    };

    el.addEventListener("mousedown", handleMouseDown);
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);

    return () => {
      el.removeEventListener("mousedown", handleMouseDown);
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [ref]);
}
