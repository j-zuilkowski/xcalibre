import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { useSettingsStore } from "../store/settingsStore"

interface Props {
  open_: boolean
  onClose: () => void
}

export function SettingsModal({ open_, onClose }: Props) {
  const { fontSize, theme, fontFamily, setFontSize, setTheme, setFontFamily } =
    useSettingsStore()
  const [autolibUrl, setAutolibUrl] = useState("")

  useEffect(() => {
    if (!open_) return
    invoke<string | null>("get_xs_url").then((u) => setAutolibUrl(u ?? ""))
    void invoke("has_token").catch(() => {})
  }, [open_])

  const save = async () => {
    await invoke("save_config", { autolibUrl: autolibUrl || null })
    onClose()
  }

  if (!open_) return null

  return (
    <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-900 rounded-xl shadow-xl w-full max-w-lg p-6 flex flex-col gap-6">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold">Settings</h2>
          <button onClick={onClose} className="text-gray-500 hover:text-gray-700 text-xl">
            ×
          </button>
        </div>

        <section>
          <h3 className="text-sm font-semibold text-gray-500 uppercase mb-3">General</h3>
          <label className="flex flex-col gap-1 mb-3">
            <span className="text-sm">Server URL</span>
            <input
              value={autolibUrl}
              onChange={(e) => setAutolibUrl(e.target.value)}
              className="border border-gray-300 rounded px-2 py-1 text-sm"
              placeholder="https://api.xcalibre.app"
            />
          </label>
        </section>

        <section>
          <h3 className="text-sm font-semibold text-gray-500 uppercase mb-3">Reader</h3>
          <div className="flex flex-col gap-3">
            <label className="flex items-center gap-3 text-sm">
              <span className="w-24">Font size</span>
              <input
                type="range"
                min={14}
                max={24}
                value={fontSize}
                onChange={(e) => setFontSize(Number(e.target.value))}
                className="flex-1"
              />
              <span className="w-6 text-center">{fontSize}</span>
            </label>
            <label className="flex items-center gap-3 text-sm">
              <span className="w-24">Theme</span>
              <select
                value={theme}
                onChange={(e) => setTheme(e.target.value as "light" | "dark" | "sepia")}
                className="border border-gray-300 rounded px-2 py-1"
              >
                <option value="light">Light</option>
                <option value="dark">Dark</option>
                <option value="sepia">Sepia</option>
              </select>
            </label>
            <label className="flex items-center gap-3 text-sm">
              <span className="w-24">Font family</span>
              <select
                value={fontFamily}
                onChange={(e) => setFontFamily(e.target.value as "serif" | "sans" | "mono")}
                className="border border-gray-300 rounded px-2 py-1"
              >
                <option value="serif">Serif</option>
                <option value="sans">Sans-serif</option>
                <option value="mono">Monospace</option>
              </select>
            </label>
          </div>
        </section>

        <section>
          <h3 className="text-sm font-semibold text-gray-500 uppercase mb-2">About</h3>
          <p className="text-sm text-gray-500">
            xCalibre v{import.meta.env.VITE_APP_VERSION ?? "0.0.0"}
          </p>
        </section>

        <div className="flex justify-end gap-2">
          <button onClick={onClose} className="px-4 py-2 text-sm border border-gray-300 rounded">
            Cancel
          </button>
          <button onClick={save} className="px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700">
            Save
          </button>
        </div>
      </div>
    </div>
  )
}
