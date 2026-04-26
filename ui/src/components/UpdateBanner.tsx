import { useEffect, useState } from "react"
import { check } from "@tauri-apps/plugin-updater"

export function UpdateBanner() {
  const [available, setAvailable] = useState(false)
  const [downloading, setDownloading] = useState(false)
  const [updateHandle, setUpdateHandle] = useState<Awaited<ReturnType<typeof check>> | null>(null)

  useEffect(() => {
    try {
      void check()
        .then((update) => {
          if (update?.available) {
            setAvailable(true)
            setUpdateHandle(update)
          }
        })
        .catch(() => { /* no update server configured */ })
    } catch {
      // Skip update checks outside a fully initialized Tauri runtime.
    }
  }, [])

  if (!available) return null

  const install = async () => {
    if (!updateHandle) return
    setDownloading(true)
    try {
      await updateHandle.downloadAndInstall()
    } catch (e) {
      console.error("Update failed:", e)
      setDownloading(false)
    }
  }

  return (
    <div className="fixed bottom-4 right-4 z-50 bg-blue-600 text-white rounded-xl shadow-lg px-4 py-3 flex items-center gap-3">
      <span className="text-sm font-medium">Update available</span>
      <button
        onClick={install}
        disabled={downloading}
        className="text-sm px-3 py-1 bg-white text-blue-600 rounded-lg hover:bg-blue-50 disabled:opacity-50"
      >
        {downloading ? "Installing…" : "Install & Restart"}
      </button>
    </div>
  )
}
