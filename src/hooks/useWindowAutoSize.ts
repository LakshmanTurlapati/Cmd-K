import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";

export function useWindowAutoSize(ref: React.RefObject<HTMLElement | null>) {
  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const resize = () => {
      const w = el.offsetWidth;
      const h = el.offsetHeight;
      if (w > 0 && h > 0) {
        getCurrentWindow().setSize(new LogicalSize(w, h));
      }
    };

    const observer = new ResizeObserver(() => {
      resize();
    });

    observer.observe(el);
    // Initial resize
    resize();

    return () => observer.disconnect();
  }, [ref]);
}
