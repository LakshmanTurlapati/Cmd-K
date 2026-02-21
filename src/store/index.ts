import { create } from "zustand";

interface OverlayState {
  visible: boolean;
  inputValue: string;
  submitted: boolean;
  showApiWarning: boolean;
  show: () => void;
  hide: () => void;
  setInputValue: (value: string) => void;
  submit: () => void;
  reset: () => void;
}

export const useOverlayStore = create<OverlayState>((set) => ({
  visible: false,
  inputValue: "",
  submitted: false,
  showApiWarning: false,

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
}));
