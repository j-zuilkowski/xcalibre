# RMP-01b — Multiple Libraries (Green: Implementation)

> HOW TO USE: Write every file exactly as shown, run every shell block.
> DO NOT print code as output — write to disk using your tools.
> Prerequisite: rmp01a complete (migrations + failing tests committed).
> TDD role: GREEN — implement until `cargo test --workspace` is fully green.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R01b-T01 | `db/library_queries.rs` — CRUD functions | ⬜ |
| R01b-T02 | `config.rs` — LibraryConfig + LibraryEntry | ⬜ |
| R01b-T03 | `library/mod.rs` — managed_path() | ⬜ |
| R01b-T04 | Tauri commands: library management | ⬜ |
| R01b-T05 | LibrarySwitcher.tsx + tests | ⬜ |
| R01b-T06 | App.tsx startup flow: library selection | ⬜ |
| R01b-T07 | Milestone check + visual inspection | ⬜ |

---

## R01b-T01

In `processing/src/db/mod.rs`, add:
```rust
pub mod library_queries;
```

Write `processing/src/db/library_queries.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use sqlx::SqlitePool;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NewLibrary {
    pub name:      String,
    pub db_path:   String,
    pub cover_dir: String,
    pub layout:    String,
    pub xs_url:    Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct LibraryRow {
    pub id:         String,
    pub name:       String,
    pub db_path:    String,
    pub cover_dir:  String,
    pub layout:     String,
    pub xs_url:     Option<String>,
    pub is_active:  bool,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn create_library(pool: &SqlitePool, lib: &NewLibrary) -> Result<String, ProcessingError> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO libraries (id, name, db_path, cover_dir, layout, xs_url, created_at, updated_at)
         VALUES (?,?,?,?,?,?,?,?)",
    )
    .bind(&id).bind(&lib.name).bind(&lib.db_path).bind(&lib.cover_dir)
    .bind(&lib.layout).bind(&lib.xs_url).bind(&now).bind(&now)
    .execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(id)
}

pub async fn list_libraries(pool: &SqlitePool) -> Result<Vec<LibraryRow>, ProcessingError> {
    sqlx::query_as::<_, LibraryRow>(
        "SELECT id, name, db_path, cover_dir, layout, xs_url,
                CAST(is_active AS BOOLEAN) as is_active, created_at, updated_at
         FROM libraries ORDER BY created_at ASC",
    )
    .fetch_all(pool).await.map_err(ProcessingError::DbError)
}

pub async fn get_active_library(pool: &SqlitePool) -> Result<Option<LibraryRow>, ProcessingError> {
    sqlx::query_as::<_, LibraryRow>(
        "SELECT id, name, db_path, cover_dir, layout, xs_url,
                CAST(is_active AS BOOLEAN) as is_active, created_at, updated_at
         FROM libraries WHERE is_active = 1 LIMIT 1",
    )
    .fetch_optional(pool).await.map_err(ProcessingError::DbError)
}

pub async fn set_active_library(pool: &SqlitePool, id: &str) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE libraries SET is_active = 0, updated_at = ? WHERE is_active = 1")
        .bind(&now).execute(pool).await.map_err(ProcessingError::DbError)?;
    sqlx::query("UPDATE libraries SET is_active = 1, updated_at = ? WHERE id = ?")
        .bind(&now).bind(id).execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn update_library(
    pool: &SqlitePool, id: &str, name: &str, xs_url: Option<&str>,
) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE libraries SET name = ?, xs_url = ?, updated_at = ? WHERE id = ?")
        .bind(name).bind(xs_url).bind(&now).bind(id)
        .execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn delete_library(pool: &SqlitePool, id: &str) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM libraries WHERE id = ?")
        .bind(id).execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}
```

Then run:
```bash
cargo build --workspace
cargo test --workspace -- test_libraries
git add processing/src/db/library_queries.rs processing/src/db/mod.rs
git commit -m "R01b-T01: library CRUD queries — library tests green"
```

---

## R01b-T02

In `processing/src/lib.rs`, add `pub mod config;`.

