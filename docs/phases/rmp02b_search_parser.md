# RMP-02b — Field-scoped Search Parser (Green: Implementation)

> HOW TO USE: Write every file exactly as shown, run every shell block.
> DO NOT print code as output — write to disk using your tools.
> Prerequisite: rmp02a complete (failing tests committed).
> TDD role: GREEN — implement until all tests pass.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R02b-T01 | `search/mod.rs` — QueryNode + parse_query() | ⬜ |
| R02b-T02 | `search/execute.rs` — execute_query() | ⬜ |
| R02b-T03 | Tauri command: search_library_advanced | ⬜ |
| R02b-T04 | SearchBar.tsx component | ⬜ |
| R02b-T05 | Wire SearchBar into LibraryView | ⬜ |
| R02b-T06 | Milestone check + visual inspection | ⬜ |

---

## R02b-T01

In `processing/src/lib.rs`, add `pub mod search;`.

Write `processing/src/search/mod.rs` with this exact content:
```rust
pub mod execute;
pub use execute::execute_query;

use crate::error::ProcessingError;

/// AST node produced by parse_query().
#[derive(Debug, Clone, PartialEq)]
pub enum QueryNode {
    /// Bare term — falls through to FTS5
    FtsTerm(String),
    /// Field-scoped match: field:value
    Field { field: String, value: String, negate: bool },
    /// Logical AND of two nodes
    And(Box<QueryNode>, Box<QueryNode>),
    /// Logical OR of two nodes
    Or(Box<QueryNode>, Box<QueryNode>),
    /// Logical NOT of a node
    Not(Box<QueryNode>),
}

/// Supported field names.
const KNOWN_FIELDS: &[&str] = &[
    "title", "author", "tag", "series", "format",
    "publisher", "language", "rating",
];

/// Parse a query string into a QueryNode AST.
/// Grammar (simplified LL(1)):
///   query  := or_expr
///   or_expr  := and_expr ( "OR" and_expr )*
///   and_expr := not_expr ( "AND" not_expr )*
///   not_expr := "NOT" atom | atom
///   atom     := field_term | fts_term | "(" query ")"
///   field_term := IDENT ":" value
///   value     := quoted_string | bare_word
pub fn parse_query(input: &str) -> Result<QueryNode, ProcessingError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ProcessingError::MetadataError("empty query".into()));
    }
    let tokens = tokenize(input);
    let mut pos = 0usize;
    let node = parse_or_expr(&tokens, &mut pos)?;
    Ok(node)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    Quoted(String),
    Colon,
    LParen,
    RParen,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => { chars.next(); }
            ':' => { chars.next(); tokens.push(Token::Colon); }
            '(' => { chars.next(); tokens.push(Token::LParen); }
            ')' => { chars.next(); tokens.push(Token::RParen); }
            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' { chars.next(); break; }
                    s.push(ch); chars.next();
                }
                tokens.push(Token::Quoted(s));
            }
            _ => {
                let mut word = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == ' ' || ch == ':' || ch == '(' || ch == ')' { break; }
                    word.push(ch); chars.next();
                }
                tokens.push(Token::Word(word));
            }
        }
    }
    tokens
}

fn parse_or_expr(tokens: &[Token], pos: &mut usize) -> Result<QueryNode, ProcessingError> {
    let mut left = parse_and_expr(tokens, pos)?;
    while *pos < tokens.len() {
        if let Token::Word(w) = &tokens[*pos] {
            if w.eq_ignore_ascii_case("OR") {
                *pos += 1;
                let right = parse_and_expr(tokens, pos)?;
                left = QueryNode::Or(Box::new(left), Box::new(right));
                continue;
            }
        }
        break;
    }
    Ok(left)
}

fn parse_and_expr(tokens: &[Token], pos: &mut usize) -> Result<QueryNode, ProcessingError> {
    let mut left = parse_not_expr(tokens, pos)?;
    while *pos < tokens.len() {
        if let Token::Word(w) = &tokens[*pos] {
            if w.eq_ignore_ascii_case("AND") {
                *pos += 1;
                let right = parse_not_expr(tokens, pos)?;
                left = QueryNode::And(Box::new(left), Box::new(right));
                continue;
            }
            // implicit AND for consecutive terms
            if !w.eq_ignore_ascii_case("OR") && !w.eq_ignore_ascii_case("NOT") {
                let right = parse_not_expr(tokens, pos)?;
                left = QueryNode::And(Box::new(left), Box::new(right));
                continue;
            }
        }
        break;
    }
    Ok(left)
}

fn parse_not_expr(tokens: &[Token], pos: &mut usize) -> Result<QueryNode, ProcessingError> {
    if let Some(Token::Word(w)) = tokens.get(*pos) {
        if w.eq_ignore_ascii_case("NOT") {
            *pos += 1;
            let inner = parse_atom(tokens, pos)?;
            return Ok(QueryNode::Not(Box::new(inner)));
        }
    }
    parse_atom(tokens, pos)
}

fn parse_atom(tokens: &[Token], pos: &mut usize) -> Result<QueryNode, ProcessingError> {
    if *pos >= tokens.len() {
        return Err(ProcessingError::MetadataError("unexpected end of query".into()));
    }
    match &tokens[*pos] {
        Token::LParen => {
            *pos += 1;
            let inner = parse_or_expr(tokens, pos)?;
            if let Some(Token::RParen) = tokens.get(*pos) { *pos += 1; }
            Ok(inner)
        }
        Token::Word(w) => {
            let word = w.clone();
            *pos += 1;
            // Check for field:value
            if let Some(Token::Colon) = tokens.get(*pos) {
                *pos += 1;
                let value = match tokens.get(*pos) {
                    Some(Token::Word(v))   => { *pos += 1; v.clone() }
                    Some(Token::Quoted(v)) => { *pos += 1; v.clone() }
                    _ => String::new(),
                };
                let field_lower = word.to_lowercase();
                // Unknown fields fall through as FTS terms
                if KNOWN_FIELDS.contains(&field_lower.as_str()) {
                    return Ok(QueryNode::Field { field: field_lower, value, negate: false });
                } else {
                    return Ok(QueryNode::FtsTerm(format!("{}:{}", word, value)));
                }
            }
            Ok(QueryNode::FtsTerm(word))
        }
        Token::Quoted(q) => {
            let q = q.clone(); *pos += 1;
            Ok(QueryNode::FtsTerm(q))
        }
        _ => Err(ProcessingError::MetadataError("unexpected token in query".into())),
    }
}
```

