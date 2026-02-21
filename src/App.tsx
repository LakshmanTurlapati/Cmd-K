import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Store } from "@tauri-apps/plugin-store";
import { useOverlayStore } from "@/store";
import { useKeyboard } from "@/hooks/useKeyboard";
import { Overlay } from "@/components/Overlay";
import { HotkeyConfig } from "@/components/HotkeyConfig";

function App() {
  const hide = useOverlayStore((state) => state.hide);
  const show = useOverlayStore((state) => state.show);
  const submit = useOverlayStore((state) => state.submit);
  const openHotkeyConfig = useOverlayStore((state) => state.openHotkeyConfig);
  const hotkeyConfigOpen = useOverlayStore((state) => state.hotkeyConfigOpen);
  const setCurrentHotkey = useOverlayStore((state) => state.setCurrentHotkey);
  const panelRef = useRef<HTMLDivElement>(null);

  // Register keyboard handler (Escape dismiss + event sync)
  useKeyboard();

  // On startup: load persisted hotkey from store and re-register it
  useEffect(() => {
    const loadPersistedHotkey = async () => {
      try {
        const store = await Store.load("settings.json");
        const savedHotkey = await store.get<string>("hotkey");
        if (savedHotkey) {
          await invoke("register_hotkey", { shortcutStr: savedHotkey });
          setCurrentHotkey(savedHotkey);
        }
      } catch (err) {
        // Non-fatal: fall back to default hotkey already registered by Rust
        console.error("Failed to load persisted hotkey:", err);
      }
    };

    loadPersistedHotkey();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // Listen for open-hotkey-config event from tray menu
  useEffect(() => {
    const unlisten = listen("open-hotkey-config", () => {
      openHotkeyConfig();
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, [openHotkeyConfig]);

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
      {hotkeyConfigOpen && <HotkeyConfig />}
    </div>
  );
}

export default App;
