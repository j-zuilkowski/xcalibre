import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Props {
  bookId: string
  initialTitle: string
  initialAuthors: string[]
  onSaved: () => void
}

export function MetadataEditorPanel({ bookId, initialTitle, initialAuthors, onSaved }: Props) {
  const [title, setTitle] = useState(initialTitle)
  const [authors, setAuthors] = useState(initialAuthors.join(", "))
  const [saving, setSaving] = useState(false)

  async function handleSave() {
    setSaving(true)
    try {
      await invoke("editor_update_metadata", { bookId, title, authors: authors.split(",").map(s => s.trim()).filter(Boolean) })
      onSaved()
    } finally { setSaving(false) }
  }

  return (
    <div style={{ padding: "1rem", display: "flex", flexDirection: "column", gap: "0.75rem" }}>
      <h3 style={{ margin: 0 }}>Metadata</h3>
      <label>Title <input data-testid="meta-title-input" value={title} onChange={e => setTitle(e.target.value)}
        style={{ display: "block", width: "100%", marginTop: "0.25rem", padding: "0.4rem", background: "var(--bg-overlay, #313244)", border: "1px solid var(--border, #45475a)", borderRadius: "4px", color: "inherit", boxSizing: "border-box" }} /></label>
      <label>Authors (comma-separated) <input data-testid="meta-authors-input" value={authors} onChange={e => setAuthors(e.target.value)}
        style={{ display: "block", width: "100%", marginTop: "0.25rem", padding: "0.4rem", background: "var(--bg-overlay, #313244)", border: "1px solid var(--border, #45475a)", borderRadius: "4px", color: "inherit", boxSizing: "border-box" }} /></label>
      <button data-testid="meta-save-btn" onClick={handleSave} disabled={saving}
        style={{ alignSelf: "flex-end", padding: "0.4rem 1rem", background: "var(--blue, #89b4fa)", border: "none", borderRadius: "6px", cursor: "pointer", color: "#1e1e2e", fontWeight: 600 }}>
        {saving ? "Saving…" : "Save Metadata"}
      </button>
    </div>
  )
}
