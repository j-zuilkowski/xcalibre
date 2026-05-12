import { useState, useCallback } from "react"
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
  note: NoteRow | null
  onSave: (note: NoteRow) => void
  onDelete: (id: string) => void
}

export function NotesEditor({ bookId, note, onSave, onDelete }: Props) {
  const [title,   setTitle]   = useState(note?.title     ?? "")
  const [body,    setBody]    = useState(note?.body_text ?? "")
  const [saving,  setSaving]  = useState(false)
  const [deleting,setDeleting]= useState(false)

  const handleSave = useCallback(async () => {
    setSaving(true)
    const body_html = `<p>${body.replace(/\n/g, "</p><p>")}</p>`
    try {
      if (note) {
        await invoke("update_note_cmd", {
          id: note.id, title: title.trim() || "Untitled Note", bodyHtml: body_html, bodyText: body,
        })
        onSave({ ...note, title: title.trim() || "Untitled Note", body_html, body_text: body })
      } else {
        const created = await invoke<NoteRow>("create_note_cmd", {
          bookId, title: title.trim() || "Untitled Note", bodyHtml: body_html, bodyText: body,
        })
        onSave(created)
      }
    } catch (e) { console.error("NotesEditor save error:", e) } finally { setSaving(false) }
  }, [body, title, note, bookId, onSave])

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", gap: "0.5rem" }}>
      <input data-testid="note-title-input" value={title} onChange={e => setTitle(e.target.value)} placeholder="Note title"
        style={{ padding: "0.5rem 0.75rem", background: "var(--bg-overlay, #313244)", border: "1px solid var(--border, #45475a)", borderRadius: "6px", color: "inherit", fontSize: "1rem" }} />
      <textarea data-testid="note-editor-area" value={body} onChange={e => setBody(e.target.value)} placeholder="Write your note…"
        style={{ flex: 1, overflow: "auto", padding: "0.75rem", background: "var(--bg-overlay, #313244)", border: "1px solid var(--border, #45475a)", borderRadius: "6px", color: "inherit", minHeight: "200px", resize: "vertical" }} />
      <div style={{ display: "flex", gap: "0.5rem", justifyContent: "flex-end" }}>
        {note && (
          <button data-testid="note-delete-btn" onClick={async () => { if (!note) return; setDeleting(true); try { await invoke("delete_note_cmd", { id: note.id }); onDelete(note.id) } catch (e) { console.error(e) } finally { setDeleting(false) } }} disabled={deleting}
            style={{ padding: "0.4rem 1rem", background: "var(--red-dim, #3a1e1e)", border: "1px solid var(--red, #f38ba8)", borderRadius: "6px", cursor: "pointer", color: "var(--red, #f38ba8)", opacity: deleting ? 0.6 : 1 }}>
            {deleting ? "Deleting…" : "Delete"}
          </button>
        )}
        <button data-testid="note-save-btn" onClick={handleSave} disabled={saving}
          style={{ padding: "0.4rem 1rem", background: "var(--blue, #89b4fa)", border: "none", borderRadius: "6px", cursor: "pointer", color: "#1e1e2e", fontWeight: 600, opacity: saving ? 0.6 : 1 }}>
          {saving ? "Saving…" : "Save"}
        </button>
      </div>
    </div>
  )
}

