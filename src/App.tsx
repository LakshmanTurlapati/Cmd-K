import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Store } from "@tauri-apps/plugin-store";
import { useOverlayStore, ModelWithMeta } from "@/store";
import { useKeyboard } from "@/hooks/useKeyboard";
import { useWindowAutoSize } from "@/hooks/useWindowAutoSize";
import { useDrag } from "@/hooks/useDrag";
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

  // Enable drag-to-reposition on the overlay panel
  useDrag(panelRef);

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

        // Load selectedProvider and selectedModels map BEFORE API key checks
        const savedProvider = await store.get<string>("selectedProvider");
        if (savedProvider) {
          useOverlayStore.getState().setSelectedProvider(savedProvider);
        }
        const savedModels = await store.get<Record<string, string>>("selectedModels");
        if (savedModels) {
          useOverlayStore.getState().setSelectedModels(savedModels);
        }

        const onboardingComplete = await store.get<boolean>("onboardingComplete");
        if (!onboardingComplete) {
          let onboardingStep = (await store.get<number>("onboardingStep")) ?? 0;

          // Handle v0.2.4 upgrade: old 4-step wizard had no Provider step at index 0.
          // If no savedProvider, user was on old wizard -- reset to step 0 (provider select).
          // If savedProvider exists, old step indices need +1 offset for the new Provider step.
          if (!savedProvider && onboardingStep > 0) {
            onboardingStep = 0;
          } else if (savedProvider && onboardingStep > 0) {
            // Already on new wizard or provider was somehow set -- keep step as-is
            // (the +1 shift is only needed if the step was saved by old code)
          }

          // Load persisted destructive detection preference
          const destructiveEnabled = await store.get<boolean>("destructiveDetectionEnabled");
          useOverlayStore.getState().setDestructiveDetectionEnabled(destructiveEnabled ?? true);

          // Load persisted auto-paste preference
          const autoPaste = await store.get<boolean>("autoPasteEnabled");
          useOverlayStore.getState().setAutoPasteEnabled(autoPaste ?? true);

          // Load persisted turn limit preference
          const turnLimitValue = await store.get<number>("turnLimit");
          useOverlayStore.getState().setTurnLimit(turnLimitValue ?? 7);

          // Check if API key already exists (edge case: user saved key then closed)
          try {
            const provider = useOverlayStore.getState().selectedProvider;
            const existingKey = await invoke<string | null>("get_api_key", { provider });
            if (existingKey) {
              // Validate existing key and pre-populate store state
              await invoke("validate_api_key", { provider, apiKey: existingKey });
              const models = await invoke<ModelWithMeta[]>(
                "fetch_models",
                { provider, apiKey: existingKey }
              );
              setApiKeyStatus("valid");
              setModels(models);
              setApiKeyLast4(existingKey.slice(-4));
              // API Key is now step 1. If step was on API key step or earlier, advance past it.
              const effectiveStep =
                onboardingStep <= 1
                  ? 2  // skip past API key step since key exists
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
            const provider2 = useOverlayStore.getState().selectedProvider;
            const existingKey = await invoke<string | null>("get_api_key", { provider: provider2 });
            if (existingKey) {
              setApiKeyLast4(existingKey.slice(-4));
              await invoke("validate_api_key", { provider: provider2, apiKey: existingKey });
              const models = await invoke<ModelWithMeta[]>(
                "fetch_models",
                { provider: provider2, apiKey: existingKey }
              );
              setApiKeyStatus("valid");
              setModels(models);
            }
          } catch {
            // Non-fatal: settings panel will show current status
          }

          // Load per-provider model for the active provider from the map
          const activeProvider = useOverlayStore.getState().selectedProvider;
          if (savedModels && activeProvider && savedModels[activeProvider]) {
            setSelectedModel(savedModels[activeProvider]);
          } else {
            // Fall back to legacy single selectedModel
            const savedModel = await store.get<string>("selectedModel");
            if (savedModel) setSelectedModel(savedModel);
          }

          // Load persisted destructive detection preference
          const destructiveEnabled = await store.get<boolean>("destructiveDetectionEnabled");
          useOverlayStore.getState().setDestructiveDetectionEnabled(destructiveEnabled ?? true);

          // Load persisted auto-paste preference
          const autoPaste = await store.get<boolean>("autoPasteEnabled");
          useOverlayStore.getState().setAutoPasteEnabled(autoPaste ?? true);

          // Load persisted turn limit preference
          const turnLimitValue = await store.get<number>("turnLimit");
          useOverlayStore.getState().setTurnLimit(turnLimitValue ?? 7);
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

  // Listen for open-about event from tray menu "About" item
  useEffect(() => {
    const unlisten = listen("open-about", () => {
      openSettings("account");
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
      <div ref={panelRef} className="select-text cursor-grab active:cursor-grabbing">
        <Overlay onSubmit={handleSubmit} />
      </div>
    </div>
  );
}

export default App;
