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

export interface TerminalContextSnapshot {
  cwd: string | null;
  shell_type: string | null;
  visible_output: string | null;
}

export interface HistoryEntry {
  query: string;
  response: string;
  timestamp: number;
  terminal_context: TerminalContextSnapshot | null;
  is_error: boolean;
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

  // Window identity and per-window history
  windowKey: string | null;
  windowHistory: HistoryEntry[];

  // AI conversation context
  turnLimit: number;

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

  // Window identity actions
  setWindowKey: (key: string | null) => void;
  setWindowHistory: (history: HistoryEntry[]) => void;

  // AI conversation context actions
  setTurnLimit: (limit: number) => void;
  setTurnHistory: (history: TurnMessage[]) => void;

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

  // Window identity initial state
  windowKey: null,
  windowHistory: [],

  // AI conversation context initial state
  turnLimit: 7,

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
      // Reset window identity on each overlay open
      windowKey: null,
      windowHistory: [],
      // Reset streaming state on each overlay open (turnHistory reconstructed from windowHistory below)
      streamingText: "",
      isStreaming: false,
      displayMode: "input",
      previousQuery: "",
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

        // Fetch window key (computed synchronously by hotkey handler before overlay showed)
        const windowKey = await invoke<string | null>("get_window_key");
        console.log("[store] window key:", windowKey);
        if (windowKey) {
          useOverlayStore.getState().setWindowKey(windowKey);
          // Fetch existing history for this window
          const history = await invoke<HistoryEntry[]>("get_window_history", { windowKey });
          console.log("[store] window history entries:", history.length);
          useOverlayStore.getState().setWindowHistory(history);

          // Reconstruct turnHistory from stored entries (CTXT-01, CTXT-02)
          const turnLimit = useOverlayStore.getState().turnLimit;
          const turnMessages: TurnMessage[] = history
            .filter(e => !e.is_error && e.response)
            .flatMap(e => [
              { role: "user" as const, content: e.query },
              { role: "assistant" as const, content: e.response },
            ]);
          const maxMessages = turnLimit * 2;
          const trimmed = turnMessages.length > maxMessages
            ? turnMessages.slice(turnMessages.length - maxMessages)
            : turnMessages;
          useOverlayStore.getState().setTurnHistory(trimmed);
        } else {
          useOverlayStore.getState().setWindowKey(null);
          useOverlayStore.getState().setWindowHistory([]);
          useOverlayStore.getState().setTurnHistory([]);
        }

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

  // Window identity action implementations
  setWindowKey: (key) => set({ windowKey: key }),
  setWindowHistory: (history) => set({ windowHistory: history }),

  // AI conversation context action implementations
  setTurnLimit: (limit) => set({ turnLimit: limit }),
  setTurnHistory: (history) => set({ turnHistory: history }),

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
        const currentTurnLimit = useOverlayStore.getState().turnLimit;
        const maxMessages = currentTurnLimit * 2;
        const trimmedHistory =
          updatedHistory.length > maxMessages
            ? updatedHistory.slice(updatedHistory.length - maxMessages)
            : updatedHistory;

        // Persist to Rust-side history (survives overlay close/reopen)
        const currentWindowKey = useOverlayStore.getState().windowKey;
        if (currentWindowKey) {
          const historyCtx = appContext?.terminal ? {
            cwd: appContext.terminal.cwd,
            shell_type: appContext.terminal.shell_type,
            visible_output: appContext.terminal.visible_output,
          } : null;

          invoke("add_history_entry", {
            windowKey: currentWindowKey,
            query,
            response: fullText,
            terminalContext: historyCtx,
            isError: false,
          }).catch((err: unknown) => {
            console.error("[store] add_history_entry failed:", err);
          });

          // Sync local windowHistory for immediate Arrow-Up recall
          const historySync: HistoryEntry = {
            query,
            response: fullText,
            timestamp: Date.now(),
            terminal_context: historyCtx,
            is_error: false,
          };
          const currentHistory = useOverlayStore.getState().windowHistory;
          useOverlayStore.getState().setWindowHistory([...currentHistory, historySync]);
        }

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

        // Persist failed query to history too (user may want to retry via arrow-key recall)
        const errWindowKey = useOverlayStore.getState().windowKey;
        if (errWindowKey) {
          invoke("add_history_entry", {
            windowKey: errWindowKey,
            query,
            response: "",
            terminalContext: null,
            isError: true,
          }).catch((histErr: unknown) => {
            console.error("[store] add_history_entry (error case) failed:", histErr);
          });

          // Sync local windowHistory for error query recall
          const errorHistorySync: HistoryEntry = {
            query,
            response: "",
            timestamp: Date.now(),
            terminal_context: null,
            is_error: true,
          };
          const currentErrHistory = useOverlayStore.getState().windowHistory;
          useOverlayStore.getState().setWindowHistory([...currentErrHistory, errorHistorySync]);
        }

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