Then run:
```bash
cargo test --workspace -- test_search_parser
git add processing/src/search/mod.rs processing/src/lib.rs
git commit -m "R02b-T01: query AST parser — parser tests green"
```

---

## R02b-T02

Write `processing/src/search/execute.rs` with this exact content:
```rust
use super::QueryNode;
use crate::error::ProcessingError;
use sqlx::SqlitePool;

/// Execute a parsed query and return matching book IDs.
pub async fn execute_query(pool: &SqlitePool, query: &str) -> Result<Vec<String>, ProcessingError> {
    let node = super::parse_query(query)?;
    let ids = collect_ids(pool, &node).await?;
    Ok(ids)
}

async fn collect_ids(pool: &SqlitePool, node: &QueryNode) -> Result<Vec<String>, ProcessingError> {
    match node {
        QueryNode::FtsTerm(term) => {
            fts_search(pool, term).await
        }
        QueryNode::Field { field, value, negate } => {
            let ids = field_search(pool, field, value).await?;
            if *negate {
                let all = all_ids(pool).await?;
                Ok(all.into_iter().filter(|id| !ids.contains(id)).collect())
            } else {
                Ok(ids)
            }
        }
        QueryNode::And(left, right) => {
            let l = collect_ids(pool, left).await?;
            let r = collect_ids(pool, right).await?;
            Ok(l.into_iter().filter(|id| r.contains(id)).collect())
        }
        QueryNode::Or(left, right) => {
            let mut l = collect_ids(pool, left).await?;
            let r = collect_ids(pool, right).await?;
            for id in r { if !l.contains(&id) { l.push(id); } }
            Ok(l)
        }
        QueryNode::Not(inner) => {
            let excluded = collect_ids(pool, inner).await?;
            let all = all_ids(pool).await?;
            Ok(all.into_iter().filter(|id| !excluded.contains(id)).collect())
        }
    }
}

async fn fts_search(pool: &SqlitePool, term: &str) -> Result<Vec<String>, ProcessingError> {
    // Use FTS5 books_fts table (created in migration 0005_fts.sql)
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT b.id FROM local_books b
         JOIN books_fts f ON f.rowid = b.rowid
         WHERE books_fts MATCH ?
         ORDER BY rank",
    )
    .bind(term)
    .fetch_all(pool).await.map_err(ProcessingError::DbError)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

async fn field_search(pool: &SqlitePool, field: &str, value: &str) -> Result<Vec<String>, ProcessingError> {
    let like = format!("%{}%", value.to_lowercase());
    let sql = match field {
        "title"     => "SELECT id FROM local_books WHERE lower(title) LIKE ?",
        "author"    => "SELECT id FROM local_books WHERE lower(authors_json) LIKE ?",
        "tag"       => "SELECT id FROM local_books WHERE lower(tags_json) LIKE ?",
        "series"    => "SELECT id FROM local_books WHERE lower(COALESCE(series,'')) LIKE ?",
        "format"    => "SELECT id FROM local_books WHERE lower(format) = lower(?)",
        "publisher" => "SELECT id FROM local_books WHERE lower(COALESCE(publisher,'')) LIKE ?",
        "language"  => "SELECT id FROM local_books WHERE lower(COALESCE(language,'en')) LIKE ?",
        _           => return Ok(vec![]),
    };
    // format:VALUE uses exact match not LIKE
    let param = if field == "format" { value.to_string() } else { like };
    let rows: Vec<(String,)> = sqlx::query_as(sql)
        .bind(&param)
        .fetch_all(pool).await.map_err(ProcessingError::DbError)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

async fn all_ids(pool: &SqlitePool) -> Result<Vec<String>, ProcessingError> {
    let rows: Vec<(String,)> = sqlx::query_as("SELECT id FROM local_books")
        .fetch_all(pool).await.map_err(ProcessingError::DbError)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}
```

