/**
 * PluginManagerModal — list, install, enable/disable, and remove plugins.
 *
 * ## Layout
 *
 * A centred modal overlay containing:
 * - A list of installed plugins, each showing name, version, type badge,
 *   an enable/disable toggle, and a Remove button.
 * - A dashed "Install plugin from ZIP…" button at the bottom that opens the
 *   native file picker filtered to `.zip` files.
 * - An error banner that appears if any IPC call fails.
 *
 * ## Tauri commands used
 *
 * | Command | Purpose |
 * |---------|---------|
 * | `list_plugins_cmd` | Load the plugin list on mount and after every mutation |
 * | `install_plugin_from_zip` | Extract ZIP + register in DB |
 * | `set_plugin_enabled_cmd` | Toggle enabled state |
 * | `uninstall_plugin_cmd` | Remove DB row + delete dylib |
 *
 * ## Data flow
 *
 * All mutations call `reload()` after the IPC resolves so the list is always
 * in sync with the database. No local optimistic updates are applied — the
 * round-trip is fast enough on localhost SQLite that it is not needed.
 *
 * See also:
 * - `xcalibre-plugin-sdk/src/lib.rs` — ABI types and version constant
 * - `processing/src/plugins/loader.rs` — ZIP extraction logic
 * - `processing/src/db/plugin_queries.rs` — DB layer
 * - `src-tauri/src/commands.rs` — Tauri command wrappers
 */

import { useEffect, useState } from "react"
import { invoke, open as openDialog } from "@tauri-apps/api/core"

/** Minimal plugin shape returned by `list_plugins_cmd`. */
interface Plugin {
  id: string
  name: string
  version: string
  plugin_type: string  // "metadata_source" | "conversion_output" | "store"
  enabled: boolean
}

interface Props { onClose: () => void }

export function PluginManagerModal({ onClose }: Props) {
  const [plugins, setPlugins] = useState<Plugin[]>([])
  const [error, setError]     = useState<string | null>(null)

  /** Fetch the current plugin list from the database. */
  const reload = () =>
    invoke<Plugin[]>("list_plugins_cmd").then(setPlugins).catch(e => setError(String(e)))

  useEffect(() => { reload() }, [])

  /**
   * Toggle a plugin's enabled state.
   * The backend flips the current value, so we pass the *current* `enabled`
   * and the command inverts it.
   */
  const toggle = async (id: string, enabled: boolean) => {
    await invoke("set_plugin_enabled_cmd", { id, enabled: !enabled })
    reload()
  }

  /** Remove a plugin from the database and delete its dylib from disk. */
  const uninstall = async (id: string) => {
    await invoke("uninstall_plugin_cmd", { id })
    reload()
  }

  /**
   * Open the native file picker, let the user choose a `.zip`, and hand it to
   * the Tauri backend for validation and extraction.
   *
   * The backend rejects ZIPs whose `api_version` does not match the compiled
   * SDK version — the error is shown in the error banner.
   */
  const installZip = async () => {
    const path = await openDialog({ filters: [{ name: "Plugin ZIP", extensions: ["zip"] }] })
    if (!path) return
    try {
      await invoke("install_plugin_from_zip", { zipPath: path })
      reload()
    } catch (e) { setError(String(e)) }
  }

  return (
    <div data-testid="plugin-manager" className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-lg p-6">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg font-semibold dark:text-white">Plugins</h2>
          <button data-testid="plugin-manager-close" onClick={onClose} className="text-gray-400 hover:text-gray-600 text-xl">×</button>
        </div>
        {error && <p className="text-red-500 text-sm mb-3">{error}</p>}
        {plugins.length === 0 && <p className="text-gray-400 text-sm mb-4">No plugins installed.</p>}
        <ul className="space-y-2 mb-4">
          {plugins.map(p => (
            <li key={p.id} data-testid={`plugin-row-${p.id}`}
                className="flex items-center justify-between px-3 py-2 rounded border dark:border-gray-700">
              <div>
                <span className="font-medium text-sm dark:text-white">{p.name}</span>
                <span className="ml-2 text-xs text-gray-400">{p.version} · {p.plugin_type}</span>
              </div>
              <div className="flex gap-2">
                {/* Toggle button: green when enabled, grey when disabled */}
                <button onClick={() => toggle(p.id, p.enabled)}
                  className={`text-xs px-2 py-1 rounded ${p.enabled ? "bg-green-100 text-green-700" : "bg-gray-100 text-gray-500"}`}>
                  {p.enabled ? "Enabled" : "Disabled"}
                </button>
                <button onClick={() => uninstall(p.id)}
                  className="text-xs px-2 py-1 rounded bg-red-100 text-red-600 hover:bg-red-200">
                  Remove
                </button>
              </div>
            </li>
          ))}
        </ul>
        {/* Dashed drop-zone style button for installing a new plugin */}
        <button data-testid="install-plugin-btn" onClick={installZip}
          className="w-full text-sm border-2 border-dashed border-gray-300 dark:border-gray-600
                     rounded-lg py-2 text-gray-500 hover:border-blue-400 hover:text-blue-500">
          + Install plugin from ZIP…
        </button>
      </div>
    </div>
  )
}