Write `processing/src/config.rs` with this exact content:
```rust
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::error::ProcessingError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub id:        String,
    pub name:      String,
    pub db_path:   PathBuf,
    pub cover_dir: PathBuf,
    pub layout:    String,
    pub xs_url:    Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Inner {
    active_id:  Option<String>,
    libraries:  Vec<LibraryEntry>,
}

pub struct LibraryConfig {
    path:  PathBuf,
    inner: Inner,
}

impl LibraryConfig {
    pub fn new(path: PathBuf) -> Self {
        Self { path, inner: Inner { active_id: None, libraries: vec![] } }
    }

    pub fn load(path: PathBuf) -> Result<Self, ProcessingError> {
        let data = std::fs::read_to_string(&path)
            .map_err(ProcessingError::IoError)?;
        let inner: Inner = serde_json::from_str(&data)
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        Ok(Self { path, inner })
    }

    pub fn save(&self) -> Result<(), ProcessingError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(ProcessingError::IoError)?;
        }
        let data = serde_json::to_string_pretty(&self.inner)
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        std::fs::write(&self.path, data).map_err(ProcessingError::IoError)
    }

    pub fn add_library(&mut self, entry: LibraryEntry) {
        self.inner.libraries.push(entry);
    }

    pub fn set_active(&mut self, id: &str) {
        self.inner.active_id = Some(id.to_string());
    }

    pub fn active_id(&self) -> Option<&str> {
        self.inner.active_id.as_deref()
    }

    pub fn active_library(&self) -> Option<&LibraryEntry> {
        let id = self.inner.active_id.as_deref()?;
        self.inner.libraries.iter().find(|e| e.id == id)
    }

    pub fn libraries(&self) -> &[LibraryEntry] {
        &self.inner.libraries
    }

    pub fn remove_library(&mut self, id: &str) {
        self.inner.libraries.retain(|e| e.id != id);
        if self.inner.active_id.as_deref() == Some(id) {
            self.inner.active_id = self.inner.libraries.first().map(|e| e.id.clone());
        }
    }
}
```

Then run:
```bash
cargo test --workspace -- test_library_config
git add processing/src/config.rs processing/src/lib.rs
git commit -m "R01b-T02: LibraryConfig — config tests green"
```

---

## R01b-T03

In `processing/src/lib.rs`, add `pub mod library;`.

Write `processing/src/library/mod.rs` with this exact content:
```rust
use std::path::{Path, PathBuf};

/// Derive the managed on-disk path for a book.
/// Format: `<base>/<Author Last, First>/<Title (ID)>/`
/// Sanitizes path-unsafe characters to underscores.
pub fn managed_path(base: &Path, authors: &str, title: &str, book_id: i64) -> PathBuf {
    let author_dir = format_author(authors);
    let title_dir  = format_title(title, book_id);
    base.join(sanitize(&author_dir)).join(sanitize(&title_dir))
}

fn format_author(authors: &str) -> String {
    // Use first author; convert "First Last" → "Last, First"
    let primary = authors
        .split(['&', ','])
        .next()
        .unwrap_or(authors)
        .trim();
    let parts: Vec<&str> = primary.splitn(2, ' ').collect();
    if parts.len() == 2 {
        format!("{}, {}", parts[1].trim(), parts[0].trim())
    } else {
        primary.to_string()
    }
}

fn format_title(title: &str, id: i64) -> String {
    format!("{} ({})", title.trim(), id)
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn format_author_single() {
        assert_eq!(format_author("Terry Pratchett"), "Pratchett, Terry");
    }
    #[test]
    fn format_author_already_last_first() {
        assert_eq!(format_author("Pratchett"), "Pratchett");
    }
}
```

Then run:
```bash
cargo test --workspace -- test_managed_layout
cargo test --workspace -- library::tests
git add processing/src/library/mod.rs processing/src/lib.rs
git commit -m "R01b-T03: managed_path — managed layout tests green"
```

---

## R01b-T04

In `src-tauri/src/commands.rs`, append the following library management commands (do not remove existing commands):

```rust
use xcalibre_processing::db::library_queries::{
    create_library, delete_library, get_active_library,
    list_libraries, set_active_library, update_library, NewLibrary, LibraryRow,
};

#[tauri::command]
pub async fn list_libraries_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
) -> Result<Vec<LibraryRow>, String> {
    list_libraries(pool.inner().as_ref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_library_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    name: String,
    db_path: String,
    cover_dir: String,
    layout: String,
    xs_url: Option<String>,
) -> Result<String, String> {
    let lib = NewLibrary { name, db_path, cover_dir, layout, xs_url };
    create_library(pool.inner().as_ref(), &lib).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_active_library_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    id: String,
) -> Result<(), String> {
    set_active_library(pool.inner().as_ref(), &id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_active_library_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
) -> Result<Option<LibraryRow>, String> {
    get_active_library(pool.inner().as_ref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_library_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    id: String,
) -> Result<(), String> {
    delete_library(pool.inner().as_ref(), &id).await.map_err(|e| e.to_string())
}
```

In `src-tauri/src/main.rs`, add the new commands to `generate_handler!`:
```rust
commands::list_libraries_cmd,
commands::create_library_cmd,
commands::set_active_library_cmd,
commands::get_active_library_cmd,
commands::delete_library_cmd,
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "R01b-T04: Tauri library management commands"
```

---

## R01b-T05

