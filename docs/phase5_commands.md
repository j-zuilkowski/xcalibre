# Phase 5 — Library Management

> HOW TO USE: For every "Write `path`" line → call write_file with that path and content.
> For every "Then run:" block → call your shell tool for each command.
> DO NOT print code as output. Write it to disk using your tools.
> Prerequisite: Phase 4 complete and all tests green.
> Status: ✅ done

## Status

| Task | Title | Status |
|------|-------|--------|
| P5-T01 | Migration 0004 — extended schema | ✅ |
| P5-T02 | Extended DB queries (tags, identifiers) | ✅ |
| P5-T03 | Title sort / author sort utilities | ✅ |
| P5-T04 | Migration 0005 — book_formats | ✅ |
| P5-T05 | book_formats DB queries | ✅ |
| P5-T06 | Migration 0006 — FTS5 | ✅ |
| P5-T07 | FTS population function | ✅ |
| P5-T08 | Search Tauri command | ✅ |
| P5-T09 | SearchBar UI component | ✅ |
| P5-T10 | Filter backend + Tauri command | ✅ |
| P5-T11 | FilterBar UI component | ✅ |
| P5-T12 | Migration 0007 — collections | ✅ |
| P5-T13 | Collections DB CRUD | ✅ |
| P5-T14 | CollectionsSidebar UI | ✅ |
| P5-T15 | Bulk operations — Tauri commands | ✅ |
| P5-T16 | BulkActionBar UI | ✅ |
| P5-T17 | OPF sidecar parser | ✅ |
| P5-T18 | Calibre library import | ✅ |
| P5-T19 | Library integrity check | ✅ |
| P5-T20 | Catalog export (HTML + CSV) | ✅ |
| P5-T21 | Wire new modules into lib.rs | ✅ |
| P5-T22 | Tests | ✅ |

---

## P5-T01

Write `processing/src/db/migrations/0004_extended_schema.sql` with this exact content:
```sql
-- Additional scalar metadata columns on local_books
ALTER TABLE local_books ADD COLUMN title_sort   TEXT;
ALTER TABLE local_books ADD COLUMN author_sort  TEXT;
ALTER TABLE local_books ADD COLUMN pubdate      TEXT;
ALTER TABLE local_books ADD COLUMN description  TEXT;
ALTER TABLE local_books ADD COLUMN publisher    TEXT;
ALTER TABLE local_books ADD COLUMN series_name  TEXT;
ALTER TABLE local_books ADD COLUMN series_index REAL DEFAULT 1.0;
ALTER TABLE local_books ADD COLUMN rating       INTEGER DEFAULT 0;

-- Tags (flat, user-assigned)
CREATE TABLE IF NOT EXISTS tags (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE
);
CREATE TABLE IF NOT EXISTS book_tags (
    book_id TEXT    NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    tag_id  INTEGER NOT NULL REFERENCES tags(id)        ON DELETE CASCADE,
    PRIMARY KEY (book_id, tag_id)
);

-- Identifiers (isbn, uuid, asin, etc.)
CREATE TABLE IF NOT EXISTS identifiers (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id TEXT    NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    type    TEXT    NOT NULL,
    value   TEXT    NOT NULL,
    UNIQUE  (book_id, type)
);
```

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/db/migrations/0004_extended_schema.sql
git commit -m "P5-T01: migration 0004 — extended schema (tags, identifiers, sort, series, rating)"
```

---

## P5-T02

Write `processing/src/db/extended_queries.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use sqlx::SqlitePool;

// ── Tags ──────────────────────────────────────────────────────────────────────

pub async fn upsert_tag(
    pool: &SqlitePool,
    book_id: &str,
    tag: &str,
) -> Result<(), ProcessingError> {
    sqlx::query("INSERT OR IGNORE INTO tags (name) VALUES (?)")
        .bind(tag)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;

    sqlx::query(
        "INSERT OR IGNORE INTO book_tags (book_id, tag_id)
         SELECT ?, id FROM tags WHERE name = ?",
    )
    .bind(book_id)
    .bind(tag)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(())
}

pub async fn get_tags(pool: &SqlitePool, book_id: &str) -> Result<Vec<String>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT t.name FROM tags t
         JOIN book_tags bt ON bt.tag_id = t.id
         WHERE bt.book_id = ?
         ORDER BY t.name",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(rows.into_iter().map(|(name,)| name).collect())
}

pub async fn remove_tag(
    pool: &SqlitePool,
    book_id: &str,
    tag: &str,
) -> Result<(), ProcessingError> {
    sqlx::query(
        "DELETE FROM book_tags WHERE book_id = ?
         AND tag_id = (SELECT id FROM tags WHERE name = ?)",
    )
    .bind(book_id)
    .bind(tag)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}

// ── Identifiers ───────────────────────────────────────────────────────────────

pub async fn upsert_identifier(
    pool: &SqlitePool,
    book_id: &str,
    id_type: &str,
    value: &str,
) -> Result<(), ProcessingError> {
    sqlx::query(
        "INSERT INTO identifiers (book_id, type, value) VALUES (?, ?, ?)
         ON CONFLICT(book_id, type) DO UPDATE SET value = excluded.value",
    )
    .bind(book_id)
    .bind(id_type)
    .bind(value)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn get_identifiers(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Vec<(String, String)>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT type, value FROM identifiers WHERE book_id = ? ORDER BY type",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(rows)
}

// ── Extended metadata update ──────────────────────────────────────────────────

pub async fn update_extended_metadata(
    pool: &SqlitePool,
    book_id: &str,
    title_sort: Option<&str>,
    author_sort: Option<&str>,
    pubdate: Option<&str>,
    description: Option<&str>,
    publisher: Option<&str>,
    series_name: Option<&str>,
    series_index: Option<f64>,
    rating: Option<i64>,
) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE local_books SET
             title_sort   = COALESCE(?, title_sort),
             author_sort  = COALESCE(?, author_sort),
             pubdate      = COALESCE(?, pubdate),
             description  = COALESCE(?, description),
             publisher    = COALESCE(?, publisher),
             series_name  = COALESCE(?, series_name),
             series_index = COALESCE(?, series_index),
             rating       = COALESCE(?, rating),
             updated_at   = ?
         WHERE id = ?",
    )
    .bind(title_sort)
    .bind(author_sort)
    .bind(pubdate)
    .bind(description)
    .bind(publisher)
    .bind(series_name)
    .bind(series_index)
    .bind(rating)
    .bind(&now)
    .bind(book_id)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}
