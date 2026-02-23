import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

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
}));
