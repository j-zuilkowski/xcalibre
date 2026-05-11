# RMP-11a — Rich Notes (Red: Failing Tests)

> Prerequisite: rmp01b complete (library_id FK available).
> TDD role: RED — define the notes DB schema and editor contract via failing tests.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R11a-T01 | Migration 0017 — notes table | ⬜ |
| R11a-T02 | Failing tests: notes DB queries | ⬜ |
| R11a-T03 | Failing tests: NotesEditor component | ⬜ |
| R11a-T04 | Failing tests: NotesList component | ⬜ |

---

## R11a-T01

Write `processing/src/db/migrations/0017_notes.sql`:
```sql
-- Rich notes attached to a book.
CREATE TABLE IF NOT EXISTS notes (
    id          TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    book_id     TEXT NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    title       TEXT NOT NULL DEFAULT 'Untitled Note',
    body_html   TEXT NOT NULL DEFAULT '',
    body_text   TEXT NOT NULL DEFAULT '',  -- plain-text version for FTS
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS notes_book_id ON notes(book_id);
CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
    body_text, title,
    content='notes',
    content_rowid='rowid'
);
CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
    INSERT INTO notes_fts(rowid, body_text, title) VALUES (new.rowid, new.body_text, new.title);
END;
CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, body_text, title) VALUES ('delete', old.rowid, old.body_text, old.title);
    INSERT INTO notes_fts(rowid, body_text, title) VALUES (new.rowid, new.body_text, new.title);
END;
CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, body_text, title) VALUES ('delete', old.rowid, old.body_text, old.title);
END;
```

```bash
cargo build --workspace
git add processing/src/db/migrations/0017_notes.sql
git commit -m "R11a-T01: migration 0017 — notes table with FTS5"
```

---

## R11a-T02

Write `processing/tests/test_notes.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::notes_queries::{
    create_note, list_notes, get_note, update_note, delete_note,
    search_notes, NewNote, NoteRow,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_create_and_get_note() {
    let pool = setup().await;
    let new = NewNote {
        book_id:   "book1".into(),
        title:     "Chapter 1 Thoughts".into(),
        body_html: "<p>Great opening chapter.</p>".into(),
        body_text: "Great opening chapter.".into(),
    };
    create_note(&pool, &new).await.expect("create");
    let notes = list_notes(&pool, "book1").await.expect("list");
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "Chapter 1 Thoughts");
    assert_eq!(notes[0].body_html, "<p>Great opening chapter.</p>");
}

#[tokio::test]
async fn test_update_note() {
    let pool = setup().await;
    let new = NewNote {
        book_id:   "book1".into(),
        title:     "Original".into(),
        body_html: "<p>Old body.</p>".into(),
        body_text: "Old body.".into(),
    };
    create_note(&pool, &new).await.unwrap();
    let notes = list_notes(&pool, "book1").await.unwrap();
    let id = &notes[0].id.clone();

    update_note(&pool, id, "Updated", "<p>New body.</p>", "New body.")
        .await.expect("update");

    let note = get_note(&pool, id).await.unwrap().unwrap();
    assert_eq!(note.title, "Updated");
    assert_eq!(note.body_html, "<p>New body.</p>");
}

#[tokio::test]
async fn test_delete_note() {
    let pool = setup().await;
    let new = NewNote {
        book_id:   "book1".into(),
        title:     "To Delete".into(),
        body_html: "".into(),
        body_text: "".into(),
    };
    create_note(&pool, &new).await.unwrap();
    let notes = list_notes(&pool, "book1").await.unwrap();
    let id = notes[0].id.clone();

    delete_note(&pool, &id).await.expect("delete");
    let after = list_notes(&pool, "book1").await.unwrap();
    assert!(after.is_empty());
}

#[tokio::test]
async fn test_notes_scoped_to_book() {
    let pool = setup().await;
    for (book, title) in [("b1", "Note A"), ("b1", "Note B"), ("b2", "Note C")] {
        let new = NewNote {
            book_id:   book.into(),
            title:     title.into(),
            body_html: "".into(),
            body_text: "".into(),
        };
        create_note(&pool, &new).await.unwrap();
    }
    let b1 = list_notes(&pool, "b1").await.unwrap();
    let b2 = list_notes(&pool, "b2").await.unwrap();
    assert_eq!(b1.len(), 2);
    assert_eq!(b2.len(), 1);
}

#[tokio::test]
async fn test_search_notes_fts() {
    let pool = setup().await;
    for (title, text) in [("About Paul", "Paul Atreides leads"), ("About Sandworms", "Sandworms are massive")] {
        let new = NewNote {
            book_id:   "b1".into(),
            title:     title.into(),
            body_html: format!("<p>{text}</p>"),
            body_text: text.into(),
        };
        create_note(&pool, &new).await.unwrap();
    }
    let results = search_notes(&pool, "b1", "sandworms").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "About Sandworms");
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

Expected: `db::notes_queries` not found. RED confirmed.

```bash
git add processing/tests/test_notes.rs
git commit -m "R11a-T02: failing tests for notes DB queries"
```

---

## R11a-T03

Write `ui/src/components/NotesEditor.test.tsx`:
```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { NotesEditor } from "./NotesEditor"
import { mockInvoke } from "../test/setup"

