# RMP-20a — Custom Columns (Red: Failing Tests)

> Prerequisite: rmp01b complete (library support).
> TDD role: RED — define custom column schema and query API via failing tests.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R20a-T01 | Migration 0018 — custom_columns + book_custom_values tables | ⬜ |
| R20a-T02 | Failing tests: custom column CRUD | ⬜ |
| R20a-T03 | Failing tests: custom column value read/write | ⬜ |
| R20a-T04 | Failing tests: CustomColumnsEditor component | ⬜ |

---

## R20a-T01

Write `processing/src/db/migrations/0018_custom_columns.sql`:
```sql
-- Custom columns: user-defined metadata fields per library.
CREATE TABLE IF NOT EXISTS custom_columns (
    id          TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    library_id  TEXT REFERENCES libraries(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    label       TEXT NOT NULL,
    col_type    TEXT NOT NULL DEFAULT 'text'
                CHECK (col_type IN ('text','integer','float','bool','date','list')),
    is_multiple INTEGER NOT NULL DEFAULT 0,  -- allows multiple values (list type)
    display_in_grid INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX IF NOT EXISTS custom_columns_name_lib ON custom_columns(library_id, name);

-- Per-book values for each custom column.
CREATE TABLE IF NOT EXISTS book_custom_values (
    book_id   TEXT NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    column_id TEXT NOT NULL REFERENCES custom_columns(id) ON DELETE CASCADE,
    value     TEXT,
    PRIMARY KEY (book_id, column_id)
);
```

```bash
cargo build --workspace
git add processing/src/db/migrations/0018_custom_columns.sql
git commit -m "R20a-T01: migration 0018 — custom_columns + book_custom_values tables"
```

---

## R20a-T02

