# RMP-20b — Custom Columns (Green: Implementation)

> Prerequisite: rmp20a complete.
> TDD role: GREEN — implement custom column storage, queries, Tauri commands, and editor UI.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R20b-T01 | `db/custom_columns.rs` — full CRUD + value queries | ⬜ |
| R20b-T02 | Tauri commands for custom columns | ⬜ |
| R20b-T03 | `CustomColumnsEditor.tsx` — manage column definitions | ⬜ |
| R20b-T04 | Custom column values in BookDetailPanel | ⬜ |
| R20b-T05 | Milestone check + visual inspection | ⬜ |

---

## R20b-T01

Write `processing/src/db/custom_columns.rs`:
```rust
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CustomColumn {
    pub id:              String,
    pub library_id:      Option<String>,
    pub name:            String,
    pub label:           String,
    pub col_type:        String,
    pub is_multiple:     bool,
    pub display_in_grid: bool,
    pub created_at:      String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCustomColumn {
    pub library_id:      Option<String>,
    pub name:            String,
    pub label:           String,
    pub col_type:        String,
    pub is_multiple:     bool,
    pub display_in_grid: bool,
}

pub async fn create_custom_column(
    pool: &SqlitePool,
    new: &NewCustomColumn,
) -> Result<CustomColumn, sqlx::Error> {
    sqlx::query_as::<_, CustomColumn>(
        "INSERT INTO custom_columns (library_id, name, label, col_type, is_multiple, display_in_grid)
         VALUES (?, ?, ?, ?, ?, ?)
         RETURNING *"
    )
    .bind(&new.library_id)
    .bind(&new.name)
    .bind(&new.label)
    .bind(&new.col_type)
    .bind(new.is_multiple)
    .bind(new.display_in_grid)
    .fetch_one(pool)
    .await
}

pub async fn list_custom_columns(
    pool: &SqlitePool,
    library_id: Option<&str>,
) -> Result<Vec<CustomColumn>, sqlx::Error> {
    match library_id {
        Some(lid) => {
            sqlx::query_as::<_, CustomColumn>(
                "SELECT * FROM custom_columns WHERE library_id=? ORDER BY created_at"
            )
            .bind(lid)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, CustomColumn>(
                "SELECT * FROM custom_columns ORDER BY created_at"
            )
            .fetch_all(pool)
            .await
        }
    }
}

pub async fn update_custom_column(
    pool: &SqlitePool,
    id: &str,
    label: &str,
    display_in_grid: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE custom_columns SET label=?, display_in_grid=? WHERE id=?"
    )
    .bind(label)
    .bind(display_in_grid)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_custom_column(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM custom_columns WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_book_custom_value(
    pool: &SqlitePool,
    book_id: &str,
    column_id: &str,
    value: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO book_custom_values (book_id, column_id, value)
         VALUES (?, ?, ?)
         ON CONFLICT(book_id, column_id) DO UPDATE SET value=excluded.value"
    )
    .bind(book_id)
    .bind(column_id)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_book_custom_value(
    pool: &SqlitePool,
    book_id: &str,
    column_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT value FROM book_custom_values WHERE book_id=? AND column_id=?"
    )
    .bind(book_id)
    .bind(column_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|(v,)| v))
}

pub async fn get_all_book_custom_values(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<HashMap<String, Option<String>>, sqlx::Error> {
    let rows: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT column_id, value FROM book_custom_values WHERE book_id=?"
    )
    .bind(book_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().collect())
}
```

Add to `processing/src/db/mod.rs`:
```rust
pub mod custom_columns;
```

Then run:
```bash
cargo test --workspace -- test_custom_columns test_custom_values
git add processing/src/db/custom_columns.rs processing/src/db/mod.rs
git commit -m "R20b-T01: custom columns CRUD + value queries — all tests green"
```

---

## R20b-T02

