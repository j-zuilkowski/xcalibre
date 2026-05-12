import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"

interface CustomColumn {
  id: string; name: string; label: string; col_type: string
  is_multiple: boolean; display_in_grid: boolean
}

const COL_TYPES = ["text", "integer", "float", "bool", "date", "list"]

interface Props { onClose: () => void }

export function CustomColumnsEditor({ onClose }: Props) {
  const [columns, setColumns] = useState<CustomColumn[]>([])
  const [adding,  setAdding]  = useState(false)
  const [newName, setNewName] = useState("")
  const [newLabel,setNewLabel]= useState("")
  const [newType, setNewType] = useState("text")
  const [saving,  setSaving]  = useState(false)

  useEffect(() => {
    invoke<CustomColumn[]>("list_custom_columns_cmd", { libraryId: null })
      .then(setColumns)
      .catch(console.error)
  }, [])

  async function handleAdd() {
    if (!newName.trim() || !newLabel.trim()) return
    setSaving(true)
    try {
      const col = await invoke<CustomColumn>("create_custom_column_cmd", {
        libraryId: null, name: newName.trim(), label: newLabel.trim(),
        colType: newType, isMultiple: false, displayInGrid: true,
      })
      setColumns(prev => [...prev, col])
      setNewName(""); setNewLabel(""); setNewType("text"); setAdding(false)
    } catch (e) { console.error(e) }
    finally { setSaving(false) }
  }

  async function handleDelete(id: string) {
    await invoke("delete_custom_column_cmd", { id })
    setColumns(prev => prev.filter(c => c.id !== id))
  }

  return (
    <div role="dialog" aria-modal="true" style={{
      position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)",
      display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000,
    }}>
      <div style={{
        background: "var(--bg-surface, #1e1e2e)", borderRadius: "12px",
        padding: "2rem", minWidth: "480px", color: "var(--text-primary, #cdd6f4)",
      }}>
        <h2 style={{ margin: "0 0 1.5rem" }}>Custom Columns</h2>

        <table style={{ width: "100%", borderCollapse: "collapse", marginBottom: "1rem" }}>
          <thead>
            <tr style={{ borderBottom: "1px solid var(--border, #45475a)" }}>
              {["Name", "Label", "Type", ""].map(h => (
                <th key={h} style={{ textAlign: "left", padding: "0.4rem 0.5rem",
                                     fontSize: "0.85rem", color: "var(--text-muted, #6c7086)" }}>
                  {h}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {columns.map(col => (
              <tr key={col.id} style={{ borderBottom: "1px solid var(--border, #45475a)" }}>
                <td style={{ padding: "0.4rem 0.5rem", fontSize: "0.9rem" }}>{col.name}</td>
                <td style={{ padding: "0.4rem 0.5rem", fontSize: "0.9rem" }}>{col.label}</td>
                <td style={{ padding: "0.4rem 0.5rem", fontSize: "0.85rem",
                             color: "var(--text-muted, #6c7086)" }}>{col.col_type}</td>
                <td style={{ padding: "0.4rem 0.5rem" }}>
                  <button
                    data-testid={`delete-column-${col.id}`}
                    onClick={() => handleDelete(col.id)}
                    style={{
                      padding: "0.2rem 0.5rem", background: "transparent",
                      border: "1px solid var(--red, #f38ba8)", borderRadius: "4px",
                      cursor: "pointer", color: "var(--red, #f38ba8)", fontSize: "0.8rem",
                    }}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>

        {adding ? (
          <div style={{ display: "flex", gap: "0.5rem", marginBottom: "1rem", flexWrap: "wrap" }}>
            <input
              placeholder="name (no spaces)"
              value={newName}
              onChange={e => setNewName(e.target.value.replace(/\s/g, '_'))}
              style={{ padding: "0.4rem", background: "var(--bg-overlay, #313244)",
                       border: "1px solid var(--border, #45475a)", borderRadius: "4px",
                       color: "inherit", flex: "1 1 120px" }}
            />
            <input
              placeholder="Display Label"
              value={newLabel}
              onChange={e => setNewLabel(e.target.value)}
              style={{ padding: "0.4rem", background: "var(--bg-overlay, #313244)",
                       border: "1px solid var(--border, #45475a)", borderRadius: "4px",
                       color: "inherit", flex: "1 1 120px" }}
            />
            <select
              value={newType}
              onChange={e => setNewType(e.target.value)}
              style={{ padding: "0.4rem", background: "var(--bg-overlay, #313244)",
                       border: "1px solid var(--border, #45475a)", borderRadius: "4px",
                       color: "inherit" }}
            >
              {COL_TYPES.map(t => <option key={t} value={t}>{t}</option>)}
            </select>
            <button
              onClick={handleAdd}
              disabled={saving}
              style={{ padding: "0.4rem 0.75rem", background: "var(--blue, #89b4fa)",
                       border: "none", borderRadius: "4px", cursor: "pointer",
                       color: "#1e1e2e", fontWeight: 600 }}
            >
              Add
            </button>
            <button
              onClick={() => setAdding(false)}
              style={{ padding: "0.4rem 0.75rem", background: "var(--bg-overlay, #313244)",
                       border: "none", borderRadius: "4px", cursor: "pointer", color: "inherit" }}
            >
              Cancel
            </button>
          </div>
        ) : (
          <button
            data-testid="add-column-btn"
            onClick={() => setAdding(true)}
            style={{ padding: "0.4rem 1rem", background: "var(--bg-overlay, #313244)",
                     border: "1px solid var(--border, #45475a)", borderRadius: "6px",
                     cursor: "pointer", color: "inherit", marginBottom: "1rem" }}
          >
            + Add Column
          </button>
        )}

        <div style={{ display: "flex", justifyContent: "flex-end" }}>
          <button
            onClick={onClose}
            style={{ padding: "0.4rem 1rem", background: "var(--blue, #89b4fa)",
                     border: "none", borderRadius: "6px", cursor: "pointer",
                     color: "#1e1e2e", fontWeight: 600 }}
          >
            Done
          </button>
        </div>
      </div>
    </div>
  )
}