Write `processing/tests/test_custom_columns.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::custom_columns::{
    create_custom_column, list_custom_columns, update_custom_column,
    delete_custom_column, NewCustomColumn, CustomColumn,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_create_and_list_custom_columns() {
    let pool = setup().await;
    let new = NewCustomColumn {
        library_id:      None,
        name:            "read_status".into(),
        label:           "Read Status".into(),
        col_type:        "text".into(),
        is_multiple:     false,
        display_in_grid: true,
    };
    create_custom_column(&pool, &new).await.expect("create");
    let cols = list_custom_columns(&pool, None).await.expect("list");
    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "read_status");
    assert_eq!(cols[0].label, "Read Status");
}

#[tokio::test]
async fn test_column_types_accepted() {
    let pool = setup().await;
    for (i, col_type) in ["text","integer","float","bool","date","list"].iter().enumerate() {
        let new = NewCustomColumn {
            library_id: None, name: format!("col_{i}"),
            label: format!("Col {i}"), col_type: col_type.to_string(),
            is_multiple: false, display_in_grid: true,
        };
        create_custom_column(&pool, &new).await
            .expect(&format!("create {col_type} column"));
    }
    let cols = list_custom_columns(&pool, None).await.unwrap();
    assert_eq!(cols.len(), 6);
}

#[tokio::test]
async fn test_update_custom_column_label() {
    let pool = setup().await;
    let new = NewCustomColumn {
        library_id: None, name: "rating".into(), label: "Old Label".into(),
        col_type: "integer".into(), is_multiple: false, display_in_grid: true,
    };
    create_custom_column(&pool, &new).await.unwrap();
    let cols = list_custom_columns(&pool, None).await.unwrap();
    let id = cols[0].id.clone();

    update_custom_column(&pool, &id, "New Label", true).await.expect("update");
    let updated = list_custom_columns(&pool, None).await.unwrap();
    assert_eq!(updated[0].label, "New Label");
}

#[tokio::test]
async fn test_delete_custom_column() {
    let pool = setup().await;
    let new = NewCustomColumn {
        library_id: None, name: "to_delete".into(), label: "Delete Me".into(),
        col_type: "text".into(), is_multiple: false, display_in_grid: false,
    };
    create_custom_column(&pool, &new).await.unwrap();
    let cols = list_custom_columns(&pool, None).await.unwrap();
    let id = cols[0].id.clone();

    delete_custom_column(&pool, &id).await.expect("delete");
    let after = list_custom_columns(&pool, None).await.unwrap();
    assert!(after.is_empty());
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

RED confirmed.

```bash
git add processing/tests/test_custom_columns.rs
git commit -m "R20a-T02: failing tests for custom column CRUD"
```

---

## R20a-T03

Write `processing/tests/test_custom_values.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::custom_columns::{
    create_custom_column, NewCustomColumn,
    get_book_custom_value, set_book_custom_value, get_all_book_custom_values,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    // Insert a book
    sqlx::query(
        "INSERT INTO local_books (id, title, authors, format) VALUES ('b1', 'Dune', '[]', 'EPUB')"
    ).execute(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_set_and_get_custom_value() {
    let pool = setup().await;
    let col = create_custom_column(&pool, &NewCustomColumn {
        library_id: None, name: "my_rating".into(), label: "My Rating".into(),
        col_type: "integer".into(), is_multiple: false, display_in_grid: true,
    }).await.unwrap();

    set_book_custom_value(&pool, "b1", &col.id, Some("5")).await.expect("set");
    let val = get_book_custom_value(&pool, "b1", &col.id).await.expect("get");
    assert_eq!(val.as_deref(), Some("5"));
}

#[tokio::test]
async fn test_set_custom_value_to_null() {
    let pool = setup().await;
    let col = create_custom_column(&pool, &NewCustomColumn {
        library_id: None, name: "notes".into(), label: "Notes".into(),
        col_type: "text".into(), is_multiple: false, display_in_grid: false,
    }).await.unwrap();

    set_book_custom_value(&pool, "b1", &col.id, Some("initial")).await.unwrap();
    set_book_custom_value(&pool, "b1", &col.id, None).await.unwrap();
    let val = get_book_custom_value(&pool, "b1", &col.id).await.unwrap();
    assert!(val.is_none());
}

#[tokio::test]
async fn test_get_all_book_custom_values() {
    let pool = setup().await;
    let col1 = create_custom_column(&pool, &NewCustomColumn {
        library_id: None, name: "col1".into(), label: "C1".into(),
        col_type: "text".into(), is_multiple: false, display_in_grid: true,
    }).await.unwrap();
    let col2 = create_custom_column(&pool, &NewCustomColumn {
        library_id: None, name: "col2".into(), label: "C2".into(),
        col_type: "integer".into(), is_multiple: false, display_in_grid: true,
    }).await.unwrap();

    set_book_custom_value(&pool, "b1", &col1.id, Some("hello")).await.unwrap();
    set_book_custom_value(&pool, "b1", &col2.id, Some("42")).await.unwrap();

    let all = get_all_book_custom_values(&pool, "b1").await.expect("get all");
    assert_eq!(all.len(), 2);
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

RED confirmed.

```bash
git add processing/tests/test_custom_values.rs
git commit -m "R20a-T03: failing tests for custom column value read/write"
```

---

## R20a-T04

Write `ui/src/components/CustomColumnsEditor.test.tsx`:
```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { CustomColumnsEditor } from "./CustomColumnsEditor"
import { mockInvoke } from "../test/setup"

const mockColumns = [
  { id: "c1", name: "rating", label: "Rating", col_type: "integer", is_multiple: false, display_in_grid: true },
  { id: "c2", name: "status", label: "Status", col_type: "text",    is_multiple: false, display_in_grid: true },
]

beforeEach(() => {
  mockInvoke("list_custom_columns", mockColumns)
  mockInvoke("create_custom_column", { id: "c3", name: "new_col", label: "New Col", col_type: "text", is_multiple: false, display_in_grid: true })
  mockInvoke("update_custom_column", undefined)
  mockInvoke("delete_custom_column", undefined)
})

describe("CustomColumnsEditor", () => {
  it("lists existing custom columns", async () => {
    render(<CustomColumnsEditor onClose={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByText("Rating")).toBeInTheDocument()
      expect(screen.getByText("Status")).toBeInTheDocument()
    })
  })

  it("shows add-column button", async () => {
    render(<CustomColumnsEditor onClose={vi.fn()} />)
    expect(screen.getByTestId("add-column-btn")).toBeInTheDocument()
  })

  it("shows delete button for each column", async () => {
    render(<CustomColumnsEditor onClose={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getAllByTestId(/delete-column-/)).toHaveLength(2)
    })
  })

  it("shows column type for each column", async () => {
    render(<CustomColumnsEditor onClose={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByText("integer")).toBeInTheDocument()
    })
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/CustomColumnsEditor.test.tsx
git commit -m "R20a-T04: failing tests for CustomColumnsEditor component"
```

---

### ✅ RED Checkpoint

```bash
cargo test --workspace 2>&1 | grep -c "^error"
cd ui && npm test 2>&1 | grep -c "FAIL" && cd ..
```

Both should show failures. Proceed to **rmp20b**.
