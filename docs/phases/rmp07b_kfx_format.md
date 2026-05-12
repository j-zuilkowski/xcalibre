# RMP-07b — KFX Format Support (Green: Implementation)

> Prerequisite: rmp07a complete.
> TDD role: GREEN — implement KFX container parsing until all tests pass.
> Decision: S5-A — full text + metadata extraction.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R07b-T01 | Add rusqlite dependency + KFX format detection | ⬜ |
| R07b-T02 | `metadata/kfx.rs` — metadata from fragment_properties | ⬜ |
| R07b-T03 | `text/kfx.rs` — text from content fragments | ⬜ |
| R07b-T04 | Wire KFX into pipeline stages | ⬜ |
| R07b-T05 | Milestone check + visual inspection | ⬜ |

---

## R07b-T01

Add to `processing/Cargo.toml` under `[dependencies]`:
```toml
rusqlite = { version = "0.31", features = ["bundled"] }
```

In `processing/src/plugins/mod.rs`, add KFX detection to the `detect_format()` function.
KFX files are SQLite databases; their first 16 bytes are the SQLite magic:
`53 51 4C 69 74 65 20 66 6F 72 6D 61 74 20 33 00` ("SQLite format 3\0").

After detecting SQLite magic, open the DB and check for the `fragments` table
(which is specific to KFX):
```rust
DetectedFormat::Kfx => { /* variant already added in R07a */ }
// In detect_format():
// After reading magic bytes:
if magic == b"SQLite format 3\0" {
    // Check if it has KFX-specific tables
    if is_kfx_database(path) {
        return Ok(DetectedFormat::Kfx);
    }
    return Err(ProcessingError::UnsupportedFormat("SQLite (non-KFX)".into()));
}

fn is_kfx_database(path: &Path) -> bool {
    use rusqlite::Connection;
    let Ok(conn) = Connection::open(path) else { return false };
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='fragments'",
        [], |r| r.get(0),
    ).unwrap_or(0);
    count > 0
}
```

Then run:
```bash
cargo test --workspace -- test_detect_kfx
git add processing/src/plugins/mod.rs processing/Cargo.toml
git commit -m "R07b-T01: KFX SQLite detection — detection test green"
```

---

## R07b-T02

In `processing/src/metadata/mod.rs`, add `pub mod kfx;`.

Write `processing/src/metadata/kfx.rs`:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use rusqlite::Connection;
use std::path::Path;

pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let conn = Connection::open(path)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut meta = BookMetadata::default();

    // KFX stores metadata in fragment_properties table
    // Standard keys: title, author, language, publisher, publication_date, description
    let mut stmt = conn.prepare(
        "SELECT key, value FROM fragment_properties
         WHERE fragment_id IN (
             SELECT id FROM fragments WHERE ftype = '$270'
         )"
    ).map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let rows = stmt.query_map([], |row| {
        let key: String = row.get(0)?;
        let value: Option<String> = row.get(1)?;
        Ok((key, value))
    }).map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    for row in rows {
        let (key, value) = row.map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        let value = match value { Some(v) if !v.is_empty() => v, _ => continue };
        match key.as_str() {
            "title"            => meta.title       = Some(value),
            "author"           => meta.authors.push(value),
            "language"         => meta.language    = Some(value),
            "publisher"        => meta.publisher   = Some(value),
            "publication_date" => meta.published   = Some(value),
            "description"      => meta.description = Some(value),
            "isbn"             => meta.isbn        = Some(value),
            _                  => {}
        }
    }
    Ok(meta)
}
```

Then run:
```bash
cargo test --workspace -- test_kfx_metadata
git add processing/src/metadata/kfx.rs processing/src/metadata/mod.rs
git commit -m "R07b-T02: KFX metadata extraction — metadata tests green"
```

---

## R07b-T03

In `processing/src/text/mod.rs`, add `pub mod kfx;`.

Write `processing/src/text/kfx.rs`:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use rusqlite::Connection;
use std::path::Path;

/// Extract text from KFX content fragments.
/// KFX content is stored in fragments with ftype '$608' (text content).
/// The payload is a binary blob; we extract readable UTF-8 sequences from it.
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let conn = Connection::open(path)
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let mut stmt = conn.prepare(
        "SELECT payload FROM fragments WHERE ftype = '$608' AND payload IS NOT NULL
         ORDER BY rowid"
    ).map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let blobs: Vec<Vec<u8>> = stmt.query_map([], |row| {
        let b: Vec<u8> = row.get(0)?;
        Ok(b)
    })
    .map_err(|e| ProcessingError::TextError(e.to_string()))?
    .filter_map(|r| r.ok())
    .collect();

    let mut parts = Vec::new();
    for blob in &blobs {
        // KFX content fragments may be UTF-8 text, or binary with embedded text.
        // Try direct UTF-8 first, then extract printable ASCII sequences.
        let text = if let Ok(s) = std::str::from_utf8(blob) {
            s.to_string()
        } else {
            extract_utf8_sequences(blob)
        };
        let cleaned = clean_kfx_text(&text);
        if !cleaned.is_empty() { parts.push(cleaned); }
    }

    let full_text  = parts.join("\n\n");
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}

fn extract_utf8_sequences(data: &[u8]) -> String {
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        if data[i] >= 0x20 && data[i] < 0x7F {
            let start = i;
            while i < data.len() && data[i] >= 0x20 && data[i] < 0x7F { i += 1; }
            if i - start >= 4 { // minimum meaningful sequence
                if let Ok(s) = std::str::from_utf8(&data[start..i]) {
                    result.push_str(s);
                    result.push(' ');
                }
            }
        } else {
            i += 1;
        }
    }
    result
}

fn clean_kfx_text(text: &str) -> String {
    // Strip KFX markup tags (similar to HTML stripping)
    let tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
    let cleaned = tag_re.replace_all(text, " ");
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}
```

Then run:
```bash
cargo test --workspace -- test_kfx
git add processing/src/text/kfx.rs processing/src/text/mod.rs
git commit -m "R07b-T03: KFX text extraction — all KFX tests green"
```

---

## R07b-T04

In `processing/src/pipeline/metadata.rs`, add KFX to the match:
```rust
DetectedFormat::Kfx => metadata::kfx::extract(path)?,
```

In `processing/src/pipeline/text.rs`, add KFX:
```rust
DetectedFormat::Kfx => text::kfx::extract(path)?,
```

Then run:
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/pipeline/metadata.rs processing/src/pipeline/text.rs
git commit -m "R07b-T04: wire KFX into pipeline metadata and text stages"
```

---

## R07b-T05 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

**Visual inspection:**
If you have a real `.kfx` file (from a Kindle device or Amazon download tool):
1. Drag it into xCalibre
2. Verify it appears in the library with title and author populated
3. Check that the format badge shows "KFX"
4. Open book details — word count should be populated

```bash
pkill -x xcalibre 2>/dev/null || true
```

```bash
git add -A
git commit -m "R07b-T05: RMP-07 KFX format — all tests green, pipeline wired"
```