Write `ui/src/components/LibrarySwitcher.tsx` with this exact content:
```tsx
import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"

export interface Library {
  id: string
  name: string
  db_path: string
  layout: string
  is_active: boolean
}

interface Props {
  onSwitch: (lib: Library) => void
  onCreateNew: () => void
}

export function LibrarySwitcher({ onSwitch, onCreateNew }: Props) {
  const [libraries, setLibraries] = useState<Library[]>([])
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    setLoading(true)
    invoke<Library[]>("list_libraries_cmd")
      .then(setLibraries)
      .finally(() => setLoading(false))
  }, [])

  if (loading) return <div data-testid="switcher-loading">Loading libraries…</div>

  return (
    <div data-testid="library-switcher" className="p-4">
      <h2 className="text-base font-semibold mb-3 dark:text-white">Libraries</h2>
      <ul className="space-y-1 mb-4">
        {libraries.map((lib) => (
          <li key={lib.id}>
            <button
              data-testid={`library-item-${lib.id}`}
              onClick={() => onSwitch(lib)}
              className={`w-full text-left px-3 py-2 rounded text-sm transition-colors
                ${lib.is_active
                  ? "bg-blue-600 text-white"
                  : "hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-200"}`}
            >
              <span className="font-medium">{lib.name}</span>
              <span className="ml-2 text-xs opacity-60">{lib.layout}</span>
            </button>
          </li>
        ))}
      </ul>
      <button
        data-testid="create-library-btn"
        onClick={onCreateNew}
        className="w-full text-center text-sm text-blue-600 dark:text-blue-400 hover:underline"
      >
        + New Library
      </button>
    </div>
  )
}
```

Write `ui/src/components/LibrarySwitcher.test.tsx` with this exact content:
```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { LibrarySwitcher, Library } from "./LibrarySwitcher"
import { mockInvoke } from "../test/setup"

const mockLibraries: Library[] = [
  { id: "a", name: "Main", db_path: "/tmp/a.db", layout: "in_place", is_active: true },
  { id: "b", name: "Research", db_path: "/tmp/b.db", layout: "managed", is_active: false },
]

beforeEach(() => {
  mockInvoke("list_libraries_cmd", mockLibraries)
})

describe("LibrarySwitcher", () => {
  it("renders library names after load", async () => {
    render(<LibrarySwitcher onSwitch={vi.fn()} onCreateNew={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByTestId("library-item-a")).toHaveTextContent("Main")
      expect(screen.getByTestId("library-item-b")).toHaveTextContent("Research")
    })
  })

  it("shows loading state initially", () => {
    mockInvoke("list_libraries_cmd", new Promise(() => {}))
    render(<LibrarySwitcher onSwitch={vi.fn()} onCreateNew={vi.fn()} />)
    expect(screen.getByTestId("switcher-loading")).toBeInTheDocument()
  })

  it("calls onSwitch when a library is clicked", async () => {
    const onSwitch = vi.fn()
    render(<LibrarySwitcher onSwitch={onSwitch} onCreateNew={vi.fn()} />)
    await waitFor(() => screen.getByTestId("library-item-b"))
    fireEvent.click(screen.getByTestId("library-item-b"))
    expect(onSwitch).toHaveBeenCalledWith(mockLibraries[1])
  })

  it("calls onCreateNew when + New Library is clicked", async () => {
    const onCreate = vi.fn()
    render(<LibrarySwitcher onSwitch={vi.fn()} onCreateNew={onCreate} />)
    await waitFor(() => screen.getByTestId("create-library-btn"))
    fireEvent.click(screen.getByTestId("create-library-btn"))
    expect(onCreate).toHaveBeenCalled()
  })
})
```

Then run:
```bash
cd ui && npm test -- --reporter=verbose 2>&1 | tail -20 && cd ..
cargo build --workspace
git add ui/src/components/LibrarySwitcher.tsx ui/src/components/LibrarySwitcher.test.tsx
git commit -m "R01b-T05: LibrarySwitcher component + tests"
```

---

## R01b-T06

Update `ui/src/App.tsx` to add a startup library check. On first launch (no libraries), show a "Create First Library" dialog. When libraries exist, show `LibrarySwitcher` in the sidebar.

The startup flow:
1. On mount, call `get_active_library_cmd`
2. If `null` returned → show `CreateLibraryModal` (collect name + layout choice)
3. Once a library is active → show normal `LibraryView`

