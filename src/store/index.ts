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

  show: () => {
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

  hide: () =>
    set((state) => ({
      visible: false,
      mode:
        state.mode === "onboarding" && !state.onboardingComplete
          ? "onboarding"
          : "command",
      hotkeyConfigOpen: false,
      // Reset streaming state on close
      isStreaming: false,
      displayMode: "input",
      streamingText: "",
      streamError: null,
    })),

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
    const currentState = useOverlayStore.getState();

    // Validate API key status before proceeding
    if (currentState.apiKeyStatus !== "valid") {
      set({
        streamError: "No API key configured. Open Settings to add one.",
        displayMode: "result",
        submitted: true,
      });
      return;
    }

    // Transition to streaming state (keep inputValue so the field shows the query)
    set({
      isStreaming: true,
      displayMode: "streaming",
      streamingText: "",
      streamError: null,
      previousQuery: query,
      inputValue: query,
      submitted: true,
      showApiWarning: false,
    });

    // Run the async streaming operation
    (async () => {
      try {
        const state = useOverlayStore.getState();
        const selectedModel = state.selectedModel ?? "grok-3";
        const appContext = state.appContext;
        const contextJson = appContext ? JSON.stringify(appContext) : "{}";
        const history = state.turnHistory;

        // Create a Tauri Channel for token streaming
        const onToken = new Channel<string>();
        onToken.onmessage = (token: string) => {
          useOverlayStore.getState().appendToken(token);
        };

        await invoke("stream_ai_response", {
          query,
          model: selectedModel,
          contextJson,
          history,
          onToken,
        });

        // Success: transition to result mode
        const finalState = useOverlayStore.getState();
        const finalText = finalState.streamingText;

        // Build updated turn history (max 7 turns = 14 messages)
        const updatedHistory = [
          ...finalState.turnHistory,
          { role: "user" as const, content: query },
          { role: "assistant" as const, content: finalText },
        ];
        // Trim oldest turn if we exceed 14 messages (7 turns)
        const trimmedHistory =
          updatedHistory.length > 14
            ? updatedHistory.slice(updatedHistory.length - 14)
            : updatedHistory;

        set({
          isStreaming: false,
          displayMode: "result",
          turnHistory: trimmedHistory,
        });

        // Auto-copy completed response to clipboard
        if (finalText) {
          navigator.clipboard.writeText(finalText).catch((err) => {
            console.error("[store] clipboard auto-copy failed:", err);
          });
        }
      } catch (err) {
        const errorMessage =
          typeof err === "string" ? err : "An error occurred. Try again.";
        set({
          isStreaming: false,
          displayMode: "result",
          streamError: errorMessage,
        });
      }
    })();
  },

  cancelStreaming: () =>
    set({
      isStreaming: false,
      displayMode: "input",
      streamingText: "",
      streamError: null,
    }),

  returnToInput: () =>
    set((state) => ({
      displayMode: "input",
      inputValue: state.previousQuery,
      streamingText: "",
      streamError: null,
      isStreaming: false,
    })),

  setStreamError: (error) => set({ streamError: error }),
}));