```

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/db/extended_queries.rs
git commit -m "P5-T02: extended DB queries — tags, identifiers, extended metadata update"
```

---

## P5-T03

Write `processing/src/utils/sort.rs` with this exact content:
```rust
static ARTICLES: &[&str] = &["the ", "a ", "an "];

/// Convert a display title to a sortable form.
/// "The Lord of the Rings" → "Lord of the Rings, The"
pub fn title_sort(title: &str) -> String {
    let lower = title.to_lowercase();
    for article in ARTICLES {
        if let Some(rest) = lower.strip_prefix(article) {
            let _ = rest; // only used for the check
            let split = article.len();
            let article_trimmed = title[..split].trim_end();
            let rest_display    = title[split..].trim_start();
            return format!("{}, {}", rest_display, article_trimmed);
        }
    }
    title.to_string()
}

/// Convert an author name to sortable "Last, First" form.
/// "J.R.R. Tolkien" → "Tolkien, J.R.R."
/// "Tolkien, J.R.R." is returned unchanged.
pub fn author_sort(name: &str) -> String {
    if name.contains(',') {
        return name.to_string();
    }
    let parts: Vec<&str> = name.trim().split_whitespace().collect();
    match parts.len() {
        0 => String::new(),
        1 => parts[0].to_string(),
        _ => {
            let last  = parts[parts.len() - 1];
            let first = parts[..parts.len() - 1].join(" ");
            format!("{}, {}", last, first)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_title_sort_article() {
        assert_eq!(title_sort("The Great Gatsby"), "Great Gatsby, The");
    }
    #[test] fn test_title_sort_no_article() {
        assert_eq!(title_sort("Dune"), "Dune");
    }
    #[test] fn test_author_sort_two_names() {
        assert_eq!(author_sort("Frank Herbert"), "Herbert, Frank");
    }
    #[test] fn test_author_sort_already_sorted() {
        assert_eq!(author_sort("Herbert, Frank"), "Herbert, Frank");
    }
}
```

In `processing/src/utils/mod.rs`, add `pub mod sort;` after the existing lines.

Then run:
```bash
cd processing && cargo test --lib && cd ..
git add processing/src/utils/sort.rs processing/src/utils/mod.rs
git commit -m "P5-T03: title_sort and author_sort utilities"
```

---

## P5-T04

Write `processing/src/db/migrations/0005_book_formats.sql` with this exact content:
```sql
-- Multiple format files per book record
CREATE TABLE IF NOT EXISTS book_formats (
    id          TEXT    NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    book_id     TEXT    NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    format      TEXT    NOT NULL,
    file_path   TEXT    NOT NULL UNIQUE,
    file_sha256 TEXT    NOT NULL UNIQUE,
    file_size   INTEGER NOT NULL DEFAULT 0,
    added_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS book_formats_book_id ON book_formats(book_id);
```

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/db/migrations/0005_book_formats.sql
git commit -m "P5-T04: migration 0005 — book_formats (multi-format per book)"
```

---

## P5-T05

Write `processing/src/db/format_queries.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct BookFormat {
    pub id:          String,
    pub book_id:     String,
    pub format:      String,
    pub file_path:   String,
    pub file_sha256: String,
    pub file_size:   i64,
    pub added_at:    String,
}

pub async fn insert_book_format(
    pool: &SqlitePool,
    book_id: &str,
    format: &str,
    file_path: &str,
    file_sha256: &str,
    file_size: i64,
) -> Result<String, ProcessingError> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT OR IGNORE INTO book_formats
         (id, book_id, format, file_path, file_sha256, file_size)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(book_id)
    .bind(format)
    .bind(file_path)
    .bind(file_sha256)
    .bind(file_size)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(id)
}

pub async fn get_formats_for_book(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Vec<BookFormat>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, i64, String)>(
        "SELECT id, book_id, format, file_path, file_sha256, file_size, added_at
         FROM book_formats WHERE book_id = ? ORDER BY added_at ASC",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(rows
        .into_iter()
        .map(|(id, book_id, format, file_path, file_sha256, file_size, added_at)| BookFormat {
            id, book_id, format, file_path, file_sha256, file_size, added_at,
        })
        .collect())
}
```

In `processing/src/db/mod.rs`, add `pub mod extended_queries;` and `pub mod format_queries;` after the existing line.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/db/format_queries.rs processing/src/db/mod.rs processing/src/db/extended_queries.rs
git commit -m "P5-T05: book_formats DB queries + extended_queries module"
```

---

## P5-T06

Write `processing/src/db/migrations/0006_fts.sql` with this exact content:
```sql
-- Full-text search over book title, authors, and extracted body text
CREATE VIRTUAL TABLE IF NOT EXISTS books_fts USING fts5(
    book_id   UNINDEXED,
    title,
    authors,
    full_text,
    tokenize  = "unicode61 remove_diacritics 2"
);
```

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/db/migrations/0006_fts.sql
git commit -m "P5-T06: migration 0006 — FTS5 virtual table"
```

---

## P5-T07

Write `processing/src/db/fts_queries.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use sqlx::SqlitePool;

/// Insert or replace a book's FTS entry.
/// Deletes any existing rows for this book_id first.
pub async fn upsert_fts(
    pool: &SqlitePool,
    book_id: &str,
    title: &str,
    authors: &str,
    full_text: &str,
) -> Result<(), ProcessingError> {
    // FTS5 does not allow WHERE on UNINDEXED columns in DELETE,
    // so we use a subquery to find the rowid.
    sqlx::query(
        "DELETE FROM books_fts WHERE rowid IN
         (SELECT rowid FROM books_fts WHERE book_id = ?)",
    )
    .bind(book_id)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    sqlx::query(
        "INSERT INTO books_fts (book_id, title, authors, full_text) VALUES (?, ?, ?, ?)",
    )
    .bind(book_id)
    .bind(title)
    .bind(authors)
    .bind(full_text)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(())
}

