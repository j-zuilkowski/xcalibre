# RMP-12b — Similar Books (Green: Implementation)

> Prerequisite: rmp12a complete.
> TDD role: GREEN — implement the similarity engine and panel UI.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R12b-T01 | `db/similar_queries.rs` — weighted scoring | ⬜ |
| R12b-T02 | Tauri command `find_similar_books` | ⬜ |
| R12b-T03 | `SimilarBooksPanel.tsx` component | ⬜ |
| R12b-T04 | Wire into BookDetailPanel | ⬜ |
| R12b-T05 | Milestone check + visual inspection | ⬜ |

---

## R12b-T01

Write `processing/src/db/similar_queries.rs`:
```rust
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityFactors {
    pub same_author:   bool,
    pub same_series:   bool,
    pub shared_tags:   bool,
    pub same_language: bool,
}

impl Default for SimilarityFactors {
    fn default() -> Self {
        Self { same_author: true, same_series: true, shared_tags: true, same_language: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SimilarBook {
    pub id:      String,
    pub title:   String,
    pub authors: String,
    pub score:   i64,
}

/// Score weights:
///   same_series:   4
///   same_author:   3
///   per_shared_tag:1 (up to 3 total)
///   same_language: 1
pub async fn find_similar_books(
    pool: &SqlitePool,
    book_id: &str,
    factors: &SimilarityFactors,
    limit: i64,
) -> Result<Vec<SimilarBook>, sqlx::Error> {
    // Fetch source book metadata
    let src = sqlx::query!(
        "SELECT authors, series, tags_json, language FROM local_books WHERE id=?",
        book_id
    )
    .fetch_optional(pool)
    .await?;

    let src = match src {
        Some(r) => r,
        None    => return Ok(vec![]),
    };

    let src_authors: Vec<String> = src.authors
        .as_deref()
        .and_then(|a| serde_json::from_str(a).ok())
        .unwrap_or_default();
    let src_tags: Vec<String> = src.tags_json
        .as_deref()
        .and_then(|t| serde_json::from_str(t).ok())
        .unwrap_or_default();
    let src_series   = src.series.clone();
    let src_language = src.language.clone();

    // Fetch all other books
    let candidates = sqlx::query!(
        "SELECT id, title, authors, series, tags_json, language
         FROM local_books WHERE id != ?",
        book_id
    )
    .fetch_all(pool)
    .await?;

    let mut scored: Vec<SimilarBook> = candidates
        .into_iter()
        .filter_map(|row| {
            let cand_authors: Vec<String> = row.authors
                .as_deref()
                .and_then(|a| serde_json::from_str(a).ok())
                .unwrap_or_default();
            let cand_tags: Vec<String> = row.tags_json
                .as_deref()
                .and_then(|t| serde_json::from_str(t).ok())
                .unwrap_or_default();

            let mut score: i64 = 0;

            if factors.same_series {
                if let (Some(ss), Some(cs)) = (&src_series, &row.series) {
                    if ss == cs { score += 4; }
                }
            }
            if factors.same_author {
                let any_match = src_authors.iter().any(|a| cand_authors.contains(a));
                if any_match { score += 3; }
            }
            if factors.shared_tags {
                let shared = src_tags.iter().filter(|t| cand_tags.contains(t)).count() as i64;
                score += shared.min(3);
            }
            if factors.same_language {
                if src_language == row.language { score += 1; }
            }

            if score == 0 { return None; }

            Some(SimilarBook {
                id:      row.id,
                title:   row.title,
                authors: row.authors.unwrap_or_default(),
                score,
            })
        })
        .collect();

    scored.sort_by(|a, b| b.score.cmp(&a.score).then(a.title.cmp(&b.title)));
    scored.truncate(limit as usize);
    Ok(scored)
}
```

Add to `processing/src/db/mod.rs`:
```rust
pub mod similar_queries;
```

Then run:
```bash
cargo test --workspace -- test_similar_books
git add processing/src/db/similar_queries.rs processing/src/db/mod.rs
git commit -m "R12b-T01: similar books weighted scoring — all tests green"
```

---

## R12b-T02

