# RMP-10a — Virtual Libraries & Saved Searches (Red: Failing Tests)

> Prerequisite: rmp02b complete (search parser needed for virtual library filter).
> TDD role: RED — define the virtual library DB schema and UI contract via failing tests.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R10a-T01 | Migration 0016 — virtual_libraries table | ⬜ |
| R10a-T02 | Failing tests: virtual library DB queries | ⬜ |
| R10a-T03 | Failing tests: VirtualLibraryEditor component | ⬜ |
| R10a-T04 | Failing tests: sidebar virtual library list | ⬜ |

---

## R10a-T01

Write `processing/src/db/migrations/0016_virtual_libraries.sql`:
```sql
-- Virtual libraries: saved searches that appear as dynamic collections.
CREATE TABLE IF NOT EXISTS virtual_libraries (
    id          TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    library_id  TEXT REFERENCES libraries(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    search_expr TEXT NOT NULL,   -- serialized QueryNode expression string
    sort_field  TEXT NOT NULL DEFAULT 'title',
    sort_asc    INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS vlib_library_id ON virtual_libraries(library_id);
```

```bash
cargo build --workspace
git add processing/src/db/migrations/0016_virtual_libraries.sql
git commit -m "R10a-T01: migration 0016 — virtual_libraries table"
```

---

## R10a-T02

Write `processing/tests/test_virtual_libraries.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::vlib_queries::{
    create_virtual_library, list_virtual_libraries, update_virtual_library,
    delete_virtual_library, VirtualLibrary, NewVirtualLibrary,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_create_and_list_virtual_libraries() {
    let pool = setup().await;
    let new = NewVirtualLibrary {
        library_id:  None,
        name:        "Science Fiction".into(),
        search_expr: "tag:science-fiction".into(),
        sort_field:  "title".into(),
        sort_asc:    true,
    };
    create_virtual_library(&pool, &new).await.expect("create");
    let libs = list_virtual_libraries(&pool, None).await.expect("list");
    assert_eq!(libs.len(), 1);
    assert_eq!(libs[0].name, "Science Fiction");
    assert_eq!(libs[0].search_expr, "tag:science-fiction");
}

#[tokio::test]
async fn test_update_virtual_library() {
    let pool = setup().await;
    let new = NewVirtualLibrary {
        library_id:  None,
        name:        "Old Name".into(),
        search_expr: "author:tolkien".into(),
        sort_field:  "title".into(),
        sort_asc:    true,
    };
    create_virtual_library(&pool, &new).await.unwrap();
    let libs = list_virtual_libraries(&pool, None).await.unwrap();
    let id = &libs[0].id.clone();

    update_virtual_library(&pool, id, "New Name", "author:tolkien", "added_at", false)
        .await.expect("update");

    let updated = list_virtual_libraries(&pool, None).await.unwrap();
    assert_eq!(updated[0].name, "New Name");
    assert_eq!(updated[0].sort_field, "added_at");
    assert!(!updated[0].sort_asc);
}

#[tokio::test]
async fn test_delete_virtual_library() {
    let pool = setup().await;
    let new = NewVirtualLibrary {
        library_id:  None,
        name:        "To Delete".into(),
        search_expr: "format:epub".into(),
        sort_field:  "title".into(),
        sort_asc:    true,
    };
    create_virtual_library(&pool, &new).await.unwrap();
    let libs = list_virtual_libraries(&pool, None).await.unwrap();
    let id = libs[0].id.clone();

    delete_virtual_library(&pool, &id).await.expect("delete");
    let after = list_virtual_libraries(&pool, None).await.unwrap();
    assert!(after.is_empty());
}

#[tokio::test]
async fn test_virtual_library_scoped_to_library() {
    let pool = setup().await;
    for (name, lib_id) in [("A", Some("lib1")), ("B", Some("lib2")), ("C", None)] {
        let new = NewVirtualLibrary {
            library_id:  lib_id.map(String::from),
            name:        name.into(),
            search_expr: "format:epub".into(),
            sort_field:  "title".into(),
            sort_asc:    true,
        };
        create_virtual_library(&pool, &new).await.unwrap();
    }
    let lib1_only = list_virtual_libraries(&pool, Some("lib1")).await.unwrap();
    assert_eq!(lib1_only.len(), 1);
    assert_eq!(lib1_only[0].name, "A");

    let global = list_virtual_libraries(&pool, None).await.unwrap();
    assert_eq!(global.len(), 3);
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

Expected: `db::vlib_queries` not found. RED confirmed.

```bash
git add processing/tests/test_virtual_libraries.rs
git commit -m "R10a-T02: failing tests for virtual library DB queries"
```

---

## R10a-T03

Write `ui/src/components/VirtualLibraryEditor.test.tsx`:
```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { VirtualLibraryEditor } from "./VirtualLibraryEditor"
import { mockInvoke } from "../test/setup"

