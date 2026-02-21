import { create } from "zustand";

export type OverlayMode = "command" | "onboarding" | "settings";

export interface XaiModelWithMeta {
  id: string;
  label: string;
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

  show: () =>
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
    })),

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
}));