Write `src-tauri/src/commands/similar.rs`:
```rust
use tauri::State;
use xcalibre_processing::db::similar_queries::{
    find_similar_books, SimilarBook, SimilarityFactors,
};
use crate::AppState;

#[tauri::command]
pub async fn find_similar_books_cmd(
    book_id:     String,
    same_author: Option<bool>,
    same_series: Option<bool>,
    shared_tags: Option<bool>,
    same_language: Option<bool>,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<SimilarBook>, String> {
    let factors = SimilarityFactors {
        same_author:   same_author.unwrap_or(true),
        same_series:   same_series.unwrap_or(true),
        shared_tags:   shared_tags.unwrap_or(true),
        same_language: same_language.unwrap_or(true),
    };
    find_similar_books(&state.pool, &book_id, &factors, limit.unwrap_or(10))
        .await.map_err(|e| e.to_string())
}
```

Register in `src-tauri/src/main.rs`:
```rust
commands::similar::find_similar_books_cmd,
```

```bash
cargo build --workspace
git add src-tauri/src/commands/similar.rs src-tauri/src/main.rs
git commit -m "R12b-T02: find_similar_books Tauri command"
```

---

## R12b-T03

Write `ui/src/components/SimilarBooksPanel.tsx`:
```tsx
import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Book {
  id: string
  title: string
  authors: string[]
  format: string
  cover_path: string | null
  progress_percent: number
  last_opened_at: string | null
}

interface SimilarBook {
  id: string
  title: string
  authors: string
  score: number
}

interface Props {
  book: Book
  onOpenBook: (id: string) => void
}

export function SimilarBooksPanel({ book, onOpenBook }: Props) {
  const [similar, setSimilar] = useState<SimilarBook[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    invoke<SimilarBook[]>("find_similar_books_cmd", { bookId: book.id, limit: 8 })
      .then(setSimilar)
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [book.id])

  return (
    <section>
      <h3
        data-testid="similar-books-header"
        style={{ margin: "0 0 0.75rem", fontSize: "1rem", fontWeight: 600 }}
      >
        Similar Books
      </h3>

      {loading && (
        <p style={{ color: "var(--text-muted, #6c7086)", fontSize: "0.9rem" }}>Loading…</p>
      )}

      {!loading && similar.length === 0 && (
        <p
          data-testid="similar-books-empty"
          style={{ color: "var(--text-muted, #6c7086)", fontSize: "0.9rem" }}
        >
          No similar books found in your library.
        </p>
      )}

      {!loading && similar.length > 0 && (
        <ul style={{ listStyle: "none", padding: 0, margin: 0 }}>
          {similar.map(s => (
            <li
              key={s.id}
              onClick={() => onOpenBook(s.id)}
              style={{
                padding: "0.5rem 0.75rem", borderRadius: "6px", cursor: "pointer",
                marginBottom: "0.25rem",
              }}
              onMouseEnter={e => (e.currentTarget.style.background = "var(--bg-overlay, #313244)")}
              onMouseLeave={e => (e.currentTarget.style.background = "transparent")}
            >
              <div style={{ fontWeight: 500 }}>{s.title}</div>
              <div style={{ fontSize: "0.8rem", color: "var(--text-muted, #6c7086)" }}>
                {s.authors}
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  )
}
```

Then run:
```bash
cd ui && npm test -- SimilarBooksPanel && cd ..
git add ui/src/components/SimilarBooksPanel.tsx
git commit -m "R12b-T03: SimilarBooksPanel component — all UI tests green"
```

---

## R12b-T04

In `ui/src/components/BookDetailPanel.tsx`, add the SimilarBooksPanel below the book metadata section:

```tsx
import { SimilarBooksPanel } from "./SimilarBooksPanel"

// In JSX, below existing detail fields:
<SimilarBooksPanel book={book} onOpenBook={onOpenBook} />
```

```bash
cd ui && npm test && cd ..
git add ui/src/components/BookDetailPanel.tsx
git commit -m "R12b-T04: SimilarBooksPanel wired into BookDetailPanel"
```

---

## R12b-T05 — Milestone Check + Visual Inspection

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
2. Open a book with known author/series/tags (e.g. first in a series)
3. Scroll down in the detail panel — verify "Similar Books" section appears
4. Verify same-series books appear first (highest score)
5. Verify same-author books appear next
6. Click a similar book — verify it opens that book's detail view
7. For a book with no metadata, verify "No similar books found" message

```bash
pkill -x xcalibre 2>/dev/null || true
```

```bash
git add -A
git commit -m "R12b-T05: RMP-12 Similar Books — all tests green, panel wired"
```
