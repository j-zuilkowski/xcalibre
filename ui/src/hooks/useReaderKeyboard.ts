import { useKeyboard } from "./useKeyboard"

interface ReaderActions {
  nextPage: () => void
  prevPage: () => void
  close: () => void
  increaseFontSize: () => void
  decreaseFontSize: () => void
}

/** Keyboard shortcuts for the EPUB reader. */
export function useReaderKeyboard(actions: ReaderActions) {
  useKeyboard({
    "arrowright": actions.nextPage,
    "arrowleft": actions.prevPage,
    "arrowdown": actions.nextPage,
    "arrowup": actions.prevPage,
    "escape": actions.close,
    "meta+=": actions.increaseFontSize,
    "ctrl+=": actions.increaseFontSize,
    "meta+-": actions.decreaseFontSize,
    "ctrl+-": actions.decreaseFontSize,
  })
}
