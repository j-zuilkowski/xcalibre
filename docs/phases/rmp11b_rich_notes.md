# RMP-11b — Rich Notes (Green: Implementation)

> Prerequisite: rmp11a complete.
> TDD role: GREEN — implement notes storage, Tauri commands, and editor UI.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R11b-T01 | `db/notes_queries.rs` — CRUD + FTS search | ⬜ |
| R11b-T02 | Tauri commands for notes | ⬜ |
| R11b-T03 | `NotesEditor.tsx` — rich text editor | ⬜ |
| R11b-T04 | `NotesList.tsx` — notes sidebar panel | ⬜ |
| R11b-T05 | Wire notes into BookDetailPanel | ⬜ |
| R11b-T06 | Milestone check + visual inspection | ⬜ |

---

## R11b-T01

Write `processing/src/db/notes_queries.rs`:
```rust
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NoteRow {
    pub id:         String,
    pub book_id:    String,
    pub title:      String,
    pub body_html:  String,
    pub body_text:  String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewNote {
    pub book_id:   String,
    pub title:     String,
    pub body_html: String,
    pub body_text: String,
}

pub async fn create_note(
    pool: &SqlitePool,
    new: &NewNote,
) -> Result<NoteRow, sqlx::Error> {
    sqlx::query_as::<_, NoteRow>(
        "INSERT INTO notes (book_id, title, body_html, body_text)
         VALUES (?, ?, ?, ?)
         RETURNING *"
    )
    .bind(&new.book_id)
    .bind(&new.title)
    .bind(&new.body_html)
    .bind(&new.body_text)
    .fetch_one(pool)
    .await
}

pub async fn list_notes(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Vec<NoteRow>, sqlx::Error> {
    sqlx::query_as::<_, NoteRow>(
        "SELECT * FROM notes WHERE book_id=? ORDER BY updated_at DESC"
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
}

pub async fn get_note(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<NoteRow>, sqlx::Error> {
    sqlx::query_as::<_, NoteRow>("SELECT * FROM notes WHERE id=?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn update_note(
    pool: &SqlitePool,
    id: &str,
    title: &str,
    body_html: &str,
    body_text: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE notes SET title=?, body_html=?, body_text=?, updated_at=datetime('now')
         WHERE id=?"
    )
    .bind(title)
    .bind(body_html)
    .bind(body_text)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_note(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM notes WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn search_notes(
    pool: &SqlitePool,
    book_id: &str,
    query: &str,
) -> Result<Vec<NoteRow>, sqlx::Error> {
    // FTS5 match joined back to notes table
    sqlx::query_as::<_, NoteRow>(
        "SELECT n.* FROM notes n
         JOIN notes_fts f ON n.rowid = f.rowid
         WHERE n.book_id=? AND notes_fts MATCH ?
         ORDER BY n.updated_at DESC"
    )
    .bind(book_id)
    .bind(query)
    .fetch_all(pool)
    .await
}
```

Add to `processing/src/db/mod.rs`:
```rust
pub mod notes_queries;
```

Then run:
```bash
cargo test --workspace -- test_notes
git add processing/src/db/notes_queries.rs processing/src/db/mod.rs
git commit -m "R11b-T01: notes CRUD + FTS search — all notes DB tests green"
```

---

## R11b-T02

Write `src-tauri/src/commands/notes.rs`:
```rust
use tauri::State;
use xcalibre_processing::db::notes_queries::{
    create_note, delete_note, get_note, list_notes, search_notes,
    update_note, NewNote, NoteRow,
};
use crate::AppState;

#[tauri::command]
pub async fn list_notes_cmd(
    book_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<NoteRow>, String> {
    list_notes(&state.pool, &book_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_note_cmd(
    id: String,
    state: State<'_, AppState>,
) -> Result<Option<NoteRow>, String> {
    get_note(&state.pool, &id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_note_cmd(
    book_id:   String,
    title:     String,
    body_html: String,
    body_text: String,
    state: State<'_, AppState>,
) -> Result<NoteRow, String> {
    let new = NewNote { book_id, title, body_html, body_text };
    create_note(&state.pool, &new).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_note_cmd(
    id:        String,
    title:     String,
    body_html: String,
    body_text: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    update_note(&state.pool, &id, &title, &body_html, &body_text)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_note_cmd(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    delete_note(&state.pool, &id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_notes_cmd(
    book_id: String,
    query:   String,
    state: State<'_, AppState>,
) -> Result<Vec<NoteRow>, String> {
    search_notes(&state.pool, &book_id, &query).await.map_err(|e| e.to_string())
}
```

