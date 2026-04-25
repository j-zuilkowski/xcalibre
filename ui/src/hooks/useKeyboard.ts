import { useEffect } from "react"

type KeyMap = Record<string, () => void>

/**
 * Bind keyboard shortcuts globally.
 * Keys use the format: "ctrl+k", "meta+f", "escape", "arrowleft"
 * Call inside a component; bindings are removed on unmount.
 */
export function useKeyboard(keyMap: KeyMap) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const key = [
        e.ctrlKey ? "ctrl" : null,
        e.metaKey ? "meta" : null,
        e.altKey ? "alt" : null,
        e.shiftKey ? "shift" : null,
        e.key.toLowerCase(),
      ]
        .filter(Boolean)
        .join("+")

      const action = keyMap[key]
      if (action) {
        e.preventDefault()
        action()
      }
    }

    window.addEventListener("keydown", handler)
    return () => window.removeEventListener("keydown", handler)
  }, [keyMap])
}