/// Search the FTS index. Returns matching book_ids.
pub async fn search(pool: &SqlitePool, query: &str) -> Result<Vec<String>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT book_id FROM books_fts WHERE books_fts MATCH ? ORDER BY rank",
    )
    .bind(query)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(rows.into_iter().map(|(id,)| id).collect())
}
```

In `processing/src/db/mod.rs`, add `pub mod fts_queries;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/db/fts_queries.rs processing/src/db/mod.rs
git commit -m "P5-T07: FTS5 upsert and search query functions"
```

---

## P5-T08

In `src-tauri/src/commands.rs`, add these two commands at the end of the file:
```rust
#[tauri::command]
pub async fn search_books(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    query: String,
) -> Result<Vec<serde_json::Value>, String> {
    let book_ids = xcalibre_processing::db::fts_queries::search(
        pool.inner().as_ref(),
        &query,
    )
    .await
    .map_err(|e| e.to_string())?;

    if book_ids.is_empty() {
        return Ok(vec![]);
    }

    // Fetch full book records for matched ids
    let placeholders = book_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT id, title, authors_json, format, cover_path, progress_percent, last_opened_at
         FROM local_books WHERE id IN ({}) ORDER BY title_sort, title",
        placeholders
    );
    let mut q = sqlx::query_as::<_, (String, String, String, String, Option<String>, f64, Option<String>)>(&sql);
    for id in &book_ids {
        q = q.bind(id);
    }
    let rows = q
        .fetch_all(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|(id, title, authors_json, format, cover_path, progress_percent, last_opened_at)| {
            let authors: Vec<String> = serde_json::from_str(&authors_json).unwrap_or_default();
            serde_json::json!({
                "id": id, "title": title, "authors": authors,
                "format": format, "cover_path": cover_path,
                "progress_percent": progress_percent,
                "last_opened_at": last_opened_at,
            })
        })
        .collect())
}

#[tauri::command]
pub async fn filter_books(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    format: Option<String>,
    status: Option<String>,
    author: Option<String>,
    tag: Option<String>,
    series: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let mut conditions = vec!["1=1".to_string()];
    let mut binds: Vec<String> = vec![];

    if let Some(ref f) = format {
        conditions.push("lb.format = ?".to_string());
        binds.push(f.clone());
    }
    if let Some(ref s) = status {
        conditions.push("lb.status = ?".to_string());
        binds.push(s.clone());
    }
    if let Some(ref a) = author {
        conditions.push("lb.authors_json LIKE ?".to_string());
        binds.push(format!("%{}%", a));
    }
    if let Some(ref s) = series {
        conditions.push("lb.series_name = ?".to_string());
        binds.push(s.clone());
    }

    let tag_join = if tag.is_some() {
        conditions.push("t.name = ?".to_string());
        binds.push(tag.as_ref().unwrap().clone());
        "JOIN book_tags bt ON bt.book_id = lb.id JOIN tags t ON t.id = bt.tag_id"
    } else {
        ""
    };

    let sql = format!(
        "SELECT DISTINCT lb.id, lb.title, lb.authors_json, lb.format, lb.cover_path,
                lb.progress_percent, lb.last_opened_at
         FROM local_books lb {}
         WHERE {} ORDER BY lb.title_sort, lb.title",
        tag_join,
        conditions.join(" AND ")
    );

    let mut q = sqlx::query_as::<_, (String, String, String, String, Option<String>, f64, Option<String>)>(&sql);
    for b in &binds {
        q = q.bind(b.as_str());
    }
    let rows = q
        .fetch_all(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|(id, title, authors_json, format, cover_path, progress_percent, last_opened_at)| {
            let authors: Vec<String> = serde_json::from_str(&authors_json).unwrap_or_default();
            serde_json::json!({
                "id": id, "title": title, "authors": authors,
                "format": format, "cover_path": cover_path,
                "progress_percent": progress_percent,
                "last_opened_at": last_opened_at,
            })
        })
        .collect())
}
```

Also add `commands::search_books` and `commands::filter_books` to the `invoke_handler!` list in `src-tauri/src/main.rs`.

Then run:
```bash
cargo build --workspace
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "P5-T08: search_books and filter_books Tauri commands"
```

---

## P5-T09

Write `ui/src/components/SearchBar.tsx` with this exact content:
```tsx
import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Book, useLibraryStore } from "../store/libraryStore"

export function SearchBar() {
  const [query, setQuery] = useState("")
  const setBooks = useLibraryStore((s) => s.setBooks)

  const onSearch = async (q: string) => {
    setQuery(q)
    if (q.trim().length < 2) {
      useLibraryStore.getState().fetchBooks()
      return
    }
    try {
      const results = await invoke<Book[]>("search_books", { query: q })
      setBooks(results)
    } catch {
      /* ignore — keep current results */
    }
  }

  return (
    <div className="relative flex-1 max-w-md">
      <input
        type="search"
        value={query}
        onChange={(e) => onSearch(e.target.value)}
        placeholder="Search title, author, or text…"
        className="w-full rounded-lg border border-gray-300 dark:border-gray-600
                   bg-white dark:bg-gray-700 px-3 py-1.5 text-sm
                   dark:text-white placeholder-gray-400 focus:outline-none
                   focus:ring-2 focus:ring-blue-500"
      />
    </div>
  )
}
```

In `ui/src/store/libraryStore.ts`, add a `setBooks` action to the store:
```ts
  setBooks: (books: Book[]) => void
```
And in the store implementation:
```ts
  setBooks: (books) => set({ books }),
```

Then run:
```bash
cd ui && npm run build && cd ..
git add ui/src/components/SearchBar.tsx ui/src/store/libraryStore.ts
git commit -m "P5-T09: SearchBar component wired to FTS5 search"
```

---

## P5-T10

In `src-tauri/src/commands.rs`, add these commands at the end:
```rust
#[tauri::command]
pub async fn list_tags(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query_as::<_, (String,)>("SELECT name FROM tags ORDER BY name")
        .fetch_all(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|(n,)| n).collect())
}

#[tauri::command]
pub async fn list_series(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT series_name FROM local_books
         WHERE series_name IS NOT NULL ORDER BY series_name",
    )
    .fetch_all(pool.inner().as_ref())
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|(n,)| n).collect())
}

