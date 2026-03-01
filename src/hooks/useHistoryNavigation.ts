import { useState, useCallback, useRef } from "react";
import { useOverlayStore } from "@/store";

export interface UseHistoryNavigationReturn {
  /** Whether the currently displayed text is from history recall (for dimming) */
  isRecalled: boolean;
  /** Call on ArrowUp/ArrowDown keydown; returns true if the event was consumed */
  handleHistoryKey: (
    e: React.KeyboardEvent<HTMLTextAreaElement>,
    textareaEl: HTMLTextAreaElement
  ) => boolean;
  /** Call when a query is submitted (resets navigation state) */
  resetOnSubmit: () => void;
  /** Call when the user edits the input (clears recalled/dimmed state) */
  markEdited: () => void;
}

/**
 * Detects whether the cursor is on the first line of a textarea.
 * Returns true if there is no newline character before the cursor position.
 */
function isCursorOnFirstLine(el: HTMLTextAreaElement): boolean {
  const value = el.value;
  const cursorPos = el.selectionStart;
  return !value.substring(0, cursorPos).includes("\n");
}

/**
 * Custom hook for shell-like arrow-key history navigation.
 *
 * Manages history index, draft preservation, and keyboard event handling.
 * History data is read from windowHistory in Zustand (populated by Phase 8).
 *
 * - historyIndex: -1 = not navigating (draft/fresh input),
 *   0 = most recent query, 1 = second most recent, etc.
 *   Maps to windowHistory[windowHistory.length - 1 - historyIndex].
 * - draftRef: saves the user's current input on the first Arrow-Up press.
 *   Uses useRef (not useState) because the draft does not need to trigger
 *   re-renders and must survive React reconciliation within the same mount.
 */
export function useHistoryNavigation(): UseHistoryNavigationReturn {
  const [historyIndex, setHistoryIndex] = useState(-1);
  const [isRecalled, setIsRecalled] = useState(false);
  const draftRef = useRef<string>("");

  const handleHistoryKey = useCallback(
    (
      e: React.KeyboardEvent<HTMLTextAreaElement>,
      textareaEl: HTMLTextAreaElement
    ): boolean => {
      const history = useOverlayStore.getState().windowHistory;

      if (e.key === "ArrowUp") {
        // Multi-line: only trigger history when cursor is on the first line
        if (!isCursorOnFirstLine(textareaEl)) {
          return false; // let native cursor-up happen
        }

        e.preventDefault();
        if (history.length === 0) return true; // silent no-op

        if (historyIndex === -1) {
          // Save draft on first navigation
          draftRef.current = useOverlayStore.getState().inputValue;
        }

        const newIndex = Math.min(historyIndex + 1, history.length - 1);
        if (newIndex === historyIndex && historyIndex === history.length - 1) {
          // Already at oldest entry, stay put (no wrap)
          return true;
        }
        setHistoryIndex(newIndex);
        const entry = history[history.length - 1 - newIndex];
        useOverlayStore.getState().setInputValue(entry.query);
        setIsRecalled(true);
        return true;
      }

      if (e.key === "ArrowDown") {
        // Arrow-Down always navigates history when in history mode (asymmetric)
        if (historyIndex > 0) {
          e.preventDefault();
          const newIndex = historyIndex - 1;
          setHistoryIndex(newIndex);
          const entry = history[history.length - 1 - newIndex];
          useOverlayStore.getState().setInputValue(entry.query);
          setIsRecalled(true);
          return true;
        } else if (historyIndex === 0) {
          // Past newest: restore draft
          e.preventDefault();
          setHistoryIndex(-1);
          useOverlayStore.getState().setInputValue(draftRef.current);
          setIsRecalled(false);
          return true;
        }
        // Not in history mode: let native cursor-down happen
        return false;
      }

      return false;
    },
    [historyIndex]
  );

  const resetOnSubmit = useCallback(() => {
    setHistoryIndex(-1);
    setIsRecalled(false);
    draftRef.current = "";
  }, []);

  const markEdited = useCallback(() => {
    if (isRecalled) {
      setIsRecalled(false);
    }
  }, [isRecalled]);

  return { isRecalled, handleHistoryKey, resetOnSubmit, markEdited };
}