Then run:
```bash
cargo test --workspace -- test_search_execution
git add processing/src/search/execute.rs
git commit -m "R02b-T02: search query executor — execution tests green"
```

---

## R02b-T03

In `src-tauri/src/commands.rs`, append:
```rust
#[tauri::command]
pub async fn search_library_advanced(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    query: String,
) -> Result<Vec<String>, String> {
    xcalibre_processing::search::execute_query(pool.inner().as_ref(), &query)
        .await
        .map_err(|e| e.to_string())
}
```

Register in `main.rs` `generate_handler!`:
```rust
commands::search_library_advanced,
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "R02b-T03: search_library_advanced Tauri command"
```

---

## R02b-T04

Write `ui/src/components/SearchBar.tsx` with this exact content:
```tsx
import { useState, useRef } from "react"

const FIELD_HINTS = ["title:", "author:", "tag:", "series:", "format:", "publisher:", "language:"]

interface Props {
  onSearch: (query: string) => void
  placeholder?: string
}

export function SearchBar({ onSearch, placeholder = "Search… (title:rust AND author:Klabnik)" }: Props) {
  const [value, setValue] = useState("")
  const [showHints, setShowHints] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  const handleChange = (v: string) => {
    setValue(v)
    // Show hints when the last typed segment ends with ':'
    const lastToken = v.split(/\s+/).pop() ?? ""
    setShowHints(lastToken.endsWith(":") && lastToken.length > 1)
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      setShowHints(false)
      onSearch(value.trim())
    }
    if (e.key === "Escape") {
      setShowHints(false)
      setValue("")
      onSearch("")
    }
  }

  const handleHintClick = (hint: string) => {
    const tokens = value.split(/\s+/)
    tokens[tokens.length - 1] = hint
    const next = tokens.join(" ")
    setValue(next)
    setShowHints(false)
    inputRef.current?.focus()
  }

  return (
    <div className="relative w-full">
      <div className="flex items-center border rounded-lg px-3 py-1.5 bg-white dark:bg-gray-800
                      border-gray-300 dark:border-gray-600 focus-within:ring-2 focus-within:ring-blue-500">
        <svg className="w-4 h-4 text-gray-400 mr-2 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
        </svg>
        <input
          ref={inputRef}
          type="text"
          value={value}
          onChange={e => handleChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          className="flex-1 bg-transparent outline-none text-sm dark:text-white placeholder-gray-400"
        />
        {value && (
          <button
            data-testid="search-clear"
            onClick={() => { setValue(""); setShowHints(false); onSearch("") }}
            className="ml-2 text-gray-400 hover:text-gray-600 text-lg leading-none"
          >×</button>
        )}
      </div>
      {showHints && (
        <ul data-testid="search-hints"
            className="absolute z-50 w-full mt-1 bg-white dark:bg-gray-800 border
                       border-gray-200 dark:border-gray-600 rounded-lg shadow-lg text-sm">
          {FIELD_HINTS.map(hint => (
            <li key={hint}>
              <button
                onMouseDown={e => { e.preventDefault(); handleHintClick(hint) }}
                className="w-full text-left px-4 py-2 hover:bg-gray-50 dark:hover:bg-gray-700
                           text-blue-600 dark:text-blue-400 font-mono"
              >
                {hint}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
```