#[tauri::command]
pub async fn list_authors(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT authors_json FROM local_books ORDER BY author_sort",
    )
    .fetch_all(pool.inner().as_ref())
    .await
    .map_err(|e| e.to_string())?;
    let mut authors: Vec<String> = rows
        .into_iter()
        .flat_map(|(json,)| {
            serde_json::from_str::<Vec<String>>(&json).unwrap_or_default()
        })
        .collect();
    authors.sort();
    authors.dedup();
    Ok(authors)
}
```

Add `commands::list_tags`, `commands::list_series`, `commands::list_authors` to the `invoke_handler!` in `src-tauri/src/main.rs`.

Then run:
```bash
cargo build --workspace
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "P5-T10: filter backend commands — list_tags, list_series, list_authors"
```

---

## P5-T11

Write `ui/src/components/FilterBar.tsx` with this exact content:
```tsx
import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Book, useLibraryStore } from "../store/libraryStore"

export function FilterBar() {
  const [formats]  = useState(["EPUB", "PDF", "MOBI", "CBZ", "TXT"])
  const [authors,  setAuthors]  = useState<string[]>([])
  const [series,   setSeries]   = useState<string[]>([])
  const [tags,     setTags]     = useState<string[]>([])
  const setBooks = useLibraryStore((s) => s.setBooks)

  useEffect(() => {
    invoke<string[]>("list_authors").then(setAuthors).catch(() => {})
    invoke<string[]>("list_series").then(setSeries).catch(() => {})
    invoke<string[]>("list_tags").then(setTags).catch(() => {})
  }, [])

  const apply = async (field: string, value: string) => {
    const args: Record<string, string | null> = {
      format: null, status: null, author: null, tag: null, series: null,
    }
    if (value) args[field] = value
    try {
      const results = await invoke<Book[]>("filter_books", args)
      setBooks(results)
    } catch { /* ignore */ }
  }

  const Select = ({ label, field, opts }: { label: string; field: string; opts: string[] }) => (
    <select
      onChange={(e) => apply(field, e.target.value)}
      className="text-sm rounded border border-gray-300 dark:border-gray-600
                 bg-white dark:bg-gray-700 dark:text-white px-2 py-1"
    >
      <option value="">{label}</option>
      {opts.map((o) => <option key={o} value={o}>{o}</option>)}
    </select>
  )

  return (
    <div className="flex flex-wrap gap-2 px-6 py-2 border-b border-gray-200 dark:border-gray-700">
      <Select label="Format"  field="format" opts={formats} />
      <Select label="Author"  field="author" opts={authors} />
      <Select label="Series"  field="series" opts={series} />
      <Select label="Tag"     field="tag"    opts={tags} />
    </div>
  )
}
```

Then run:
```bash
cd ui && npm run build && cd ..
git add ui/src/components/FilterBar.tsx
git commit -m "P5-T11: FilterBar UI component"
```

---

## P5-T12

Write `processing/src/db/migrations/0007_collections.sql` with this exact content:
```sql
CREATE TABLE IF NOT EXISTS collections (
    id         TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name       TEXT NOT NULL UNIQUE COLLATE NOCASE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS book_collections (
    book_id       TEXT NOT NULL REFERENCES local_books(id)   ON DELETE CASCADE,
    collection_id TEXT NOT NULL REFERENCES collections(id)   ON DELETE CASCADE,
    added_at      TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (book_id, collection_id)
);
```

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/db/migrations/0007_collections.sql
git commit -m "P5-T12: migration 0007 — collections table"
```

---

## P5-T13

Write `processing/src/db/collection_queries.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Collection {
    pub id:         String,
    pub name:       String,
    pub created_at: String,
}

pub async fn create_collection(
    pool: &SqlitePool,
    name: &str,
) -> Result<String, ProcessingError> {
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO collections (id, name) VALUES (?, ?)")
        .bind(&id)
        .bind(name)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(id)
}

pub async fn list_collections(pool: &SqlitePool) -> Result<Vec<Collection>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, String, String)>(
        "SELECT id, name, created_at FROM collections ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(rows
        .into_iter()
        .map(|(id, name, created_at)| Collection { id, name, created_at })
        .collect())
}

pub async fn delete_collection(pool: &SqlitePool, id: &str) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM collections WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn add_book_to_collection(
    pool: &SqlitePool,
    book_id: &str,
    collection_id: &str,
) -> Result<(), ProcessingError> {
    sqlx::query(
        "INSERT OR IGNORE INTO book_collections (book_id, collection_id) VALUES (?, ?)",
    )
    .bind(book_id)
    .bind(collection_id)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn remove_book_from_collection(
    pool: &SqlitePool,
    book_id: &str,
    collection_id: &str,
) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM book_collections WHERE book_id = ? AND collection_id = ?")
        .bind(book_id)
        .bind(collection_id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn get_books_in_collection(
    pool: &SqlitePool,
    collection_id: &str,
) -> Result<Vec<String>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT book_id FROM book_collections WHERE collection_id = ?",
    )
    .bind(collection_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}
```

In `processing/src/db/mod.rs`, add `pub mod collection_queries;` after the existing lines.

In `src-tauri/src/commands.rs`, add these commands at the end:
```rust
#[tauri::command]
pub async fn create_collection(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    name: String,
) -> Result<String, String> {
    xcalibre_processing::db::collection_queries::create_collection(pool.inner().as_ref(), &name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_collections(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<Vec<serde_json::Value>, String> {
    let cols = xcalibre_processing::db::collection_queries::list_collections(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;
    Ok(cols.into_iter().map(|c| serde_json::json!({ "id": c.id, "name": c.name })).collect())
}

#[tauri::command]
pub async fn delete_collection(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    id: String,
) -> Result<(), String> {
    xcalibre_processing::db::collection_queries::delete_collection(pool.inner().as_ref(), &id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_book_to_collection(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
    collection_id: String,
) -> Result<(), String> {
    xcalibre_processing::db::collection_queries::add_book_to_collection(
        pool.inner().as_ref(), &book_id, &collection_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_books_in_collection(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    collection_id: String,
) -> Result<Vec<String>, String> {
    xcalibre_processing::db::collection_queries::get_books_in_collection(
        pool.inner().as_ref(), &collection_id,
    )
    .await
    .map_err(|e| e.to_string())
}
```

Add all five new collection commands to the `invoke_handler!` in `src-tauri/src/main.rs`.

Then run:
```bash
cargo build --workspace
git add processing/src/db/collection_queries.rs processing/src/db/mod.rs src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "P5-T13: collections DB CRUD + Tauri commands"
```

---

## P5-T14

Write `ui/src/components/CollectionsSidebar.tsx` with this exact content:
```tsx
import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { useLibraryStore } from "../store/libraryStore"

interface Collection { id: string; name: string }

export function CollectionsSidebar() {
  const [collections, setCollections] = useState<Collection[]>([])
  const [newName, setNewName] = useState("")
  const [active, setActive] = useState<string | null>(null)
  const { fetchBooks, setBooks } = useLibraryStore()

  const load = () =>
    invoke<Collection[]>("list_collections").then(setCollections).catch(() => {})

  useEffect(() => { load() }, [])

  const create = async () => {
    if (!newName.trim()) return
    await invoke("create_collection", { name: newName.trim() }).catch(() => {})
    setNewName("")
    load()
  }

  const select = async (col: Collection) => {
    setActive(col.id)
    const ids = await invoke<string[]>("get_books_in_collection", { collectionId: col.id }).catch(() => [] as string[])
    // Re-use filter_books with the ids if non-empty, otherwise show all
    if (ids.length === 0) { setBooks([]); return }
    // Fetch the actual book records by querying filter without constraints then filtering client-side
    const allBooks = await invoke<any[]>("list_books").catch(() => [] as any[])
    setBooks(allBooks.filter((b: any) => ids.includes(b.id)))
  }

  const clearFilter = () => { setActive(null); fetchBooks() }

  return (
    <aside className="w-48 shrink-0 border-r border-gray-200 dark:border-gray-700 p-3 flex flex-col gap-2">
      <p className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">Collections</p>
      <button
        onClick={clearFilter}
        className={`text-left text-sm px-2 py-1 rounded ${active === null ? "bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300" : "hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-300"}`}
      >
        All Books
      </button>
      {collections.map((c) => (
        <button
          key={c.id}
          onClick={() => select(c)}
          className={`text-left text-sm px-2 py-1 rounded truncate ${active === c.id ? "bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300" : "hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-300"}`}
        >
          {c.name}
        </button>
      ))}
      <div className="mt-auto flex gap-1">
        <input
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && create()}
          placeholder="New collection…"
          className="flex-1 min-w-0 text-xs border rounded px-1 py-0.5 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
        />
        <button onClick={create} className="text-xs px-1.5 py-0.5 bg-blue-500 text-white rounded hover:bg-blue-600">+</button>
      </div>
    </aside>
  )
}
```

Then run:
```bash
cd ui && npm run build && cd ..
git add ui/src/components/CollectionsSidebar.tsx
git commit -m "P5-T14: CollectionsSidebar UI"
```

---

## P5-T15

In `src-tauri/src/commands.rs`, add these bulk operation commands at the end:
```rust
#[tauri::command]
pub async fn bulk_delete_books(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_ids: Vec<String>,
) -> Result<u64, String> {
    let mut count = 0u64;
    for id in &book_ids {
        let result = sqlx::query("DELETE FROM local_books WHERE id = ?")
            .bind(id)
            .execute(pool.inner().as_ref())
            .await
            .map_err(|e| e.to_string())?;
        count += result.rows_affected();
    }
    Ok(count)
}

#[tauri::command]
pub async fn bulk_reingest_books(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_ids: Vec<String>,
) -> Result<Vec<String>, String> {
    let mut queued = vec![];
    for id in &book_ids {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT file_path FROM local_books WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;

        if let Some((path,)) = row {
            let result = xcalibre_processing::pipeline::ingest::run_ingest(
                pool.inner().as_ref(),
                std::path::Path::new(&path),
            )
            .await
            .map_err(|e| e.to_string())?;
            queued.push(result.job_id);
        }
    }
    Ok(queued)
}

#[tauri::command]
pub async fn bulk_export_metadata(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_ids: Vec<String>,
) -> Result<String, String> {
    let placeholders = book_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT id, title, authors_json, format, publisher, series_name, series_index, isbn, pubdate
         FROM local_books WHERE id IN ({})",
        placeholders
    );
    let mut q = sqlx::query_as::<_, (String, String, String, String, Option<String>, Option<String>, Option<f64>, Option<String>, Option<String>)>(&sql);
    for id in &book_ids { q = q.bind(id); }
    let rows = q.fetch_all(pool.inner().as_ref()).await.map_err(|e| e.to_string())?;

    let mut csv = "id,title,authors,format,publisher,series,series_index,isbn,pubdate\n".to_string();
    for (id, title, authors_json, format, publisher, series_name, series_index, isbn, pubdate) in rows {
        let authors: Vec<String> = serde_json::from_str(&authors_json).unwrap_or_default();
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            id,
            title.replace(',', ";"),
            authors.join(";").replace(',', ";"),
            format,
            publisher.unwrap_or_default(),
            series_name.unwrap_or_default(),
            series_index.map(|f| f.to_string()).unwrap_or_default(),
            isbn.unwrap_or_default(),
            pubdate.unwrap_or_default(),
        ));
    }
    Ok(csv)
}
```

Add `commands::bulk_delete_books`, `commands::bulk_reingest_books`, `commands::bulk_export_metadata` to the `invoke_handler!` in `src-tauri/src/main.rs`.

Then run:
```bash
cargo build --workspace
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "P5-T15: bulk operations — delete, re-ingest, export metadata"
```

---

## P5-T16

Write `ui/src/components/BulkActionBar.tsx` with this exact content:
```tsx
import { invoke } from "@tauri-apps/api/core"
import { useLibraryStore } from "../store/libraryStore"

interface Props {
  selected: Set<string>
  onClear: () => void
}

export function BulkActionBar({ selected, onClear }: Props) {
  if (selected.size === 0) return null

  const { fetchBooks } = useLibraryStore()

  const ids = Array.from(selected)

  const handleDelete = async () => {
    if (!confirm(`Delete ${ids.length} book(s) from library?`)) return
    await invoke("bulk_delete_books", { bookIds: ids }).catch(console.error)
    onClear()
    fetchBooks()
  }

  const handleReingest = async () => {
    await invoke("bulk_reingest_books", { bookIds: ids }).catch(console.error)
    onClear()
    fetchBooks()
  }

  const handleExport = async () => {
    const csv = await invoke<string>("bulk_export_metadata", { bookIds: ids }).catch(() => "")
    const blob = new Blob([csv], { type: "text/csv" })
    const url = URL.createObjectURL(blob)
    const a = document.createElement("a")
    a.href = url; a.download = "xcalibre_export.csv"; a.click()
    URL.revokeObjectURL(url)
  }

  return (
    <div className="fixed bottom-4 left-1/2 -translate-x-1/2 flex items-center gap-3
                    bg-white dark:bg-gray-800 shadow-xl rounded-xl px-5 py-3 border border-gray-200 dark:border-gray-700 z-40">
      <span className="text-sm font-medium dark:text-white">{ids.length} selected</span>
      <button onClick={handleReingest} className="text-sm px-3 py-1.5 rounded bg-blue-500 text-white hover:bg-blue-600">Re-ingest</button>
      <button onClick={handleExport}   className="text-sm px-3 py-1.5 rounded bg-green-500 text-white hover:bg-green-600">Export CSV</button>
      <button onClick={handleDelete}   className="text-sm px-3 py-1.5 rounded bg-red-500 text-white hover:bg-red-600">Delete</button>
      <button onClick={onClear}        className="text-sm px-3 py-1.5 rounded bg-gray-200 dark:bg-gray-700 dark:text-white hover:bg-gray-300">Cancel</button>
    </div>
  )
}
```

Then run:
```bash
cd ui && npm run build && cd ..
git add ui/src/components/BulkActionBar.tsx
git commit -m "P5-T16: BulkActionBar UI"
```

---

## P5-T17

Write `processing/src/import/mod.rs` with this exact content:
```rust
pub mod calibre;
pub mod opf;
```

Write `processing/src/import/opf.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

/// Parse a standalone OPF sidecar file (Calibre's metadata.opf).
/// Reuses the same OPF field mapping as the EPUB extractor.
pub fn parse_opf(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut xml  = String::new();
    file.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;

    let doc = roxmltree::Document::parse(&xml)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut meta = BookMetadata::default();
    for node in doc.descendants() {
        match node.tag_name().name() {
            "title"       => meta.title       = node.text().map(str::trim).map(String::from),
            "creator"     => meta.authors.push(node.text().unwrap_or("").trim().to_string()),
            "language"    => meta.language    = node.text().map(str::trim).map(String::from),
            "publisher"   => meta.publisher   = node.text().map(str::trim).map(String::from),
            "date"        => meta.published   = node.text().map(str::trim).map(String::from),
            "description" => meta.description = node.text().map(str::trim).map(String::from),
            "subject"     => {
                if let Some(t) = node.text() {
                    let t = t.trim().to_string();
                    if !t.is_empty() { meta.tags.push(t); }
                }
            }
            "identifier"  => {
                let scheme = node.attribute("opf:scheme")
                    .or_else(|| node.attribute("scheme"))
                    .unwrap_or("");
                if scheme.eq_ignore_ascii_case("isbn") {
                    meta.isbn = node.text().map(str::trim).map(String::from);
                }
            }
            "meta" => {
                let name = node.attribute("name").unwrap_or("");
                let content = node.attribute("content").unwrap_or("");
                match name {
                    "calibre:series"       => meta.series       = Some(content.to_string()),
                    "calibre:series_index" => meta.series_index = content.parse().ok(),
                    _ => {}
                }
            }
            _ => {}
        }
    }
    meta.authors.retain(|a| !a.is_empty());
    Ok(meta)
}
```

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/import/mod.rs processing/src/import/opf.rs
git commit -m "P5-T17: OPF sidecar parser for Calibre metadata.opf"
```

---

## P5-T18

Write `processing/src/import/calibre.rs` with this exact content:
```rust
use crate::db::queries;
use crate::error::ProcessingError;
use crate::import::opf;
use crate::pipeline::ingest;
use crate::utils::hash::sha256_file;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct ImportStats {
    pub found:     usize,
    pub imported:  usize,
    pub skipped:   usize,
    pub errors:    usize,
}

static EBOOK_EXTS: &[&str] = &[
    "epub", "pdf", "mobi", "azw", "azw3", "cbz", "cbr", "txt",
    "fb2", "html", "htm", "htmlz", "rtf", "docx", "odt",
    "chm", "lit", "lrf", "pdb", "pml", "rb", "snb", "tcr", "djvu",
];

/// Walk a Calibre library directory and import all ebook files.
/// Skips files whose SHA-256 hash is already in the jobs table.
pub async fn import_calibre_library(
    pool: &SqlitePool,
    library_path: &Path,
) -> Result<ImportStats, ProcessingError> {
    let mut stats = ImportStats::default();

    // Collect (ebook_path, opf_path?) pairs
    let mut book_dirs: Vec<(Vec<PathBuf>, Option<PathBuf>)> = vec![];

    for entry in WalkDir::new(library_path).min_depth(2).max_depth(2) {
        let entry = match entry {
            Ok(e)  => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_dir() {
            continue;
        }
        let dir = entry.path();
        let mut books: Vec<PathBuf> = vec![];
        let mut opf:   Option<PathBuf> = None;

        let entries = match std::fs::read_dir(dir) {
            Ok(e)  => e,
            Err(_) => continue,
        };

        for item in entries.flatten() {
            let p = item.path();
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                let lower = ext.to_lowercase();
                if lower == "opf" {
                    opf = Some(p);
                } else if EBOOK_EXTS.contains(&lower.as_str()) {
                    books.push(p);
                }
            }
        }
        if !books.is_empty() {
            book_dirs.push((books, opf));
        }
    }

    for (books, opf_path) in book_dirs {
        // Parse OPF metadata once for all formats in this directory
        let _opf_meta = opf_path.as_deref().and_then(|p| opf::parse_opf(p).ok());

        for book_path in &books {
            stats.found += 1;

            // Dedup by SHA-256
            let sha = match sha256_file(book_path) {
                Ok(h)  => h,
                Err(e) => { warn!(path = %book_path.display(), err = %e, "hash failed"); stats.errors += 1; continue; }
            };

            let existing = sqlx::query_as::<_, (String,)>(
                "SELECT id FROM jobs WHERE file_sha256 = ? LIMIT 1",
            )
            .bind(&sha)
            .fetch_optional(pool)
            .await
            .map_err(ProcessingError::DbError)?;

            if existing.is_some() {
                info!(path = %book_path.display(), "skipping duplicate");
                stats.skipped += 1;
                continue;
            }

            match ingest::run_ingest(pool, book_path).await {
                Ok(result) => {
                    info!(job_id = %result.job_id, path = %book_path.display(), "imported");
                    stats.imported += 1;
                }
                Err(e) => {
                    warn!(path = %book_path.display(), err = %e, "ingest failed");
                    stats.errors += 1;
                }
            }
        }
    }

    Ok(stats)
}
```

In `src-tauri/src/commands.rs`, add this command at the end:
```rust
#[tauri::command]
pub async fn import_calibre(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    path: String,
) -> Result<serde_json::Value, String> {
    let stats = xcalibre_processing::import::calibre::import_calibre_library(
        pool.inner().as_ref(),
        std::path::Path::new(&path),
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "found": stats.found, "imported": stats.imported,
        "skipped": stats.skipped, "errors": stats.errors,
    }))
}
```

Add `commands::import_calibre` to the `invoke_handler!` in `src-tauri/src/main.rs`.

Then run:
```bash
cargo build --workspace
git add processing/src/import/calibre.rs src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "P5-T18: Calibre library import — walk tree, SHA-256 dedup, OPF sidecar"
```

---

## P5-T19

Write `processing/src/integrity.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use sqlx::SqlitePool;

