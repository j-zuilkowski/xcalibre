# RMP-02a — Field-scoped Search Parser (Red: Failing Tests)

> HOW TO USE: Write every file exactly as shown, run every shell block.
> DO NOT print code as output — write to disk using your tools.
> Prerequisite: rmp01b complete and all tests green.
> TDD role: RED — write failing tests that define the parser contract.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R02a-T01 | Failing tests: query AST parsing | ⬜ |
| R02a-T02 | Failing tests: query execution against DB | ⬜ |
| R02a-T03 | Failing tests: SearchBar autocomplete hints | ⬜ |

---

## R02a-T01

Write `processing/tests/test_search_parser.rs` with this exact content:
```rust
//! Tests for the field-scoped search parser (RMP-02).
//! These tests FAIL until rmp02b implements the parser.

use xcalibre_processing::search::{parse_query, QueryNode};

#[test]
fn test_bare_term() {
    let q = parse_query("rust").unwrap();
    assert!(matches!(q, QueryNode::FtsTerm(ref s) if s == "rust"));
}

#[test]
fn test_field_scoped_title() {
    let q = parse_query("title:rust").unwrap();
    assert!(matches!(q, QueryNode::Field { field, ref value, .. }
        if field == "title" && value == "rust"));
}

#[test]
fn test_field_scoped_author() {
    let q = parse_query("author:Knuth").unwrap();
    assert!(matches!(q, QueryNode::Field { field, ref value, .. }
        if field == "author" && value == "Knuth"));
}

#[test]
fn test_and_expression() {
    let q = parse_query("title:rust AND author:klabnik").unwrap();
    assert!(matches!(q, QueryNode::And(_, _)));
}

#[test]
fn test_or_expression() {
    let q = parse_query("tag:fantasy OR tag:scifi").unwrap();
    assert!(matches!(q, QueryNode::Or(_, _)));
}

#[test]
fn test_not_expression() {
    let q = parse_query("title:rust NOT tag:beginner").unwrap();
    assert!(matches!(q, QueryNode::And(_, _)));
    // NOT is represented as AND NOT
    if let QueryNode::And(left, right) = q {
        assert!(matches!(*left, QueryNode::Field { .. }));
        assert!(matches!(*right, QueryNode::Not(_)));
    }
}

#[test]
fn test_quoted_value() {
    let q = parse_query(r#"title:"The Rust Programming Language""#).unwrap();
    if let QueryNode::Field { value, .. } = q {
        assert_eq!(value, "The Rust Programming Language");
    } else {
        panic!("expected Field node");
    }
}

#[test]
fn test_supported_fields() {
    for field in &["title", "author", "tag", "series", "format", "publisher", "language"] {
        let q = parse_query(&format!("{}:x", field));
        assert!(q.is_ok(), "field '{}' should be accepted", field);
    }
}

#[test]
fn test_unknown_field_is_fts_fallback() {
    // Unknown fields should be treated as FTS terms, not errors
    let q = parse_query("foo:bar").unwrap();
    // Either FtsTerm or a Field with passthrough — just must not Err
    let _ = q;
}

#[test]
fn test_empty_query_is_err() {
    assert!(parse_query("").is_err());
}
```

Then run:
```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -10
```

Expected: compile error — `xcalibre_processing::search` module does not exist. RED confirmed.

```bash
git add processing/tests/test_search_parser.rs
git commit -m "R02a-T01: failing tests for query AST parsing"
```

---

## R02a-T02

