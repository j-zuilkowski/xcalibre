import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Props {
  onCreated: (id: string) => void
}

export function CreateLibraryModal({ onCreated }: Props) {
  const [name, setName] = useState("")
  const [layout, setLayout] = useState<"in_place" | "managed">("in_place")
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  const handleCreate = async () => {
    if (!name.trim()) { setError("Name is required"); return }
    setSaving(true)
    try {
      const dataDir = await invoke<string>("get_app_data_dir")
      const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, "_")
      const dbPath = `${dataDir}/libraries/${slug}.db`
      const coverDir = `${dataDir}/libraries/${slug}_covers`
      const id = await invoke<string>("create_library_cmd", {
        name: name.trim(), dbPath, coverDir, layout, xsUrl: null,
      })
      await invoke("set_active_library_cmd", { id })
      onCreated(id)
    } catch (e) {
      setError(String(e))
    } finally {
      setSaving(false)
    }
  }

  return (
    <div data-testid="create-library-modal"
         className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-sm p-6">
        <h2 className="text-lg font-semibold dark:text-white mb-4">Create your first library</h2>
        <label className="block text-sm text-gray-600 dark:text-gray-300 mb-1">Name</label>
        <input
          data-testid="library-name-input"
          value={name} onChange={e => setName(e.target.value)}
          className="w-full border rounded px-3 py-2 text-sm mb-4 dark:bg-gray-700 dark:text-white"
          placeholder="My Books"
        />
        <label className="block text-sm text-gray-600 dark:text-gray-300 mb-1">File layout</label>
        <select
          data-testid="layout-select"
          value={layout} onChange={e => setLayout(e.target.value as any)}
          className="w-full border rounded px-3 py-2 text-sm mb-4 dark:bg-gray-700 dark:text-white"
        >
          <option value="in_place">In-place (keep files where they are)</option>
          <option value="managed">Managed (copy into Author/Title/ folders)</option>
        </select>
        {error && <p data-testid="create-error" className="text-red-500 text-sm mb-3">{error}</p>}
        <button
          data-testid="confirm-create-btn"
          onClick={handleCreate} disabled={saving}
          className="w-full bg-blue-600 text-white rounded px-4 py-2 text-sm font-medium
                     hover:bg-blue-700 disabled:opacity-50"
        >
          {saving ? "Creating…" : "Create Library"}
        </button>
      </div>
    </div>
  )
}