#[derive(Debug, Clone, serde::Serialize)]
pub struct IntegrityIssue {
    pub book_id:   String,
    pub file_path: String,
    pub issue:     String,
}

/// Scan all local_books records and verify files exist on disk.
pub async fn check_integrity(pool: &SqlitePool) -> Result<Vec<IntegrityIssue>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT id, file_path FROM local_books WHERE file_path IS NOT NULL",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    let mut issues = vec![];
    for (id, path) in rows {
        let p = std::path::Path::new(&path);
        if !p.exists() {
            issues.push(IntegrityIssue {
                book_id:   id,
                file_path: path,
                issue:     "file_missing".to_string(),
            });
        }
    }
    Ok(issues)
}
```

In `src-tauri/src/commands.rs`, add this command at the end:
```rust
#[tauri::command]
pub async fn check_library_integrity(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<Vec<serde_json::Value>, String> {
    let issues = xcalibre_processing::integrity::check_integrity(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;
    Ok(issues.into_iter().map(|i| serde_json::json!({
        "book_id": i.book_id, "file_path": i.file_path, "issue": i.issue,
    })).collect())
}
```

Add `commands::check_library_integrity` to the `invoke_handler!`.

Then run:
```bash
cargo build --workspace
git add processing/src/integrity.rs src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "P5-T19: library integrity check — verify files exist on disk"
```

---

## P5-T20

Write `processing/src/catalog.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use sqlx::SqlitePool;

/// Export library as a CSV string.
pub async fn export_csv(pool: &SqlitePool) -> Result<String, ProcessingError> {
    let rows = sqlx::query_as::<_, (
        String, String, String, String, Option<String>,
        Option<String>, Option<f64>, Option<String>, Option<String>,
    )>(
        "SELECT id, title, authors_json, format, publisher,
                series_name, series_index, isbn, pubdate
         FROM local_books ORDER BY title_sort, title",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    let mut out = String::from("id,title,authors,format,publisher,series,series_index,isbn,pubdate\n");
    for (id, title, authors_json, format, publisher, series_name, series_index, isbn, pubdate) in rows {
        let authors: Vec<String> = serde_json::from_str(&authors_json).unwrap_or_default();
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&id), csv_escape(&title),
            csv_escape(&authors.join("; ")),
            csv_escape(&format),
            csv_escape(publisher.as_deref().unwrap_or("")),
            csv_escape(series_name.as_deref().unwrap_or("")),
            series_index.map(|f| f.to_string()).unwrap_or_default(),
            csv_escape(isbn.as_deref().unwrap_or("")),
            csv_escape(pubdate.as_deref().unwrap_or("")),
        ));
    }
    Ok(out)
}