beforeEach(() => {
  mockInvoke("list_virtual_libraries", [])
  mockInvoke("create_virtual_library", { id: "vl1", name: "New VLib", search_expr: "tag:fiction", sort_field: "title", sort_asc: true })
  mockInvoke("update_virtual_library", undefined)
  mockInvoke("delete_virtual_library", undefined)
})

describe("VirtualLibraryEditor", () => {
  it("renders name input and search expression input", () => {
    render(<VirtualLibraryEditor onClose={vi.fn()} />)
    expect(screen.getByTestId("vlib-name-input")).toBeInTheDocument()
    expect(screen.getByTestId("vlib-search-input")).toBeInTheDocument()
  })

  it("shows validation error when name is empty", async () => {
    render(<VirtualLibraryEditor onClose={vi.fn()} />)
    fireEvent.change(screen.getByTestId("vlib-search-input"), { target: { value: "tag:sci-fi" } })
    fireEvent.click(screen.getByTestId("vlib-save-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("vlib-name-error")).toBeInTheDocument()
    )
  })

  it("shows validation error when search expression is empty", async () => {
    render(<VirtualLibraryEditor onClose={vi.fn()} />)
    fireEvent.change(screen.getByTestId("vlib-name-input"), { target: { value: "My VLib" } })
    fireEvent.click(screen.getByTestId("vlib-save-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("vlib-search-error")).toBeInTheDocument()
    )
  })

  it("calls create_virtual_library with correct args on save", async () => {
    const onClose = vi.fn()
    render(<VirtualLibraryEditor onClose={onClose} />)
    fireEvent.change(screen.getByTestId("vlib-name-input"), { target: { value: "Sci-Fi" } })
    fireEvent.change(screen.getByTestId("vlib-search-input"), { target: { value: "tag:sci-fi" } })
    fireEvent.click(screen.getByTestId("vlib-save-btn"))
    await waitFor(() => expect(onClose).toHaveBeenCalled())
  })

  it("populates fields when editing existing virtual library", () => {
    const existing = { id: "vl1", name: "Fantasy", search_expr: "tag:fantasy", sort_field: "title", sort_asc: true }
    render(<VirtualLibraryEditor existing={existing} onClose={vi.fn()} />)
    expect(screen.getByTestId<HTMLInputElement>("vlib-name-input").value).toBe("Fantasy")
    expect(screen.getByTestId<HTMLInputElement>("vlib-search-input").value).toBe("tag:fantasy")
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/VirtualLibraryEditor.test.tsx
git commit -m "R10a-T03: failing tests for VirtualLibraryEditor component"
```

---

## R10a-T04

Write `ui/src/components/Sidebar.test.tsx` (add virtual library section):
```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { Sidebar } from "./Sidebar"
import { mockInvoke } from "../test/setup"

const mockVlibs = [
  { id: "vl1", name: "Science Fiction", search_expr: "tag:sci-fi", sort_field: "title", sort_asc: true },
  { id: "vl2", name: "Unread",          search_expr: "progress:0",  sort_field: "added_at", sort_asc: false },
]

beforeEach(() => {
  mockInvoke("list_virtual_libraries", mockVlibs)
  mockInvoke("list_libraries", [{ id: "lib1", name: "My Library", is_active: true }])
})

describe("Sidebar virtual libraries", () => {
  it("renders virtual library section header", async () => {
    render(<Sidebar onSelectFilter={vi.fn()} />)
    await waitFor(() =>
      expect(screen.getByTestId("vlib-section-header")).toBeInTheDocument()
    )
  })

  it("lists all virtual libraries", async () => {
    render(<Sidebar onSelectFilter={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByText("Science Fiction")).toBeInTheDocument()
      expect(screen.getByText("Unread")).toBeInTheDocument()
    })
  })

  it("calls onSelectFilter with search_expr when virtual library clicked", async () => {
    const onSelectFilter = vi.fn()
    render(<Sidebar onSelectFilter={onSelectFilter} />)
    await waitFor(() => screen.getByText("Science Fiction"))
    fireEvent.click(screen.getByText("Science Fiction"))
    expect(onSelectFilter).toHaveBeenCalledWith("tag:sci-fi")
  })

  it("shows add-virtual-library button", async () => {
    render(<Sidebar onSelectFilter={vi.fn()} />)
    await waitFor(() =>
      expect(screen.getByTestId("add-vlib-btn")).toBeInTheDocument()
    )
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/Sidebar.test.tsx
git commit -m "R10a-T04: failing tests for sidebar virtual library list"
```

---

### ✅ RED Checkpoint

```bash
cargo test --workspace 2>&1 | grep -c "^error"
cd ui && npm test 2>&1 | grep -c "FAIL" && cd ..
```

Both should show failures. Proceed to **rmp10b**.
