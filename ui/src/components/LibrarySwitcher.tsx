import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"

export interface Library {
  id: string
  name: string
  db_path: string
  layout: string
  is_active: boolean
}

interface Props {
  onSwitch: (lib: Library) => void
  onCreateNew: () => void
}

export function LibrarySwitcher({ onSwitch, onCreateNew }: Props) {
  const [libraries, setLibraries] = useState<Library[]>([])
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    setLoading(true)
    invoke<Library[]>("list_libraries_cmd")
      .then(setLibraries)
      .finally(() => setLoading(false))
  }, [])

  if (loading) return <div data-testid="switcher-loading">Loading libraries…</div>

  return (
    <div data-testid="library-switcher" className="p-4">
      <h2 className="text-base font-semibold mb-3 dark:text-white">Libraries</h2>
      <ul className="space-y-1 mb-4">
        {libraries.map((lib) => (
          <li key={lib.id}>
            <button
              data-testid={`library-item-${lib.id}`}
              onClick={() => onSwitch(lib)}
              className={`w-full text-left px-3 py-2 rounded text-sm transition-colors
                ${lib.is_active
                  ? "bg-blue-600 text-white"
                  : "hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-200"}`}
            >
              <span className="font-medium">{lib.name}</span>
              <span className="ml-2 text-xs opacity-60">{lib.layout}</span>
            </button>
          </li>
        ))}
      </ul>
      <button
        data-testid="create-library-btn"
        onClick={onCreateNew}
        className="w-full text-center text-sm text-blue-600 dark:text-blue-400 hover:underline"
      >
        + New Library
      </button>
    </div>
  )
}
