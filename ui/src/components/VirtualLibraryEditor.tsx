import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"

interface VirtualLibrary {
  id: string
  name: string
  search_expr: string
  sort_field: string
  sort_asc: boolean
}

interface Props {
  existing?: VirtualLibrary
  onClose: (created?: VirtualLibrary) => void
}

const SORT_FIELDS = [
  { value: "title",          label: "Title" },
  { value: "authors",        label: "Author" },
  { value: "added_at",       label: "Date Added" },
  { value: "last_opened_at", label: "Last Opened" },
  { value: "word_count",     label: "Word Count" },
]

export function VirtualLibraryEditor({ existing, onClose }: Props) {
  const [name,       setName]       = useState(existing?.name       ?? "")
  const [search,     setSearch]     = useState(existing?.search_expr ?? "")
  const [sortField,  setSortField]  = useState(existing?.sort_field  ?? "title")
  const [sortAsc,    setSortAsc]    = useState(existing?.sort_asc    ?? true)
  const [nameError,  setNameError]  = useState(false)
  const [searchError,setSearchError]= useState(false)
  const [saving,     setSaving]     = useState(false)

  async function handleSave() {
    let valid = true
    if (!name.trim())   { setNameError(true);   valid = false; } else { setNameError(false); }
    if (!search.trim()) { setSearchError(true);  valid = false; } else { setSearchError(false); }
    if (!valid) return

    setSaving(true)
    try {
      if (existing) {
        await invoke("update_virtual_library_cmd", {
          id: existing.id, name: name.trim(), searchExpr: search.trim(),
          sortField, sortAsc,
        })
        onClose()
      } else {
        const created = await invoke<VirtualLibrary>("create_virtual_library_cmd", {
          libraryId: null, name: name.trim(), searchExpr: search.trim(),
          sortField, sortAsc,
        })
        onClose(created)
      }
    } catch (e) {
      console.error("VirtualLibraryEditor save error:", e)
    } finally {
      setSaving(false)
    }
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      style={{
        position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)",
        display: "flex", alignItems: "center", justifyContent: "center",
        zIndex: 1000,
      }}
    >
      <div style={{
        background: "var(--bg-surface, #1e1e2e)", borderRadius: "12px",
        padding: "2rem", minWidth: "400px",
        color: "var(--text-primary, #cdd6f4)",
      }}>
        <h2 style={{ margin: "0 0 1.5rem" }}>
          {existing ? "Edit Virtual Library" : "New Virtual Library"}
        </h2>

        <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.9rem" }}>Name</label>
        <input
          data-testid="vlib-name-input"
          value={name}
          onChange={e => setName(e.target.value)}
          placeholder="e.g. Science Fiction"
          style={{
            width: "100%", padding: "0.5rem",
            background: "var(--bg-overlay, #313244)",
            border: `1px solid ${nameError ? "var(--red, #f38ba8)" : "var(--border, #45475a)"}`,
            borderRadius: "6px", color: "inherit", marginBottom: "0.25rem",
            boxSizing: "border-box",
          }}
        />
        {nameError && (
          <span data-testid="vlib-name-error" style={{ color: "var(--red, #f38ba8)", fontSize: "0.8rem" }}>
            Name is required.
          </span>
        )}

        <label style={{ display: "block", margin: "1rem 0 0.25rem", fontSize: "0.9rem" }}>Search Expression</label>
        <input
          data-testid="vlib-search-input"
          value={search}
          onChange={e => setSearch(e.target.value)}
          placeholder='e.g. tag:sci-fi author:asimov'
          style={{
            width: "100%", padding: "0.5rem",
            background: "var(--bg-overlay, #313244)",
            border: `1px solid ${searchError ? "var(--red, #f38ba8)" : "var(--border, #45475a)"}`,
            borderRadius: "6px", color: "inherit", marginBottom: "0.25rem",
            boxSizing: "border-box",
          }}
        />
        {searchError && (
          <span data-testid="vlib-search-error" style={{ color: "var(--red, #f38ba8)", fontSize: "0.8rem" }}>
            Search expression is required.
          </span>
        )}

        <div style={{ display: "flex", gap: "1rem", marginTop: "1rem" }}>
          <div style={{ flex: 1 }}>
            <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.9rem" }}>Sort by</label>
            <select
              value={sortField}
              onChange={e => setSortField(e.target.value)}
              style={{
                width: "100%", padding: "0.5rem",
                background: "var(--bg-overlay, #313244)",
                border: "1px solid var(--border, #45475a)",
                borderRadius: "6px", color: "inherit",
              }}
            >
              {SORT_FIELDS.map(f => (
                <option key={f.value} value={f.value}>{f.label}</option>
              ))}
            </select>
          </div>
          <div>
            <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.9rem" }}>Order</label>
            <select
              value={sortAsc ? "asc" : "desc"}
              onChange={e => setSortAsc(e.target.value === "asc")}
              style={{
                padding: "0.5rem",
                background: "var(--bg-overlay, #313244)",
                border: "1px solid var(--border, #45475a)",
                borderRadius: "6px", color: "inherit",
              }}
            >
              <option value="asc">A → Z</option>
              <option value="desc">Z → A</option>
            </select>
          </div>
        </div>

        <div style={{ display: "flex", gap: "0.75rem", justifyContent: "flex-end", marginTop: "1.5rem" }}>
          <button onClick={() => onClose()} style={{ padding: "0.5rem 1.25rem", background: "var(--bg-overlay, #313244)", border: "none", borderRadius: "6px", cursor: "pointer", color: "inherit" }}>
            Cancel
          </button>
          <button data-testid="vlib-save-btn" onClick={handleSave} disabled={saving} style={{ padding: "0.5rem 1.25rem", background: "var(--blue, #89b4fa)", border: "none", borderRadius: "6px", cursor: "pointer", color: "#1e1e2e", fontWeight: 600, opacity: saving ? 0.6 : 1 }}>
            {saving ? "Saving…" : "Save"}
          </button>
        </div>
      </div>
    </div>
  )
}