Write `src-tauri/src/commands/custom_columns.rs`:
```rust
use tauri::State;
use xcalibre_processing::db::custom_columns::{
    create_custom_column, delete_custom_column, get_all_book_custom_values,
    get_book_custom_value, list_custom_columns, set_book_custom_value,
    update_custom_column, CustomColumn, NewCustomColumn,
};
use crate::AppState;
use std::collections::HashMap;

#[tauri::command]
pub async fn list_custom_columns_cmd(
    library_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<CustomColumn>, String> {
    list_custom_columns(&state.pool, library_id.as_deref())
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_custom_column_cmd(
    library_id: Option<String>, name: String, label: String,
    col_type: String, is_multiple: bool, display_in_grid: bool,
    state: State<'_, AppState>,
) -> Result<CustomColumn, String> {
    let new = NewCustomColumn { library_id, name, label, col_type, is_multiple, display_in_grid };
    create_custom_column(&state.pool, &new).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_custom_column_cmd(
    id: String, label: String, display_in_grid: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    update_custom_column(&state.pool, &id, &label, display_in_grid)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_custom_column_cmd(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    delete_custom_column(&state.pool, &id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_book_custom_values_cmd(
    book_id: String,
    state: State<'_, AppState>,
) -> Result<HashMap<String, Option<String>>, String> {
    get_all_book_custom_values(&state.pool, &book_id)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_book_custom_value_cmd(
    book_id: String, column_id: String, value: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    set_book_custom_value(&state.pool, &book_id, &column_id, value.as_deref())
        .await.map_err(|e| e.to_string())
}
```

Register all commands in `src-tauri/src/main.rs`:
```rust
commands::custom_columns::list_custom_columns_cmd,
commands::custom_columns::create_custom_column_cmd,
commands::custom_columns::update_custom_column_cmd,
commands::custom_columns::delete_custom_column_cmd,
commands::custom_columns::get_book_custom_values_cmd,
commands::custom_columns::set_book_custom_value_cmd,
```

```bash
cargo build --workspace
git add src-tauri/src/commands/custom_columns.rs src-tauri/src/main.rs
git commit -m "R20b-T02: Tauri commands for custom columns and values"
```

---

## R20b-T03

Write `ui/src/components/CustomColumnsEditor.tsx`:
```tsx
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
```

Then run:
```bash
cd ui && npm test -- CustomColumnsEditor && cd ..
git add ui/src/components/CustomColumnsEditor.tsx
git commit -m "R20b-T03: CustomColumnsEditor component — all UI tests green"
```

---

## R20b-T04

In `ui/src/components/BookDetailPanel.tsx`, add a "Custom" section that shows and edits custom column values:

```tsx
// Add to imports:
import { invoke } from "@tauri-apps/api/core"
import { useEffect, useState } from "react"

// Add to component:
const [customCols,   setCustomCols]   = useState<any[]>([])
const [customValues, setCustomValues] = useState<Record<string, string | null>>({})

useEffect(() => {
  invoke<any[]>("list_custom_columns_cmd", { libraryId: null }).then(setCustomCols)
  invoke<Record<string, string | null>>("get_book_custom_values_cmd", { bookId: book.id })
    .then(setCustomValues)
}, [book.id])

// In JSX, after existing metadata fields:
// {customCols.map(col => (
//   <div key={col.id}>
//     <label>{col.label}</label>
//     <input
//       value={customValues[col.id] ?? ""}
//       onChange={e => {
//         setCustomValues(prev => ({ ...prev, [col.id]: e.target.value }))
//         invoke("set_book_custom_value_cmd", { bookId: book.id, columnId: col.id, value: e.target.value })
//       }}
//     />
//   </div>
// ))}
```

```bash
cd ui && npm test && cd ..
git add ui/src/components/BookDetailPanel.tsx
git commit -m "R20b-T04: custom column values in BookDetailPanel"
```

---

## R20b-T05 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm test && cd ..
```

**Visual inspection:**
1. Launch the app: `cd src-tauri && cargo tauri dev`
2. Open Settings → Custom Columns
3. Click "+ Add Column" → name: "my_rating", label: "My Rating", type: "integer" → Add
4. Verify "My Rating" appears in the column list
5. Open a book's detail panel → scroll to Custom section → enter a rating value → verify it saves
6. Open another book → verify its custom field is empty (not carrying over)
7. Return to Custom Columns → Delete the column → verify it disappears from book detail panels

```bash
git add -A
git commit -m "R20b-T05: RMP-20 Custom Columns — all tests green, UI wired"
```