Register all commands in `src-tauri/src/main.rs`:
```rust
commands::notes::list_notes_cmd,
commands::notes::get_note_cmd,
commands::notes::create_note_cmd,
commands::notes::update_note_cmd,
commands::notes::delete_note_cmd,
commands::notes::search_notes_cmd,
```

```bash
cargo build --workspace
git add src-tauri/src/commands/notes.rs src-tauri/src/main.rs
git commit -m "R11b-T02: Tauri commands for notes CRUD and search"
```

---

## R11b-T03

Install the rich-text editor dependency:
```bash
cd ui && npm install @tiptap/react @tiptap/starter-kit && cd ..
```

Write `ui/src/components/NotesEditor.tsx`:
```tsx
import { useState, useCallback } from "react"
import { invoke } from "@tauri-apps/api/core"
import { useEditor, EditorContent } from "@tiptap/react"
import StarterKit from "@tiptap/starter-kit"

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
  const [saving,  setSaving]  = useState(false)
  const [deleting,setDeleting]= useState(false)

  const editor = useEditor({
    extensions: [StarterKit],
    content:    note?.body_html ?? "",
  })

  const handleSave = useCallback(async () => {
    const body_html = editor?.getHTML() ?? ""
    const body_text = editor?.getText() ?? ""
    setSaving(true)
    try {
      if (note) {
        await invoke("update_note_cmd", {
          id: note.id, title: title.trim() || "Untitled Note", bodyHtml: body_html, bodyText: body_text,
        })
        onSave({ ...note, title: title.trim() || "Untitled Note", body_html, body_text })
      } else {
        const created = await invoke<NoteRow>("create_note_cmd", {
          bookId, title: title.trim() || "Untitled Note", bodyHtml: body_html, bodyText: body_text,
        })
        onSave(created)
      }
    } catch (e) {
      console.error("NotesEditor save error:", e)
    } finally {
      setSaving(false)
    }
  }, [editor, title, note, bookId, onSave])

  async function handleDelete() {
    if (!note) return
    setDeleting(true)
    try {
      await invoke("delete_note_cmd", { id: note.id })
      onDelete(note.id)
    } catch (e) {
      console.error("NotesEditor delete error:", e)
    } finally {
      setDeleting(false)
    }
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", gap: "0.5rem" }}>
      <input
        data-testid="note-title-input"
        value={title}
        onChange={e => setTitle(e.target.value)}
        placeholder="Note title"
        style={{
          padding: "0.5rem 0.75rem",
          background: "var(--bg-overlay, #313244)",
          border: "1px solid var(--border, #45475a)",
          borderRadius: "6px", color: "inherit", fontSize: "1rem",
        }}
      />

      <div
        data-testid="note-editor-area"
        style={{
          flex: 1, overflow: "auto",
          background: "var(--bg-overlay, #313244)",
          border: "1px solid var(--border, #45475a)",
          borderRadius: "6px", padding: "0.75rem",
          minHeight: "200px",
        }}
      >
        <EditorContent editor={editor} />
      </div>

      <div style={{ display: "flex", gap: "0.5rem", justifyContent: "flex-end" }}>
        {note && (
          <button
            data-testid="note-delete-btn"
            onClick={handleDelete}
            disabled={deleting}
            style={{
              padding: "0.4rem 1rem",
              background: "var(--red-dim, #3a1e1e)",
              border: "1px solid var(--red, #f38ba8)",
              borderRadius: "6px", cursor: "pointer",
              color: "var(--red, #f38ba8)",
              opacity: deleting ? 0.6 : 1,
            }}
          >
            {deleting ? "Deleting…" : "Delete"}
          </button>
        )}
        <button
          data-testid="note-save-btn"
          onClick={handleSave}
          disabled={saving}
          style={{
            padding: "0.4rem 1rem",
            background: "var(--blue, #89b4fa)",
            border: "none", borderRadius: "6px", cursor: "pointer",
            color: "#1e1e2e", fontWeight: 600,
            opacity: saving ? 0.6 : 1,
          }}
        >
          {saving ? "Saving…" : "Save"}
        </button>
      </div>
    </div>
  )
}
```

