import { create } from "zustand";
import { invoke, Channel } from "@tauri-apps/api/core";

export type OverlayMode = "command" | "onboarding" | "settings";

export interface XaiModelWithMeta {
  id: string;
  label: string;
}

export interface TerminalContext {
  shell_type: string | null;
  cwd: string | null;
  visible_output: string | null;
  running_process: string | null;
}

export interface AppContext {
  app_name: string | null;
  terminal: TerminalContext | null;
  console_detected: boolean;
  console_last_line: string | null;
  visible_text: string | null;
}

/** Resolve the badge text from AppContext using priority: shell > console > app name */
export function resolveBadge(ctx: AppContext | null): string | null {
  if (!ctx) return null;

  // Priority 1: Shell type (from terminal or editor integrated terminal)
  if (ctx.terminal?.shell_type) {
    return ctx.terminal.shell_type;
  }

  // Priority 2: Console (browser has DevTools open)
  if (ctx.console_detected) {
    return "Console";
  }

  // Priority 3: App name
  if (ctx.app_name) {
    return ctx.app_name;
  }

  return null;
}

export interface TurnMessage {
  role: "user" | "assistant";
  content: string;
}

interface OverlayState {
  // Overlay visibility
  visible: boolean;
  inputValue: string;
  submitted: boolean;
  showApiWarning: boolean;

  // Mode switching
  mode: OverlayMode;

  // Onboarding state
  onboardingStep: number;
  onboardingComplete: boolean;

  // API key and model state
  apiKeyStatus: "unknown" | "validating" | "valid" | "invalid" | "error";
  apiKeyLast4: string;
  selectedModel: string | null;
  availableModels: XaiModelWithMeta[];

  // Settings panel
  settingsTab: string;
  hotkeyConfigOpen: boolean;
  currentHotkey: string;

  // App context (terminal, browser console, app name)
  appContext: AppContext | null;
  isDetectingContext: boolean;
  accessibilityGranted: boolean;

  // Streaming state
  streamingText: string;
  isStreaming: boolean;
  displayMode: "input" | "streaming" | "result";
  previousQuery: string;
  turnHistory: TurnMessage[];
  streamError: string | null;

  // Destructive command detection
  isDestructive: boolean;
  destructiveExplanation: string | null;
  destructiveDismissed: boolean;
  destructiveDetectionEnabled: boolean;

  // Auto-paste preference
  autoPasteEnabled: boolean;
  isPasting: boolean;
  setAutoPasteEnabled: (enabled: boolean) => void;

  // Actions
  show: () => void;
  hide: () => void;
  setInputValue: (value: string) => void;
  submit: () => void;
  reset: () => void;

  openSettings: (tab?: string) => void;
  openOnboarding: (step?: number) => void;
  setMode: (mode: OverlayMode) => void;
  setOnboardingStep: (step: number) => void;
  setOnboardingComplete: (complete: boolean) => void;
  setApiKeyStatus: (
    status: "unknown" | "validating" | "valid" | "invalid" | "error"
  ) => void;
  setApiKeyLast4: (last4: string) => void;
  setModels: (models: XaiModelWithMeta[]) => void;
  setSelectedModel: (model: string) => void;
  setSettingsTab: (tab: string) => void;

  openHotkeyConfig: () => void;
  closeHotkeyConfig: () => void;
  setCurrentHotkey: (shortcut: string) => void;

  setAppContext: (ctx: AppContext | null) => void;
  setIsDetectingContext: (detecting: boolean) => void;
  setAccessibilityGranted: (granted: boolean) => void;

  // Streaming actions
  appendToken: (token: string) => void;
  submitQuery: (query: string) => void;
  cancelStreaming: () => void;
  returnToInput: () => void;
  setStreamError: (error: string | null) => void;

  // Destructive detection actions
  setIsDestructive: (value: boolean) => void;
  setDestructiveExplanation: (explanation: string | null) => void;
  dismissDestructiveBadge: () => void;
  setDestructiveDetectionEnabled: (enabled: boolean) => void;
}

// Synchronized reveal timer -- types text into overlay at ~400 chars/sec
// to match the terminal paste speed. Module-scoped so all actions can cancel it.
let _revealTimer: ReturnType<typeof setInterval> | null = null;
function clearRevealTimer() {
  if (_revealTimer !== null) {
    clearInterval(_revealTimer);
    _revealTimer = null;
  }
}

