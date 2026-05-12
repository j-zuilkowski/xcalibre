import { useState, useEffect, useCallback } from "react"
import { invoke } from "@tauri-apps/api/core"

interface NoteRow {
  id: string
  book_id: string
  title: string
  body_html: string
  body_text: string
  created_at: string
  updated_at: string
}

interface Props {
  bookId: string
  onSelectNote: (note: NoteRow | null) => void
}

export function NotesList({ bookId, onSelectNote }: Props) {
  const [notes,   setNotes]   = useState<NoteRow[]>([])
  const [query,   setQuery]   = useState("")
  const [loading, setLoading] = useState(true)

  const loadNotes = useCallback(async () => {
    try {
      const rows = await invoke<NoteRow[]>("list_notes_cmd", { bookId })
      setNotes(rows)
    } catch (e) {
      console.error("NotesList load error:", e)
    } finally {
      setLoading(false)
    }
  }, [bookId])

  useEffect(() => { loadNotes() }, [loadNotes])

  useEffect(() => {
    if (!query.trim()) { loadNotes(); return }
    const t = setTimeout(async () => {
      try {
        const rows = await invoke<NoteRow[]>("search_notes_cmd", { bookId, query })
        setNotes(rows)
      } catch { loadNotes() }
    }, 300)
    return () => clearTimeout(t)
  }, [query, bookId, loadNotes])

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", gap: "0.5rem" }}>
      <div style={{ display: "flex", gap: "0.5rem", alignItems: "center" }}>
        <input
          data-testid="notes-search-input"
          value={query} onChange={e => setQuery(e.target.value)} placeholder="Search notes…"
          style={{ flex: 1, padding: "0.4rem 0.6rem", background: "var(--bg-overlay, #313244)", border: "1px solid var(--border, #45475a)", borderRadius: "6px", color: "inherit", fontSize: "0.9rem" }}
        />
        <button data-testid="add-note-btn" onClick={() => onSelectNote(null)}
          style={{ padding: "0.4rem 0.75rem", background: "var(--blue, #89b4fa)", border: "none", borderRadius: "6px", cursor: "pointer", color: "#1e1e2e", fontWeight: 600, whiteSpace: "nowrap" }}>
          + Add
        </button>
      </div>
      {loading ? (
        <p style={{ color: "var(--text-muted, #6c7086)", fontSize: "0.9rem" }}>Loading…</p>
      ) : notes.length === 0 ? (
        <p style={{ color: "var(--text-muted, #6c7086)", fontSize: "0.9rem" }}>
          {query ? "No notes match your search." : "No notes yet. Click + Add to create one."}
        </p>
      ) : (
        <ul style={{ listStyle: "none", padding: 0, margin: 0, overflow: "auto" }}>
          {notes.map(note => (
            <li key={note.id} onClick={() => onSelectNote(note)}
              style={{ padding: "0.6rem 0.75rem", borderRadius: "6px", cursor: "pointer", marginBottom: "0.25rem" }}
              onMouseEnter={e => (e.currentTarget.style.background = "var(--bg-overlay, #313244)")}
              onMouseLeave={e => (e.currentTarget.style.background = "transparent")}>
              <div style={{ fontWeight: 500 }}>{note.title}</div>
              <div style={{ fontSize: "0.8rem", color: "var(--text-muted, #6c7086)" }}>
                {new Date(note.updated_at).toLocaleDateString()}
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
