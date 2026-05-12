import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { VirtualLibraryEditor } from "./VirtualLibraryEditor"

interface Props {
  onSelectFilter: (searchExpr: string) => void
}

interface VirtualLibrary {
  id: string
  name: string
  search_expr: string
  sort_field: string
  sort_asc: boolean
}

export function Sidebar({ onSelectFilter }: Props) {
  const [vlibs, setVlibs] = useState<VirtualLibrary[]>([])
  const [showEditor, setShowEditor] = useState(false)

  useEffect(() => {
    invoke<VirtualLibrary[]>("list_virtual_libraries_cmd", { libraryId: null })
      .then(setVlibs)
      .catch(console.error)
  }, [])

  function handleVlibCreated(vlib?: VirtualLibrary) {
    if (vlib) setVlibs(prev => [...prev, vlib].sort((a, b) => a.name.localeCompare(b.name)))
    setShowEditor(false)
  }

  return (
    <aside>
      <section>
        <header data-testid="vlib-section-header">Virtual Libraries</header>
        {vlibs.map(v => (
          <button key={v.id} onClick={() => onSelectFilter(v.search_expr)}>
            {v.name}
          </button>
        ))}
        <button data-testid="add-vlib-btn" onClick={() => setShowEditor(true)}>+ Add</button>
      </section>
      {showEditor && <VirtualLibraryEditor onClose={handleVlibCreated} />}
    </aside>
  )
}
