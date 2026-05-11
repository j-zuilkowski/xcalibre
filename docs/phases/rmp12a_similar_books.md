# RMP-12a — Similar Books (Red: Failing Tests)

> Prerequisite: rmp01b complete (library support), rmp02b complete (search executor).
> TDD role: RED — define the similarity query API via failing tests.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R12a-T01 | Failing tests: similarity query backend | ⬜ |
| R12a-T02 | Failing tests: SimilarBooksPanel component | ⬜ |

---

## R12a-T01

Write `processing/tests/test_similar_books.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::similar_queries::{
    find_similar_books, SimilarBook, SimilarityFactors,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();

    // Insert test books
    sqlx::query(
        "INSERT INTO local_books (id, title, authors, format, tags_json, series, language)
         VALUES
           ('b1', 'Dune',              '[\"Frank Herbert\"]', 'EPUB', '[\"sci-fi\",\"classic\"]',     'Dune Chronicles', 'en'),
           ('b2', 'Dune Messiah',      '[\"Frank Herbert\"]', 'EPUB', '[\"sci-fi\",\"classic\"]',     'Dune Chronicles', 'en'),
           ('b3', 'Foundation',        '[\"Isaac Asimov\"]',  'EPUB', '[\"sci-fi\",\"classic\"]',     'Foundation',      'en'),
           ('b4', 'Pride and Prejudice','[\"Jane Austen\"]',  'EPUB', '[\"romance\",\"classic\"]',    null,              'en'),
           ('b5', 'Neuromancer',       '[\"William Gibson\"]','EPUB', '[\"sci-fi\",\"cyberpunk\"]',   null,              'en')"
    ).execute(&pool).await.unwrap();

    pool
}

#[tokio::test]
async fn test_same_author_scores_high() {
    let pool = setup().await;
    let results = find_similar_books(
        &pool, "b1",
        &SimilarityFactors { same_author: true, same_series: true, shared_tags: true, same_language: true },
        10,
    ).await.expect("find_similar");
    // Dune Messiah by same author + same series should appear first
    assert!(!results.is_empty());
    assert_eq!(results[0].id, "b2", "Dune Messiah should be most similar to Dune");
}

#[tokio::test]
async fn test_shared_tags_included() {
    let pool = setup().await;
    let results = find_similar_books(
        &pool, "b1",
        &SimilarityFactors { same_author: false, same_series: false, shared_tags: true, same_language: true },
        10,
    ).await.expect("find_similar");
    let ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
    // Foundation shares sci-fi+classic tags; Neuromancer shares sci-fi
    assert!(ids.contains(&"b3") || ids.contains(&"b5"), "sci-fi books should appear: {:?}", ids);
    // Pride and Prejudice shares only 'classic' — may appear but with lower score
}

#[tokio::test]
async fn test_excludes_source_book() {
    let pool = setup().await;
    let results = find_similar_books(
        &pool, "b1",
        &SimilarityFactors { same_author: true, same_series: true, shared_tags: true, same_language: true },
        10,
    ).await.expect("find_similar");
    assert!(!results.iter().any(|r| r.id == "b1"), "source book must not appear in results");
}

#[tokio::test]
async fn test_respects_limit() {
    let pool = setup().await;
    let results = find_similar_books(
        &pool, "b1",
        &SimilarityFactors { same_author: true, same_series: true, shared_tags: true, same_language: true },
        2,
    ).await.expect("find_similar");
    assert!(results.len() <= 2, "must respect limit of 2");
}

#[tokio::test]
async fn test_no_results_for_isolated_book() {
    let pool = setup().await;
    sqlx::query(
        "INSERT INTO local_books (id, title, authors, format, tags_json, language)
         VALUES ('b99', 'Lonely Book', '[\"Unknown Author\"]', 'EPUB', '[]', 'zh')"
    ).execute(&pool).await.unwrap();

    let results = find_similar_books(
        &pool, "b99",
        &SimilarityFactors { same_author: true, same_series: true, shared_tags: true, same_language: true },
        10,
    ).await.expect("find_similar");
    assert!(results.is_empty(), "isolated book should have no similar books");
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

Expected: `db::similar_queries` not found. RED confirmed.

```bash
git add processing/tests/test_similar_books.rs
git commit -m "R12a-T01: failing tests for similar books query"
```

---

## R12a-T02

Write `ui/src/components/SimilarBooksPanel.test.tsx`:
```tsx
import { render, screen, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { SimilarBooksPanel } from "./SimilarBooksPanel"
import { mockInvoke } from "../test/setup"

const mockBook = {
  id: "b1", title: "Dune", authors: ["Frank Herbert"],
  format: "EPUB", cover_path: null, progress_percent: 0, last_opened_at: null,
}

const mockSimilar = [
  { id: "b2", title: "Dune Messiah",  authors: "Frank Herbert", score: 3 },
  { id: "b3", title: "Foundation",    authors: "Isaac Asimov",  score: 1 },
]

beforeEach(() => {
  mockInvoke("find_similar_books", mockSimilar)
})

describe("SimilarBooksPanel", () => {
  it("renders section header", async () => {
    render(<SimilarBooksPanel book={mockBook} onOpenBook={vi.fn()} />)
    expect(screen.getByTestId("similar-books-header")).toBeInTheDocument()
  })

  it("lists similar books", async () => {
    render(<SimilarBooksPanel book={mockBook} onOpenBook={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByText("Dune Messiah")).toBeInTheDocument()
      expect(screen.getByText("Foundation")).toBeInTheDocument()
    })
  })

  it("shows empty state when no similar books", async () => {
    mockInvoke("find_similar_books", [])
    render(<SimilarBooksPanel book={mockBook} onOpenBook={vi.fn()} />)
    await waitFor(() =>
      expect(screen.getByTestId("similar-books-empty")).toBeInTheDocument()
    )
  })

  it("calls onOpenBook when a result is clicked", async () => {
    const onOpenBook = vi.fn()
    render(<SimilarBooksPanel book={mockBook} onOpenBook={onOpenBook} />)
    await waitFor(() => screen.getByText("Dune Messiah"))
    screen.getByText("Dune Messiah").click()
    expect(onOpenBook).toHaveBeenCalledWith("b2")
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/SimilarBooksPanel.test.tsx
git commit -m "R12a-T02: failing tests for SimilarBooksPanel component"
```

---

### ✅ RED Checkpoint

```bash
cargo test --workspace 2>&1 | grep -c "^error"
cd ui && npm test 2>&1 | grep -c "FAIL" && cd ..
```

Both should show failures. Proceed to **rmp12b**.
