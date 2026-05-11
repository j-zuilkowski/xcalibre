# RMP-07a — KFX Format Support (Red: Failing Tests)

> Prerequisite: rmp01b complete.
> Decision: S5-A — full implementation (text + metadata extraction, 4–6 weeks).
> TDD role: RED — define the KFX parser API via failing tests.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R07a-T01 | Failing tests: KFX format detection | ⬜ |
| R07a-T02 | Failing tests: KFX metadata extraction | ⬜ |
| R07a-T03 | Failing tests: KFX text extraction | ⬜ |

---

## R07a-T01

In `processing/src/plugins/mod.rs`, add `DetectedFormat::Kfx` to the enum
(the variant itself; the extraction logic comes in rmp07b):
```rust
// Add to the DetectedFormat enum:
Kfx,
```

Also update all `match` exhaustiveness checks in the codebase to handle `Kfx`
(use `_ => {}` or `DetectedFormat::Kfx => {}` as appropriate).

Write `processing/tests/test_kfx.rs` with this exact content:
```rust
//! Tests for KFX format support (RMP-07, S5-A: full implementation).
//! FAIL until rmp07b implements the KFX parser.

use std::path::PathBuf;
use xcalibre_processing::plugins::{detect_format, DetectedFormat};
use xcalibre_processing::metadata::BookMetadata;
use xcalibre_processing::text::ExtractedText;

fn kfx_fixture() -> PathBuf {
    PathBuf::from("tests/fixtures/fixture.kfx")
}

/// Build a minimal .kfx fixture (SQLite DB with the required KFX tables).
fn create_kfx_fixture() {
    // KFX is a SQLite database; create one with the minimal structure
    let path = kfx_fixture();
    if path.exists() { return; }
    // Use rusqlite to create a minimal KFX-like SQLite DB
    // KFX magic: first 16 bytes are SQLite header "SQLite format 3\000"
    // KFX has tables: fragments, fragment_properties, etc.
    // We create a minimal version that our parser can read.
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch("
        CREATE TABLE fragments (
            id          TEXT PRIMARY KEY,
            ftype       TEXT NOT NULL,
            payload     BLOB
        );
        CREATE TABLE fragment_properties (
            fragment_id TEXT NOT NULL,
            key         TEXT NOT NULL,
            value       TEXT
        );
        INSERT INTO fragments VALUES ('meta', '$270', NULL);
        INSERT INTO fragment_properties VALUES ('meta', 'title', 'KFX Test Book');
        INSERT INTO fragment_properties VALUES ('meta', 'author', 'KFX Author');
        INSERT INTO fragment_properties VALUES ('meta', 'language', 'en');
        INSERT INTO fragments VALUES ('content_1', '$608', X'48656C6C6F20776F726C64');
    ").unwrap();
}

#[test]
fn test_detect_kfx_format() {
    create_kfx_fixture();
    let fmt = detect_format(&kfx_fixture()).expect("detect");
    assert_eq!(fmt, DetectedFormat::Kfx, "KFX SQLite file must be detected as Kfx");
}

#[test]
fn test_kfx_metadata_title() {
    create_kfx_fixture();
    let meta = xcalibre_processing::metadata::kfx::extract(&kfx_fixture())
        .expect("kfx metadata");
    assert_eq!(meta.title.as_deref(), Some("KFX Test Book"));
}

#[test]
fn test_kfx_metadata_author() {
    create_kfx_fixture();
    let meta = xcalibre_processing::metadata::kfx::extract(&kfx_fixture()).unwrap();
    assert_eq!(meta.authors, vec!["KFX Author"]);
}

#[test]
fn test_kfx_text_extraction() {
    create_kfx_fixture();
    let text = xcalibre_processing::text::kfx::extract(&kfx_fixture())
        .expect("kfx text");
    assert!(text.word_count > 0, "KFX text extraction must return non-empty text");
    assert!(text.full_text.contains("Hello") || text.full_text.contains("world"),
            "extracted text: {}", text.full_text);
}
```

Add `rusqlite` to `processing/Cargo.toml` dev-dependencies:
```toml
[dev-dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
```

Then run:
```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -10
```

Expected: `metadata::kfx` and `text::kfx` modules not found. RED confirmed.

```bash
git add processing/tests/test_kfx.rs processing/Cargo.toml \
        processing/src/plugins/mod.rs
git commit -m "R07a-T01: DetectedFormat::Kfx + failing KFX detection/extraction tests"
```

---

### ✅ RED Checkpoint

Proceed to **rmp07b**.