Write `processing/tests/test_search_execution.rs` with this exact content:
```rust
//! Tests for executing parsed queries against a real SQLite library DB.
//! These tests FAIL until rmp02b implements execute_query().

use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::search::execute_query;

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    // Insert two fixture books
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, file_path,
          file_sha256, status, progress_percent, cover_path, last_opened_at,
          series, series_index, publisher, language, tags_json)
         VALUES
         ('b1','The Rust Programming Language','[\"Steve Klabnik\"]','EPUB','/f1',
          'sha1','READY',0,NULL,NULL,'Rust Series',1,'No Starch','en','[\"programming\",\"systems\"]'),
         ('b2','Good Omens','[\"Terry Pratchett\",\"Neil Gaiman\"]','EPUB','/f2',
          'sha2','READY',0,NULL,NULL,NULL,NULL,'Gollancz','en','[\"fantasy\",\"comedy\"]')",
    )
    .execute(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_bare_term_matches_title() {
    let pool = setup().await;
    let ids = execute_query(&pool, "Rust").await.expect("execute");
    assert!(ids.contains(&"b1".to_string()), "bare 'Rust' should match b1");
}

#[tokio::test]
async fn test_field_title_match() {
    let pool = setup().await;
    let ids = execute_query(&pool, "title:\"Good Omens\"").await.unwrap();
    assert_eq!(ids, vec!["b2"]);
}

#[tokio::test]
async fn test_field_author_match() {
    let pool = setup().await;
    let ids = execute_query(&pool, "author:Gaiman").await.unwrap();
    assert_eq!(ids, vec!["b2"]);
}

#[tokio::test]
async fn test_field_tag_match() {
    let pool = setup().await;
    let ids = execute_query(&pool, "tag:fantasy").await.unwrap();
    assert_eq!(ids, vec!["b2"]);
}

#[tokio::test]
async fn test_and_narrows_results() {
    let pool = setup().await;
    // Only b1 matches both
    let ids = execute_query(&pool, "author:Klabnik AND tag:programming").await.unwrap();
    assert_eq!(ids, vec!["b1"]);
}

#[tokio::test]
async fn test_not_excludes_results() {
    let pool = setup().await;
    let ids = execute_query(&pool, "format:EPUB NOT tag:fantasy").await.unwrap();
    assert!(ids.contains(&"b1".to_string()));
    assert!(!ids.contains(&"b2".to_string()));
}

#[tokio::test]
async fn test_no_match_returns_empty() {
    let pool = setup().await;
    let ids = execute_query(&pool, "title:Nonexistent").await.unwrap();
    assert!(ids.is_empty());
}
```

Then run:
```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -10
```

Expected: compile errors. RED confirmed.

```bash
git add processing/tests/test_search_execution.rs
git commit -m "R02a-T02: failing tests for search query execution"
```

---

## R02a-T03

Write `ui/src/components/SearchBar.test.tsx` with this exact content:
```tsx
import { render, screen, fireEvent } from "@testing-library/react"
import { describe, it, expect, vi } from "vitest"
import { SearchBar } from "./SearchBar"

describe("SearchBar", () => {
  it("renders an input", () => {
    render(<SearchBar onSearch={vi.fn()} />)
    expect(screen.getByRole("textbox")).toBeInTheDocument()
  })

  it("calls onSearch when Enter is pressed", () => {
    const onSearch = vi.fn()
    render(<SearchBar onSearch={onSearch} />)
    const input = screen.getByRole("textbox")
    fireEvent.change(input, { target: { value: "title:rust" } })
    fireEvent.keyDown(input, { key: "Enter" })
    expect(onSearch).toHaveBeenCalledWith("title:rust")
  })

  it("shows field hint autocomplete when ':' is typed", async () => {
    render(<SearchBar onSearch={vi.fn()} />)
    const input = screen.getByRole("textbox")
    fireEvent.change(input, { target: { value: "title:" } })
    // Hint list should appear
    expect(screen.getByTestId("search-hints")).toBeInTheDocument()
  })

  it("shows no hints for bare text", () => {
    render(<SearchBar onSearch={vi.fn()} />)
    const input = screen.getByRole("textbox")
    fireEvent.change(input, { target: { value: "rust" } })
    expect(screen.queryByTestId("search-hints")).not.toBeInTheDocument()
  })

  it("clears input when clear button is clicked", () => {
    const onSearch = vi.fn()
    render(<SearchBar onSearch={onSearch} />)
    const input = screen.getByRole("textbox")
    fireEvent.change(input, { target: { value: "title:rust" } })
    fireEvent.click(screen.getByTestId("search-clear"))
    expect(input).toHaveValue("")
    expect(onSearch).toHaveBeenCalledWith("")
  })
})
```

Then run:
```bash
cd ui && npm test -- --reporter=verbose 2>&1 | tail -20 && cd ..
```

Expected: test failure — `SearchBar` component does not exist yet. RED confirmed.

```bash
git add ui/src/components/SearchBar.test.tsx
git commit -m "R02a-T03: failing tests for SearchBar component"
```

---

### ✅ RED Checkpoint

```bash
cargo test --workspace 2>&1 | grep -c "^error"
cd ui && npm test 2>&1 | grep -c "FAIL" && cd ..
```

Both should report failures. Proceed to **rmp02b**.