Write `ui/src/components/CreateLibraryModal.tsx`:
```tsx
import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Props {
  onCreated: (id: string) => void
}

export function CreateLibraryModal({ onCreated }: Props) {
  const [name, setName] = useState("")
  const [layout, setLayout] = useState<"in_place" | "managed">("in_place")
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  const handleCreate = async () => {
    if (!name.trim()) { setError("Name is required"); return }
    setSaving(true)
    try {
      const dataDir = await invoke<string>("get_app_data_dir")
      const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, "_")
      const dbPath = `${dataDir}/libraries/${slug}.db`
      const coverDir = `${dataDir}/libraries/${slug}_covers`
      const id = await invoke<string>("create_library_cmd", {
        name: name.trim(), dbPath, coverDir, layout, xsUrl: null,
      })
      await invoke("set_active_library_cmd", { id })
      onCreated(id)
    } catch (e) {
      setError(String(e))
    } finally {
      setSaving(false)
    }
  }

  return (
    <div data-testid="create-library-modal"
         className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-sm p-6">
        <h2 className="text-lg font-semibold dark:text-white mb-4">Create your first library</h2>
        <label className="block text-sm text-gray-600 dark:text-gray-300 mb-1">Name</label>
        <input
          data-testid="library-name-input"
          value={name} onChange={e => setName(e.target.value)}
          className="w-full border rounded px-3 py-2 text-sm mb-4 dark:bg-gray-700 dark:text-white"
          placeholder="My Books"
        />
        <label className="block text-sm text-gray-600 dark:text-gray-300 mb-1">File layout</label>
        <select
          data-testid="layout-select"
          value={layout} onChange={e => setLayout(e.target.value as any)}
          className="w-full border rounded px-3 py-2 text-sm mb-4 dark:bg-gray-700 dark:text-white"
        >
          <option value="in_place">In-place (keep files where they are)</option>
          <option value="managed">Managed (copy into Author/Title/ folders)</option>
        </select>
        {error && <p data-testid="create-error" className="text-red-500 text-sm mb-3">{error}</p>}
        <button
          data-testid="confirm-create-btn"
          onClick={handleCreate} disabled={saving}
          className="w-full bg-blue-600 text-white rounded px-4 py-2 text-sm font-medium
                     hover:bg-blue-700 disabled:opacity-50"
        >
          {saving ? "Creating…" : "Create Library"}
        </button>
      </div>
    </div>
  )
}
```

Write `ui/src/components/CreateLibraryModal.test.tsx`:
```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { CreateLibraryModal } from "./CreateLibraryModal"
import { mockInvoke } from "../test/setup"

beforeEach(() => {
  mockInvoke("get_app_data_dir", "/tmp/test_app_data")
  mockInvoke("create_library_cmd", "new-lib-id")
  mockInvoke("set_active_library_cmd", undefined)
})

describe("CreateLibraryModal", () => {
  it("shows validation error when name is empty", async () => {
    render(<CreateLibraryModal onCreated={vi.fn()} />)
    fireEvent.click(screen.getByTestId("confirm-create-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("create-error")).toHaveTextContent("Name is required")
    )
  })

  it("calls onCreated with id after successful creation", async () => {
    const onCreated = vi.fn()
    render(<CreateLibraryModal onCreated={onCreated} />)
    fireEvent.change(screen.getByTestId("library-name-input"), { target: { value: "My Library" } })
    fireEvent.click(screen.getByTestId("confirm-create-btn"))
    await waitFor(() => expect(onCreated).toHaveBeenCalledWith("new-lib-id"))
  })

  it("defaults to in_place layout", () => {
    render(<CreateLibraryModal onCreated={vi.fn()} />)
    expect(screen.getByTestId("layout-select")).toHaveValue("in_place")
  })
})
```

Add `get_app_data_dir` Tauri command in `src-tauri/src/commands.rs`:
```rust
#[tauri::command]
pub async fn get_app_data_dir(
    app: tauri::AppHandle,
) -> Result<String, String> {
    app.path().app_data_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}
```

Register it in `main.rs` `generate_handler!`.

Then run:
```bash
cd ui && npm test -- --reporter=verbose 2>&1 | tail -30 && cd ..
cargo build --workspace
git add ui/src/components/CreateLibraryModal.tsx ui/src/components/CreateLibraryModal.test.tsx \
        src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "R01b-T06: CreateLibraryModal + startup flow + get_app_data_dir command"
```

---

## R01b-T07 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm run build && npm test && cd ..
```

All tests must be green. Zero clippy warnings.

**Visual inspection (launch the app):**
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

Open the Tauri dev window. Verify:
- [ ] On first launch (no libraries in DB): `CreateLibraryModal` appears
- [ ] Entering a name and clicking "Create Library" dismisses the modal and shows `LibraryView`
- [ ] Sidebar shows `LibrarySwitcher` with the created library highlighted
- [ ] Layout selector shows "In-place" / "Managed" options
- [ ] App title bar shows "xCalibre"

```bash
pkill -x xcalibre 2>/dev/null || true
```

```bash
git add -A
git commit -m "R01b-T07: RMP-01 multiple libraries — all tests green, visual verified"
```
