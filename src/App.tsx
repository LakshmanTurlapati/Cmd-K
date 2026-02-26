import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Store } from "@tauri-apps/plugin-store";
import { useOverlayStore, XaiModelWithMeta } from "@/store";
import { useKeyboard } from "@/hooks/useKeyboard";
import { useWindowAutoSize } from "@/hooks/useWindowAutoSize";
import { Overlay } from "@/components/Overlay";

function App() {
  const hide = useOverlayStore((state) => state.hide);
  const show = useOverlayStore((state) => state.show);
  const submitQuery = useOverlayStore((state) => state.submitQuery);
  const openSettings = useOverlayStore((state) => state.openSettings);
  const openOnboarding = useOverlayStore((state) => state.openOnboarding);
  const setCurrentHotkey = useOverlayStore((state) => state.setCurrentHotkey);
  const setApiKeyStatus = useOverlayStore((state) => state.setApiKeyStatus);
  const setApiKeyLast4 = useOverlayStore((state) => state.setApiKeyLast4);
  const setModels = useOverlayStore((state) => state.setModels);
  const setSelectedModel = useOverlayStore((state) => state.setSelectedModel);
  const panelRef = useRef<HTMLDivElement>(null);

  // Register keyboard handler (Escape dismiss + event sync)
  useKeyboard();

  // Dynamically resize the Tauri window to match panel content
  useWindowAutoSize(panelRef);

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

  // On startup: check onboarding completion status and resume or skip accordingly
  useEffect(() => {
    const checkOnboarding = async () => {
      try {
        const store = await Store.load("settings.json");
        const onboardingComplete = await store.get<boolean>("onboardingComplete");
        if (!onboardingComplete) {
          const onboardingStep = (await store.get<number>("onboardingStep")) ?? 0;

          // Load persisted destructive detection preference
          const destructiveEnabled = await store.get<boolean>("destructiveDetectionEnabled");
          useOverlayStore.getState().setDestructiveDetectionEnabled(destructiveEnabled ?? true);

          // Load persisted auto-paste preference
          const autoPaste = await store.get<boolean>("autoPasteEnabled");
          useOverlayStore.getState().setAutoPasteEnabled(autoPaste ?? true);

          // Check if API key already exists (edge case: user saved key then closed)
          try {
            const existingKey = await invoke<string | null>("get_api_key");
            if (existingKey) {
              // Validate existing key and pre-populate store state
              const models = await invoke<XaiModelWithMeta[]>(
                "validate_and_fetch_models",
                { apiKey: existingKey }
              );
              setApiKeyStatus("valid");
              setModels(models);
              setApiKeyLast4(existingKey.slice(-4));
              // API Key is now step 0. If step was on API key step or earlier, advance past it.
              const effectiveStep =
                onboardingStep <= 0
                  ? 1  // skip past API key step since key exists
                  : onboardingStep;
              openOnboarding(effectiveStep);
            } else {
              openOnboarding(onboardingStep);
            }
          } catch {
            openOnboarding(onboardingStep);
          }
          // Show the native window so the onboarding wizard is visible
          await invoke("show_overlay");
        } else {
          // Pre-check accessibility so the warning doesn't flash on first show()
          try {
            const hasAccess = await invoke<boolean>("check_accessibility_permission");
            useOverlayStore.getState().setAccessibilityGranted(hasAccess);
          } catch {
            // Non-fatal: store stays false, warning will show if needed
          }

          // Onboarding done -- load API key status and models for settings panel
          try {
            const existingKey = await invoke<string | null>("get_api_key");
            if (existingKey) {
              setApiKeyLast4(existingKey.slice(-4));
              const models = await invoke<XaiModelWithMeta[]>(
                "validate_and_fetch_models",
                { apiKey: existingKey }
              );
              setApiKeyStatus("valid");
              setModels(models);
            }
          } catch {
            // Non-fatal: settings panel will show current status
          }
          // Load persisted model selection
          const savedModel = await store.get<string>("selectedModel");
          if (savedModel) setSelectedModel(savedModel);

          // Load persisted destructive detection preference
          const destructiveEnabled = await store.get<boolean>("destructiveDetectionEnabled");
          useOverlayStore.getState().setDestructiveDetectionEnabled(destructiveEnabled ?? true);

          // Load persisted auto-paste preference
          const autoPaste = await store.get<boolean>("autoPasteEnabled");
          useOverlayStore.getState().setAutoPasteEnabled(autoPaste ?? true);
        }
      } catch (err) {
        // Non-fatal: fall back to default command mode
        console.error("Failed to check onboarding status:", err);
      }
    };

    checkOnboarding();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // Listen for open-settings event from tray menu "Settings..." item
  useEffect(() => {
    const unlisten = listen("open-settings", () => {
      openSettings();
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, [openSettings]);

  // Listen for open-hotkey-config event from tray menu "Change Hotkey..." item
  useEffect(() => {
    const unlisten = listen("open-hotkey-config", () => {
      openSettings("preferences");
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, [openSettings]);

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

  // Click outside (window blur) dismisses overlay.
  // When the user clicks anywhere outside the NSPanel, the panel loses key
  // window status and the webview fires a blur event. During paste operations
  // the panel temporarily resigns key (isPasting guards against false dismiss).
  // Also guard against dismissal during streaming (AI generating) since internal
  // focus changes (textarea blur) can cause NSPanel to resign key on macOS.
  useEffect(() => {
    const handleBlur = () => {
      const state = useOverlayStore.getState();
      if (
        state.visible &&
        !state.isPasting &&
        !state.isStreaming &&
        state.mode === "command"
      ) {
        invoke("hide_overlay").catch(console.error);
        hide();
      }
    };
    window.addEventListener("blur", handleBlur);
    return () => window.removeEventListener("blur", handleBlur);
  }, [hide]);

  const handleSubmit = (value: string) => {
    const trimmed = value.trim();
    if (trimmed === "/settings") {
      openSettings();
      return;
    }
    if (trimmed) {
      submitQuery(trimmed);
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
