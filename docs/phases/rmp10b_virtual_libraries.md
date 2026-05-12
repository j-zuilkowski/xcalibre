# RMP-10b — Virtual Libraries & Saved Searches (Green: Implementation)

> Prerequisite: rmp10a complete.
> TDD role: GREEN — implement virtual library storage, query execution, and UI.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R10b-T01 | `db/vlib_queries.rs` — CRUD implementation | ⬜ |
| R10b-T02 | `execute_virtual_library()` — run saved search | ⬜ |
| R10b-T03 | Tauri commands for virtual libraries | ⬜ |
| R10b-T04 | `VirtualLibraryEditor.tsx` component | ⬜ |
| R10b-T05 | Wire into `Sidebar.tsx` | ⬜ |
| R10b-T06 | Milestone check + visual inspection | ⬜ |

---

## R10b-T01

Write `processing/src/db/vlib_queries.rs`:
```rust
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VirtualLibrary {
    pub id:          String,
    pub library_id:  Option<String>,
    pub name:        String,
    pub search_expr: String,
    pub sort_field:  String,
    pub sort_asc:    bool,
    pub created_at:  String,
    pub updated_at:  String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewVirtualLibrary {
    pub library_id:  Option<String>,
    pub name:        String,
    pub search_expr: String,
    pub sort_field:  String,
    pub sort_asc:    bool,
}

pub async fn create_virtual_library(
    pool: &SqlitePool,
    new: &NewVirtualLibrary,
) -> Result<VirtualLibrary, sqlx::Error> {
    let row = sqlx::query_as::<_, VirtualLibrary>(
        "INSERT INTO virtual_libraries (library_id, name, search_expr, sort_field, sort_asc)
         VALUES (?, ?, ?, ?, ?)
         RETURNING *"
    )
    .bind(&new.library_id)
    .bind(&new.name)
    .bind(&new.search_expr)
    .bind(&new.sort_field)
    .bind(new.sort_asc)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_virtual_libraries(
    pool: &SqlitePool,
    library_id: Option<&str>,
) -> Result<Vec<VirtualLibrary>, sqlx::Error> {
    match library_id {
        Some(lid) => {
            sqlx::query_as::<_, VirtualLibrary>(
                "SELECT * FROM virtual_libraries WHERE library_id = ? ORDER BY name"
            )
            .bind(lid)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, VirtualLibrary>(
                "SELECT * FROM virtual_libraries ORDER BY name"
            )
            .fetch_all(pool)
            .await
        }
    }
}

pub async fn update_virtual_library(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    search_expr: &str,
    sort_field: &str,
    sort_asc: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE virtual_libraries
         SET name=?, search_expr=?, sort_field=?, sort_asc=?,
             updated_at=datetime('now')
         WHERE id=?"
    )
    .bind(name)
    .bind(search_expr)
    .bind(sort_field)
    .bind(sort_asc)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_virtual_library(
    pool: &SqlitePool,
    id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM virtual_libraries WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_virtual_library(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<VirtualLibrary>, sqlx::Error> {
    sqlx::query_as::<_, VirtualLibrary>(
        "SELECT * FROM virtual_libraries WHERE id=?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
```

Add to `processing/src/db/mod.rs`:
```rust
pub mod vlib_queries;
```

Then run:
```bash
cargo test --workspace -- test_virtual_libraries
git add processing/src/db/vlib_queries.rs processing/src/db/mod.rs
git commit -m "R10b-T01: virtual library CRUD — all DB tests green"
```

---

## R10b-T02

Write `processing/src/db/vlib_execute.rs`:
```rust
use crate::db::vlib_queries::VirtualLibrary;
use crate::search::execute::execute_query;
use crate::search::parse_query;
use sqlx::sqlite::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookRow {
    pub id:      String,
    pub title:   String,
    pub authors: String,
}

pub async fn execute_virtual_library(
    pool: &SqlitePool,
    vlib: &VirtualLibrary,
) -> Result<Vec<BookRow>, crate::error::ProcessingError> {
    let ast = parse_query(&vlib.search_expr)?;
    let ids = execute_query(pool, &ast).await?;

    if ids.is_empty() {
        return Ok(vec![]);
    }

    // Build a parameterized IN clause
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let order = if vlib.sort_asc { "ASC" } else { "DESC" };
    let sort_col = sanitize_sort_field(&vlib.sort_field);
    let sql = format!(
        "SELECT id, title, authors FROM local_books WHERE id IN ({placeholders}) ORDER BY {sort_col} {order}"
    );

    let mut q = sqlx::query_as::<_, BookRow>(&sql);
    for id in &ids { q = q.bind(id); }
    let rows = q.fetch_all(pool).await
        .map_err(|e| crate::error::ProcessingError::DatabaseError(e.to_string()))?;
    Ok(rows)
}

fn sanitize_sort_field(field: &str) -> &str {
    match field {
        "title" | "authors" | "added_at" | "last_opened_at" | "word_count" => field,
        _ => "title",
    }
}
```

Add to `processing/src/db/mod.rs`:
```rust
pub mod vlib_execute;
```

Then run:
```bash
cargo build --workspace
git add processing/src/db/vlib_execute.rs processing/src/db/mod.rs
git commit -m "R10b-T02: execute_virtual_library — query runner wired to search parser"
```

---

## R10b-T03