const mockNote = {
  id: "n1", book_id: "b1", title: "My Note",
  body_html: "<p>Test content</p>", body_text: "Test content",
  created_at: "2026-01-01", updated_at: "2026-01-01",
}

beforeEach(() => {
  mockInvoke("update_note", undefined)
  mockInvoke("create_note", mockNote)
})

describe("NotesEditor", () => {
  it("renders title input and editor area", () => {
    render(<NotesEditor bookId="b1" note={mockNote} onSave={vi.fn()} onDelete={vi.fn()} />)
    expect(screen.getByTestId("note-title-input")).toBeInTheDocument()
    expect(screen.getByTestId("note-editor-area")).toBeInTheDocument()
  })

  it("populates title from note prop", () => {
    render(<NotesEditor bookId="b1" note={mockNote} onSave={vi.fn()} onDelete={vi.fn()} />)
    const input = screen.getByTestId<HTMLInputElement>("note-title-input")
    expect(input.value).toBe("My Note")
  })

  it("shows save button and calls update_note on save", async () => {
    const onSave = vi.fn()
    render(<NotesEditor bookId="b1" note={mockNote} onSave={onSave} onDelete={vi.fn()} />)
    fireEvent.click(screen.getByTestId("note-save-btn"))
    await waitFor(() => expect(onSave).toHaveBeenCalled())
  })

  it("shows delete button", () => {
    render(<NotesEditor bookId="b1" note={mockNote} onSave={vi.fn()} onDelete={vi.fn()} />)
    expect(screen.getByTestId("note-delete-btn")).toBeInTheDocument()
  })

  it("renders in new-note mode when note is null", () => {
    render(<NotesEditor bookId="b1" note={null} onSave={vi.fn()} onDelete={vi.fn()} />)
    const input = screen.getByTestId<HTMLInputElement>("note-title-input")
    expect(input.value).toBe("")
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/NotesEditor.test.tsx
git commit -m "R11a-T03: failing tests for NotesEditor component"
```

---

## R11a-T04

Write `ui/src/components/NotesList.test.tsx`:
```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { NotesList } from "./NotesList"
import { mockInvoke } from "../test/setup"

const mockNotes = [
  { id: "n1", book_id: "b1", title: "First Note",  body_html: "<p>Content A</p>", body_text: "Content A", created_at: "2026-01-01", updated_at: "2026-01-01" },
  { id: "n2", book_id: "b1", title: "Second Note", body_html: "<p>Content B</p>", body_text: "Content B", created_at: "2026-01-02", updated_at: "2026-01-02" },
]

beforeEach(() => {
  mockInvoke("list_notes", mockNotes)
  mockInvoke("delete_note", undefined)
  mockInvoke("search_notes", [mockNotes[0]])
})

describe("NotesList", () => {
  it("renders all notes for a book", async () => {
    render(<NotesList bookId="b1" onSelectNote={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByText("First Note")).toBeInTheDocument()
      expect(screen.getByText("Second Note")).toBeInTheDocument()
    })
  })

  it("calls onSelectNote when note clicked", async () => {
    const onSelectNote = vi.fn()
    render(<NotesList bookId="b1" onSelectNote={onSelectNote} />)
    await waitFor(() => screen.getByText("First Note"))
    fireEvent.click(screen.getByText("First Note"))
    expect(onSelectNote).toHaveBeenCalledWith(mockNotes[0])
  })

  it("shows add-note button", async () => {
    render(<NotesList bookId="b1" onSelectNote={vi.fn()} />)
    expect(screen.getByTestId("add-note-btn")).toBeInTheDocument()
  })

  it("filters notes via search", async () => {
    render(<NotesList bookId="b1" onSelectNote={vi.fn()} />)
    await waitFor(() => screen.getByTestId("notes-search-input"))
    fireEvent.change(screen.getByTestId("notes-search-input"), { target: { value: "Content A" } })
    await waitFor(() => {
      expect(screen.getByText("First Note")).toBeInTheDocument()
      expect(screen.queryByText("Second Note")).not.toBeInTheDocument()
    })
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/NotesList.test.tsx
git commit -m "R11a-T04: failing tests for NotesList component"
```

---

### ✅ RED Checkpoint

```bash
cargo test --workspace 2>&1 | grep -c "^error"
cd ui && npm test 2>&1 | grep -c "FAIL" && cd ..
```

Both should show failures. Proceed to **rmp11b**.
