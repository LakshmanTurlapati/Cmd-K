import { create } from "zustand";

interface OverlayState {
  // Overlay visibility
  visible: boolean;
  inputValue: string;
  submitted: boolean;
  showApiWarning: boolean;

  // Hotkey config dialog
  hotkeyConfigOpen: boolean;
  currentHotkey: string;

  // Actions
  show: () => void;
  hide: () => void;
  setInputValue: (value: string) => void;
  submit: () => void;
  reset: () => void;

  openHotkeyConfig: () => void;
  closeHotkeyConfig: () => void;
  setCurrentHotkey: (shortcut: string) => void;
}

export const useOverlayStore = create<OverlayState>((set) => ({
  visible: false,
  inputValue: "",
  submitted: false,
  showApiWarning: false,

  hotkeyConfigOpen: false,
  currentHotkey: "Super+KeyK",

  show: () =>
    set({
      visible: true,
      inputValue: "",
      submitted: false,
      showApiWarning: false,
    }),

  hide: () => set({ visible: false }),

  setInputValue: (value: string) => set({ inputValue: value }),

  submit: () =>
    set({
      submitted: true,
      // Phase 1: always show API warning since no API is configured yet
      showApiWarning: true,
    }),

  reset: () =>
    set({
      inputValue: "",
      submitted: false,
      showApiWarning: false,
    }),

  openHotkeyConfig: () => set({ hotkeyConfigOpen: true }),

  closeHotkeyConfig: () => set({ hotkeyConfigOpen: false }),

  setCurrentHotkey: (shortcut: string) => set({ currentHotkey: shortcut }),
}));