Write `src-tauri/src/commands/virtual_library.rs`:
```rust
use tauri::State;
use xcalibre_processing::db::vlib_queries::{
    create_virtual_library, delete_virtual_library, get_virtual_library,
    list_virtual_libraries, update_virtual_library, NewVirtualLibrary, VirtualLibrary,
};
use xcalibre_processing::db::vlib_execute::execute_virtual_library;
use crate::AppState;
use serde::{Deserialize, Serialize};

#[tauri::command]
pub async fn list_virtual_libraries_cmd(
    library_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<VirtualLibrary>, String> {
    list_virtual_libraries(&state.pool, library_id.as_deref())
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_virtual_library_cmd(
    library_id:  Option<String>,
    name:        String,
    search_expr: String,
    sort_field:  String,
    sort_asc:    bool,
    state: State<'_, AppState>,
) -> Result<VirtualLibrary, String> {
    let new = NewVirtualLibrary { library_id, name, search_expr, sort_field, sort_asc };
    create_virtual_library(&state.pool, &new)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_virtual_library_cmd(
    id:          String,
    name:        String,
    search_expr: String,
    sort_field:  String,
    sort_asc:    bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    update_virtual_library(&state.pool, &id, &name, &search_expr, &sort_field, sort_asc)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_virtual_library_cmd(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    delete_virtual_library(&state.pool, &id)
        .await.map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
pub struct VlibBookResult {
    pub id:      String,
    pub title:   String,
    pub authors: String,
}

#[tauri::command]
pub async fn run_virtual_library_cmd(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<VlibBookResult>, String> {
    let vlib = get_virtual_library(&state.pool, &id)
        .await.map_err(|e| e.to_string())?
        .ok_or_else(|| format!("virtual library {id} not found"))?;
    let rows = execute_virtual_library(&state.pool, &vlib)
        .await.map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| VlibBookResult { id: r.id, title: r.title, authors: r.authors }).collect())
}
```

Register all commands in `src-tauri/src/main.rs`:
```rust
commands::virtual_library::list_virtual_libraries_cmd,
commands::virtual_library::create_virtual_library_cmd,
commands::virtual_library::update_virtual_library_cmd,
commands::virtual_library::delete_virtual_library_cmd,
commands::virtual_library::run_virtual_library_cmd,
```

```bash
cargo build --workspace
git add src-tauri/src/commands/virtual_library.rs src-tauri/src/main.rs
git commit -m "R10b-T03: Tauri commands for virtual library CRUD and execution"
```

---

## R10b-T04

Write `ui/src/components/VirtualLibraryEditor.tsx`:
```tsx
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
          <button
            onClick={() => onClose()}
            style={{
              padding: "0.5rem 1.25rem",
              background: "var(--bg-overlay, #313244)",
              border: "none", borderRadius: "6px", cursor: "pointer", color: "inherit",
            }}
          >
            Cancel
          </button>
          <button
            data-testid="vlib-save-btn"
            onClick={handleSave}
            disabled={saving}
            style={{
              padding: "0.5rem 1.25rem",
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
    </div>
  )
}
```

Then run:
```bash
cd ui && npm test -- VirtualLibraryEditor && cd ..
git add ui/src/components/VirtualLibraryEditor.tsx
git commit -m "R10b-T04: VirtualLibraryEditor component — all UI tests green"
```

---

## R10b-T05

In `ui/src/components/Sidebar.tsx`, add virtual library support:

```tsx
// Add to imports:
import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { VirtualLibraryEditor } from "./VirtualLibraryEditor"

// Add to Sidebar component:
interface VirtualLibrary {
  id: string
  name: string
  search_expr: string
  sort_field: string
  sort_asc: boolean
}

// Inside the component:
const [vlibs, setVlibs] = useState<VirtualLibrary[]>([])
const [showEditor, setShowEditor] = useState(false)
const [editingVlib, setEditingVlib] = useState<VirtualLibrary | undefined>()

useEffect(() => {
  invoke<VirtualLibrary[]>("list_virtual_libraries_cmd", { libraryId: null })
    .then(setVlibs)
    .catch(console.error)
}, [])

function handleVlibCreated(vlib?: VirtualLibrary) {
  if (vlib) setVlibs(prev => [...prev, vlib].sort((a, b) => a.name.localeCompare(b.name)))
  setShowEditor(false)
  setEditingVlib(undefined)
}

// In JSX, add virtual library section:
// <section>
//   <header data-testid="vlib-section-header">Virtual Libraries</header>
//   {vlibs.map(v => (
//     <button key={v.id} onClick={() => onSelectFilter(v.search_expr)}>
//       {v.name}
//     </button>
//   ))}
//   <button data-testid="add-vlib-btn" onClick={() => setShowEditor(true)}>+ Add</button>
// </section>
// {showEditor && <VirtualLibraryEditor existing={editingVlib} onClose={handleVlibCreated} />}
```

Then run:
```bash
cd ui && npm test -- Sidebar && cd ..
git add ui/src/components/Sidebar.tsx
git commit -m "R10b-T05: Sidebar virtual library section — all Sidebar tests green"
```

---

## R10b-T06 — Milestone Check + Visual Inspection

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
2. In the sidebar, locate the "Virtual Libraries" section
3. Click "+ Add" — verify VirtualLibraryEditor dialog appears
4. Enter name "Science Fiction" and search expression "tag:sci-fi" → Save
5. Verify "Science Fiction" appears in the sidebar list
6. Click "Science Fiction" — verify book grid filters to matching books
7. Right-click virtual library → Edit — verify editor pre-fills with saved values
8. Delete a virtual library — verify it disappears from sidebar

```bash
pkill -x xcalibre 2>/dev/null || true
```

```bash
git add -A
git commit -m "R10b-T06: RMP-10 Virtual Libraries — all tests green, sidebar wired"
```