Then run:
```bash
cd ui && npm test -- --reporter=verbose 2>&1 | tail -20 && cd ..
git add ui/src/components/SearchBar.tsx
git commit -m "R02b-T04: SearchBar component — SearchBar tests green"
```

---

## R02b-T05

In `ui/src/components/LibraryView.tsx`, import and render `SearchBar` above the grid:
```tsx
import { SearchBar } from "./SearchBar"
import { invoke } from "@tauri-apps/api/core"
// ...inside LibraryView component, before the grid:
const [searchIds, setSearchIds] = useState<string[] | null>(null)

const handleSearch = async (query: string) => {
  if (!query) { setSearchIds(null); return }
  try {
    const ids = await invoke<string[]>("search_library_advanced", { query })
    setSearchIds(ids)
  } catch { setSearchIds([]) }
}

// Filter books when searchIds is set:
const displayBooks = searchIds
  ? books.filter(b => searchIds.includes(b.id))
  : books
```

Add `<SearchBar onSearch={handleSearch} />` above the grid div.

Then run:
```bash
cd ui && npm run build && npm test && cd ..
cargo build --workspace
git add ui/src/components/LibraryView.tsx
git commit -m "R02b-T05: wire SearchBar into LibraryView"
```

---

## R02b-T06 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm run build && npm test && cd ..
```

All tests green. Zero warnings.

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

Verify:
- [ ] Search bar visible at top of LibraryView
- [ ] Typing `title:` shows field hint dropdown
- [ ] Pressing a hint fills the search input with e.g. `title:`
- [ ] Entering `title:rust AND author:klabnik` and pressing Enter filters the book grid
- [ ] Clearing the search (× button) restores full grid
- [ ] Empty library shows "No books yet" not an error

```bash
pkill -x xcalibre 2>/dev/null || true
```

```bash
git add -A
git commit -m "R02b-T06: RMP-02 field-scoped search — all tests green, visual verified"
```