export const useOverlayStore = create<OverlayState>((set) => ({
  visible: false,
  inputValue: "",
  submitted: false,
  showApiWarning: false,

  mode: "command",
  onboardingStep: 0,
  onboardingComplete: false,

  apiKeyStatus: "unknown",
  apiKeyLast4: "",
  selectedModel: null,
  availableModels: [],

  settingsTab: "account",
  hotkeyConfigOpen: false,
  currentHotkey: "Super+KeyK",

  appContext: null,
  isDetectingContext: false,
  accessibilityGranted: false,

  // Streaming initial state
  streamingText: "",
  isStreaming: false,
  displayMode: "input",
  previousQuery: "",
  turnHistory: [],
  streamError: null,

  // Destructive command detection initial state
  isDestructive: false,
  destructiveExplanation: null,
  destructiveDismissed: false,
  destructiveDetectionEnabled: true,

  // Auto-paste preference initial state
  autoPasteEnabled: true,
  isPasting: false,

  show: () => {
    clearRevealTimer();
    set((state) => ({
      visible: true,
      mode:
        state.mode === "onboarding" && !state.onboardingComplete
          ? "onboarding"
          : "command",
      hotkeyConfigOpen: false,
      inputValue: "",
      submitted: false,
      showApiWarning: false,
      appContext: null,
      isDetectingContext: true,
      // Reset streaming state on each overlay open
      streamingText: "",
      isStreaming: false,
      displayMode: "input",
      previousQuery: "",
      turnHistory: [],
      streamError: null,
      // Reset destructive detection state on each overlay open
      isDestructive: false,
      destructiveExplanation: null,
      destructiveDismissed: false,
      isPasting: false,
    }));

    // Fire-and-forget context detection (non-blocking)
    // Overlay appears immediately with a spinner; context fills in after detection completes
    (async () => {
      try {
        // Check accessibility permission each time overlay opens
        const hasPermission = await invoke<boolean>(
          "check_accessibility_permission"
        );
        console.log("[store] accessibility permission:", hasPermission);
        useOverlayStore.getState().setAccessibilityGranted(hasPermission);

        // Detect app context (returns AppContext for ALL frontmost apps)
        const ctx = await invoke<AppContext | null>("get_app_context");
        console.log("[store] app context:", JSON.stringify(ctx));
        useOverlayStore.getState().setAppContext(ctx);
      } catch (err) {
        console.error("[store] context detection error:", err);
      } finally {
        useOverlayStore.getState().setIsDetectingContext(false);
      }
    })();
  },

  hide: () => {
    clearRevealTimer();
    set((state) => ({
      visible: false,
      mode:
        state.mode === "onboarding" && !state.onboardingComplete
          ? "onboarding"
          : "command",
      hotkeyConfigOpen: false,
      isStreaming: false,
      displayMode: "input",
      streamingText: "",
      streamError: null,
      isDestructive: false,
      destructiveExplanation: null,
      destructiveDismissed: false,
      isPasting: false,
    }));
  },

  setInputValue: (value: string) => set({ inputValue: value }),

  submit: () =>
    set((state) => ({
      submitted: true,
      showApiWarning: state.apiKeyStatus !== "valid",
    })),

  reset: () =>
    set({
      inputValue: "",
      submitted: false,
      showApiWarning: false,
    }),

  openSettings: (tab?: string) =>
    set({
      mode: "settings",
      visible: true,
      settingsTab: tab ?? "account",
      inputValue: "",
      submitted: false,
      showApiWarning: false,
      hotkeyConfigOpen: false,
    }),

  openOnboarding: (step?: number) =>
    set({
      mode: "onboarding",
      visible: true,
      onboardingStep: step ?? 0,
    }),

  setMode: (mode: OverlayMode) => set({ mode }),

  setOnboardingStep: (step: number) => set({ onboardingStep: step }),

  setOnboardingComplete: (complete: boolean) =>
    set({ onboardingComplete: complete }),

  setApiKeyStatus: (
    status: "unknown" | "validating" | "valid" | "invalid" | "error"
  ) => set({ apiKeyStatus: status }),

  setApiKeyLast4: (last4: string) => set({ apiKeyLast4: last4 }),

  setModels: (models: XaiModelWithMeta[]) => set({ availableModels: models }),

  setSelectedModel: (model: string) => set({ selectedModel: model }),

  setSettingsTab: (tab: string) => set({ settingsTab: tab }),

  openHotkeyConfig: () => set({ hotkeyConfigOpen: true }),

  closeHotkeyConfig: () => set({ hotkeyConfigOpen: false }),

  setCurrentHotkey: (shortcut: string) => set({ currentHotkey: shortcut }),

  setAppContext: (ctx) => set({ appContext: ctx }),

  setIsDetectingContext: (detecting) => set({ isDetectingContext: detecting }),

  setAccessibilityGranted: (granted) => set({ accessibilityGranted: granted }),

  // Streaming action implementations
  appendToken: (token) =>
    set((state) => ({
      streamingText: state.streamingText + token,
    })),

  submitQuery: (query: string) => {
    clearRevealTimer();
    const currentState = useOverlayStore.getState();

    if (currentState.apiKeyStatus !== "valid") {
      set({
        streamError: "No API key configured. Open Settings to add one.",
        displayMode: "result",
        submitted: true,
      });
      return;
    }

    set({
      isStreaming: true,
      displayMode: "streaming",
      streamingText: "",
      streamError: null,
      previousQuery: query,
      inputValue: query,
      submitted: true,
      showApiWarning: false,
      isDestructive: false,
      destructiveExplanation: null,
      destructiveDismissed: false,
    });

    (async () => {
      try {
        // Wait for context detection to finish (max 2s) so the AI gets terminal context
        console.log("[submitQuery] isDetectingContext:", useOverlayStore.getState().isDetectingContext);
        if (useOverlayStore.getState().isDetectingContext) {
          console.log("[submitQuery] waiting for context detection...");
          await new Promise<void>((resolve) => {
            const timeout = setTimeout(resolve, 2000);
            const check = () => {
              if (!useOverlayStore.getState().isDetectingContext) {
                clearTimeout(timeout);
                resolve();
              } else {
                setTimeout(check, 50);
              }
            };
            check();
          });
          console.log("[submitQuery] context detection finished, appContext:", JSON.stringify(useOverlayStore.getState().appContext));
        }

        const state = useOverlayStore.getState();
        const selectedModel = state.selectedModel ?? "grok-3";
        const appContext = state.appContext;
        const contextJson = appContext ? JSON.stringify(appContext) : "{}";
        console.log("[submitQuery] context sent to AI:", contextJson);
        const history = state.turnHistory;

        // Stream tokens to overlay in real-time so the cursor is visible
        // throughout the entire AI generation (1-3 seconds).
        let fullText = "";
        const onToken = new Channel<string>();
        onToken.onmessage = (token: string) => {
          fullText += token;
          set({ streamingText: fullText });
        };

        await invoke("stream_ai_response", {
          query,
          model: selectedModel,
          contextJson,
          history,
          onToken,
        });

        // All tokens received -- build turn history
        const finalState = useOverlayStore.getState();
        const updatedHistory = [
          ...finalState.turnHistory,
          { role: "user" as const, content: query },
          { role: "assistant" as const, content: fullText },
        ];
        const trimmedHistory =
          updatedHistory.length > 14
            ? updatedHistory.slice(updatedHistory.length - 14)
            : updatedHistory;

        // Destructive check BEFORE paste
        let destructive = false;
        const pasteState = useOverlayStore.getState();
        if (pasteState.destructiveDetectionEnabled && fullText) {
          try {
            destructive = await invoke<boolean>("check_destructive", {
              command: fullText,
            });
          } catch (err) {
            console.error("[store] check_destructive failed:", err);
          }
        }

        if (destructive) {
          // Destructive: mark with badge, no paste
          set({
            isStreaming: false,
            displayMode: "result",
            streamingText: fullText,
            turnHistory: trimmedHistory,
            isDestructive: true,
          });
        } else if (fullText) {
          // Safe: paste to terminal, text already visible in overlay
          const afterCheck = useOverlayStore.getState();
          if (afterCheck.autoPasteEnabled) {
            set({ isPasting: true });
            invoke("paste_to_terminal", { command: fullText })
              .catch((err) => {
                console.error(
                  "[store] auto-paste failed (clipboard fallback available):",
                  err
                );
              })
              .finally(() => {
                set({ isPasting: false });
              });
          }

          set({
            isStreaming: false,
            displayMode: "result",
            streamingText: fullText,
            turnHistory: trimmedHistory,
          });
        } else {
          set({
            isStreaming: false,
            displayMode: "result",
            streamingText: "",
            turnHistory: trimmedHistory,
          });
        }
      } catch (err) {
        clearRevealTimer();
        const errorMessage =
          typeof err === "string" ? err : "An error occurred. Try again.";
        set({
          isStreaming: false,
          displayMode: "result",
          streamingText: "",
          streamError: errorMessage,
        });
      }
    })();
  },

  cancelStreaming: () => {
    clearRevealTimer();
    set({
      isStreaming: false,
      displayMode: "input",
      streamingText: "",
      streamError: null,
    });
  },

  returnToInput: () => {
    clearRevealTimer();
    set((state) => ({
      displayMode: "input",
      inputValue: state.previousQuery,
      streamingText: "",
      streamError: null,
      isStreaming: false,
    }));
  },

  setStreamError: (error) => set({ streamError: error }),

  // Destructive detection action implementations
  setIsDestructive: (value) => set({ isDestructive: value }),
  setDestructiveExplanation: (explanation) => set({ destructiveExplanation: explanation }),
  dismissDestructiveBadge: () => set({ destructiveDismissed: true }),
  setDestructiveDetectionEnabled: (enabled) => set({ destructiveDetectionEnabled: enabled }),

  // Auto-paste preference action implementation
  setAutoPasteEnabled: (enabled) => set({ autoPasteEnabled: enabled }),
}));