Then run:
```bash
cd ui && npm test -- NotesEditor && cd ..
git add ui/src/components/NotesEditor.tsx
git commit -m "R11b-T03: NotesEditor component — all UI tests green"
```

---

## R11b-T04

Write `ui/src/components/NotesList.tsx`:
```tsx
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
          value={query}
          onChange={e => setQuery(e.target.value)}
          placeholder="Search notes…"
          style={{
            flex: 1, padding: "0.4rem 0.6rem",
            background: "var(--bg-overlay, #313244)",
            border: "1px solid var(--border, #45475a)",
            borderRadius: "6px", color: "inherit", fontSize: "0.9rem",
          }}
        />
        <button
          data-testid="add-note-btn"
          onClick={() => onSelectNote(null)}
          style={{
            padding: "0.4rem 0.75rem",
            background: "var(--blue, #89b4fa)",
            border: "none", borderRadius: "6px", cursor: "pointer",
            color: "#1e1e2e", fontWeight: 600, whiteSpace: "nowrap",
          }}
        >
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
            <li
              key={note.id}
              onClick={() => onSelectNote(note)}
              style={{
                padding: "0.6rem 0.75rem", borderRadius: "6px", cursor: "pointer",
                marginBottom: "0.25rem",
              }}
              onMouseEnter={e => (e.currentTarget.style.background = "var(--bg-overlay, #313244)")}
              onMouseLeave={e => (e.currentTarget.style.background = "transparent")}
            >
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
```

Then run:
```bash
cd ui && npm test -- NotesList && cd ..
git add ui/src/components/NotesList.tsx
git commit -m "R11b-T04: NotesList component — all UI tests green"
```

---

## R11b-T05

In `ui/src/components/BookDetailPanel.tsx` (or wherever book details are shown), add a Notes tab:

```tsx
// Add to imports:
import { NotesList } from "./NotesList"
import { NotesEditor } from "./NotesEditor"

// In component state:
const [activeTab, setActiveTab]       = useState<"details" | "notes">("details")
const [selectedNote, setSelectedNote] = useState<NoteRow | null | undefined>(undefined)

// In JSX tab bar:
// <button onClick={() => setActiveTab("details")}>Details</button>
// <button onClick={() => setActiveTab("notes")}>Notes</button>

// In JSX panel body:
// {activeTab === "notes" && selectedNote === undefined && (
//   <NotesList bookId={book.id} onSelectNote={setSelectedNote} />
// )}
// {activeTab === "notes" && selectedNote !== undefined && (
//   <NotesEditor
//     bookId={book.id}
//     note={selectedNote}
//     onSave={() => setSelectedNote(undefined)}
//     onDelete={() => setSelectedNote(undefined)}
//   />
// )}
```

```bash
cargo build --workspace
cd ui && npm test && cd ..
git add ui/src/components/BookDetailPanel.tsx
git commit -m "R11b-T05: wire Notes tab into BookDetailPanel"
```

---

## R11b-T06 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm test && cd ..
```

**Visual inspection:**
```bash
pkill -x xcalibre 2>/dev/null || true
cargo tauri dev &>/tmp/xcalibre_tauri_dev.log &
# Poll until xcalibre process appears — first-run compilation can take 3-5 min
for i in $(seq 1 30); do
  sleep 10
  if pgrep -x xcalibre > /dev/null 2>&1; then
    echo "xcalibre running after $((i*10))s"
    sleep 3
    break
  fi
  echo "Waiting for xcalibre… $((i*10))s elapsed"
  [ "$i" -eq 30 ] && echo "ERROR: xcalibre did not launch within 5 minutes" && exit 1
done
```

1. Verify the UI:
2. Click a book to open its detail panel
3. Click the "Notes" tab — verify NotesList renders with "+ Add" button
4. Click "+ Add" — verify NotesEditor opens with empty title and body
5. Type a title and some text → Save — verify note appears in NotesList
6. Click the note → verify NotesEditor pre-fills with saved content
7. Edit and save — verify changes persist after switching away and back
8. Type in the search box — verify filtering works
9. Click Delete — verify note disappears from list

```bash
pkill -x xcalibre 2>/dev/null || true
```

```bash
git add -A
git commit -m "R11b-T06: RMP-11 Rich Notes — all tests green, UI wired"
```
