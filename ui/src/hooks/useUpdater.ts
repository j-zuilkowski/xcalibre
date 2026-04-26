import { check } from "@tauri-apps/plugin-updater"
import { useEffect, useState } from "react"

export function useUpdater() {
  const [updateAvailable, setUpdateAvailable] = useState(false)
  const [version, setVersion] = useState("")

  useEffect(() => {
    try {
      void check()
        .then((update) => {
          if (update?.available) {
            setUpdateAvailable(true)
            setVersion(update.version)
          }
        })
        .catch(() => {})
    } catch {
      // Non-Tauri or uninitialized runtime: silently skip updater checks.
    }
  }, [])

  return { updateAvailable, version }
}