/// Export library as an HTML page.
pub async fn export_html(pool: &SqlitePool) -> Result<String, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>)>(
        "SELECT id, title, authors_json, format, series_name
         FROM local_books ORDER BY title_sort, title",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    let mut rows_html = String::new();
    for (id, title, authors_json, format, series_name) in &rows {
        let authors: Vec<String> = serde_json::from_str(authors_json).unwrap_or_default();
        rows_html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            html_escape(&title),
            html_escape(&authors.join(", ")),
            html_escape(&format),
            html_escape(series_name.as_deref().unwrap_or("")),
        ));
        let _ = id;
    }

    Ok(format!(
        r#"<!doctype html><html lang="en"><head><meta charset="UTF-8">
<title>xCalibre Library</title>
<style>body{{font-family:sans-serif;padding:2rem}}table{{border-collapse:collapse;width:100%}}
th,td{{border:1px solid #ccc;padding:.4rem .8rem;text-align:left}}th{{background:#f0f0f0}}</style>
</head><body>
<h1>xCalibre Library ({} books)</h1>
<table><thead><tr><th>Title</th><th>Authors</th><th>Format</th><th>Series</th></tr></thead>
<tbody>{}</tbody></table></body></html>"#,
        rows.len(), rows_html
    ))
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
```

In `src-tauri/src/commands.rs`, add these commands at the end:
```rust
#[tauri::command]
pub async fn export_library_csv(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<String, String> {
    xcalibre_processing::catalog::export_csv(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_library_html(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<String, String> {
    xcalibre_processing::catalog::export_html(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())
}
```

Add both catalog commands to the `invoke_handler!`.

Then run:
```bash
cargo build --workspace
git add processing/src/catalog.rs src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "P5-T20: catalog export — HTML and CSV"
```

---

## P5-T21

Update `processing/src/lib.rs` to declare all new modules. Add these lines:
```rust
pub mod catalog;
pub mod import;
pub mod integrity;
```

Also add `pub mod db { pub mod extended_queries; pub mod format_queries; pub mod fts_queries; pub mod collection_queries; }` to `processing/src/db/mod.rs` if not already there — the individual `pub mod` declarations in `db/mod.rs` cover this.

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/lib.rs
git commit -m "P5-T21: wire new modules into lib.rs"
```

---

### ✅ Milestone check — after P5-T21
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm run build && cd ..
```

---

## P5-T22

Write `processing/tests/test_library.rs` with this exact content:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::{collection_queries, extended_queries, fts_queries};
use xcalibre_processing::pipeline::ingest::run_ingest;
use xcalibre_processing::utils::sort::{author_sort, title_sort};
use std::path::PathBuf;

async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_title_sort_fn() {
    assert_eq!(title_sort("The Great Gatsby"), "Great Gatsby, The");
    assert_eq!(title_sort("Dune"), "Dune");
    assert_eq!(title_sort("A Tale of Two Cities"), "Tale of Two Cities, A");
}

#[tokio::test]
async fn test_author_sort_fn() {
    assert_eq!(author_sort("Frank Herbert"), "Herbert, Frank");
    assert_eq!(author_sort("Herbert, Frank"), "Herbert, Frank");
}

#[tokio::test]
async fn test_tags_crud() {
    let pool = setup_db().await;
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest = run_ingest(&pool, &path).await.unwrap();

    extended_queries::upsert_tag(&pool, &ingest.job_id, "fiction").await.unwrap();
    extended_queries::upsert_tag(&pool, &ingest.job_id, "classic").await.unwrap();

    let tags = extended_queries::get_tags(&pool, &ingest.job_id).await.unwrap();
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"fiction".to_string()));
}

#[tokio::test]
async fn test_identifiers_crud() {
    let pool = setup_db().await;
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest = run_ingest(&pool, &path).await.unwrap();

    extended_queries::upsert_identifier(&pool, &ingest.job_id, "isbn", "9780000000000").await.unwrap();
    let ids = extended_queries::get_identifiers(&pool, &ingest.job_id).await.unwrap();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], ("isbn".to_string(), "9780000000000".to_string()));
}

#[tokio::test]
async fn test_collections_crud() {
    let pool = setup_db().await;
    let col_id = collection_queries::create_collection(&pool, "Sci-Fi").await.unwrap();
    let cols = collection_queries::list_collections(&pool).await.unwrap();
    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "Sci-Fi");

    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest = run_ingest(&pool, &path).await.unwrap();
    collection_queries::add_book_to_collection(&pool, &ingest.job_id, &col_id).await.unwrap();

    let books = collection_queries::get_books_in_collection(&pool, &col_id).await.unwrap();
    assert_eq!(books.len(), 1);
    assert_eq!(books[0], ingest.job_id);
}

#[tokio::test]
async fn test_fts_search() {
    let pool = setup_db().await;
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest = run_ingest(&pool, &path).await.unwrap();

    fts_queries::upsert_fts(&pool, &ingest.job_id, "Fixture Book", "Test Author", "This is the fixture chapter body text").await.unwrap();
    let results = fts_queries::search(&pool, "fixture").await.unwrap();
    assert!(results.contains(&ingest.job_id));
}
```

Then run:
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
git add processing/tests/test_library.rs
git commit -m "P5-T22: library management tests — tags, identifiers, collections, FTS"
```

---

---

## P5-T23 — Frontend tests: CollectionsSidebar + FilterBar

Write tests BEFORE the implementation is considered done — TDD.

Fixture definitions (put at the top of each test file):
```ts
const COLLECTION_A = { id: "c1", name: "Sci-Fi", created_at: "2024-01-01" }
const COLLECTION_B = { id: "c2", name: "Classics", created_at: "2024-01-02" }
const BOOK_A = {
  id: "b1", title: "Dune", authors: ["Herbert"], format: "epub",
  cover_path: null, progress_percent: 0, last_opened_at: null, reading_cfi: null,
}
```

Write `ui/src/components/CollectionsSidebar.test.tsx`:
  Implement every test case from `localProject/TEST_SPEC.md` section
  `"ui/src/components/CollectionsSidebar.test.tsx"`.

  Rules:
  - Use the real `useLibraryStore`. Reset in `beforeEach`:
    `useLibraryStore.setState({ books: [], loading: false, error: null })`.
  - Use `userEvent.setup()` for all interactions.
  - Components call `invoke` on mount via `useEffect` — use `await screen.findBy*`
    (async queries) after render, not `screen.getBy*`.
  - Use `mockInvoke()` to seed the invoke response before rendering.

Write `ui/src/components/FilterBar.test.tsx`:
  Implement every test case from `localProject/TEST_SPEC.md` section
  `"ui/src/components/FilterBar.test.tsx"`.

  Rules:
  - Same store reset and `userEvent.setup()` pattern as above.

Then run:
```bash
cd ui && npm test -- --reporter=verbose 2>&1
```
All tests must pass.

```bash
git add ui/src/components/CollectionsSidebar.test.tsx \
        ui/src/components/FilterBar.test.tsx
git commit -m "test(phase5): CollectionsSidebar and FilterBar tests"
```

---

## P5-T24 — Frontend tests: SearchBar + BulkActionBar

Write `ui/src/components/SearchBar.test.tsx`:
  Implement every test case from `localProject/TEST_SPEC.md` section
  `"ui/src/components/SearchBar.test.tsx"`.

  Rules:
  - SearchBar debounces 200ms. Use `vi.useFakeTimers()` and
    `await vi.advanceTimersByTimeAsync(200)` to trigger debounce without real waiting.
  - Always call `vi.useRealTimers()` in `afterEach`.

Write `ui/src/components/BulkActionBar.test.tsx`:
  Implement every test case from `localProject/TEST_SPEC.md` section
  `"ui/src/components/BulkActionBar.test.tsx"`.

  Rules:
  - Props: `selectedIds`, `selectedBook`, `onDone`, `onEditMetadata`, `onRepair`,
    `onConvert` — pass `vi.fn()` for all callbacks.
  - For delete tests: `vi.spyOn(window, "confirm").mockReturnValue(true)`.
    Restore in `afterEach`: `vi.restoreAllMocks()`.

Then run:
```bash
cd ui && npm test -- --reporter=verbose 2>&1
```
All tests must pass.

```bash
git add ui/src/components/SearchBar.test.tsx \
        ui/src/components/BulkActionBar.test.tsx
git commit -m "test(phase5): SearchBar and BulkActionBar tests"
```

---

### ✅ Milestone check — Phase 5 complete
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm run build && npm test && cd ..
```
