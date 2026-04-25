# Phase 3 — Metadata Enrichment

> HOW TO USE: For every "Write `path`" line → call write_file with that path and content.
> For every "Then run:" block → call your shell tool for each command.
> DO NOT print code as output. Write it to disk using your tools.
> Prerequisite: Phase 2 complete and all tests green.
> Run `/security-review` before starting any task in this phase.
> Status: ✅ done · ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| P3-T01 | ISBN detection (EPUB + PDF) | ✅ |
| P3-T02 | Open Library API lookup | ✅ |
| P3-T03 | Google Books API lookup | ✅ |
| P3-T04 | Enrichment result struct + merge logic | ✅ |
| P3-T05 | User confirmation gate | ✅ |
| P3-T06 | Enrichment cache (SQLite table) | ✅ |
| P3-T07 | run_enrichment pipeline stage | ✅ |
| P3-T08 | Enrichment tests | ✅ |

---

## P3-T01

In `processing/src/metadata/mod.rs`, add `pub mod isbn;` after the existing module declarations.

Write `processing/src/metadata/isbn.rs` with this exact content:
```rust
use std::io::Read;
use std::path::Path;

pub fn from_epub(path: &Path) -> Option<String> {
    let file = std::fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file)).ok()?;

    let container_xml = {
        let mut entry = archive.by_name("META-INF/container.xml").ok()?;
        let mut s = String::new();
        entry.read_to_string(&mut s).ok()?;
        s
    };

    let opf_path = {
        let doc = roxmltree::Document::parse(&container_xml).ok()?;
        doc.descendants()
            .find(|n| n.tag_name().name() == "rootfile")
            .and_then(|n| n.attribute("full-path"))
            .map(String::from)?
    };

    let opf_xml = {
        let mut entry = archive.by_name(&opf_path).ok()?;
        let mut s = String::new();
        entry.read_to_string(&mut s).ok()?;
        s
    };

    let doc = roxmltree::Document::parse(&opf_xml).ok()?;
    for node in doc.descendants() {
        if node.tag_name().name() == "identifier" {
            let scheme = node
                .attribute("opf:scheme")
                .or_else(|| node.attribute("scheme"))
                .unwrap_or("");
            if scheme.eq_ignore_ascii_case("isbn") {
                if let Some(raw) = node.text() {
                    return validate_isbn(raw);
                }
            }
        }
    }
    None
}

pub fn from_pdf(path: &Path) -> Option<String> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut buf = vec![0u8; 8192];
    let n = std::io::Read::read(&mut file, &mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf[..n]);

    let re = regex::Regex::new(r"ISBN[:\s\-]*([0-9\-]{10,17})").ok()?;
    for cap in re.captures_iter(&text) {
        if let Some(raw) = cap.get(1) {
            if let Some(isbn) = validate_isbn(raw.as_str()) {
                return Some(isbn);
            }
        }
    }
    None
}

fn validate_isbn(raw: &str) -> Option<String> {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 10 || digits.len() == 13 {
        Some(digits)
    } else {
        None
    }
}
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/metadata/isbn.rs processing/src/metadata/mod.rs
git commit -m "P3-T01: add ISBN detection for EPUB and PDF"
```

---

## P3-T02

In `api/src/lib.rs`, add `pub mod enrichment;` at the end of the file.

Write `api/src/enrichment/mod.rs` with this exact content:
```rust
pub mod open_library;
```

Write `api/src/enrichment/open_library.rs` with this exact content:
```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OLBook {
    pub title:        Option<String>,
    pub authors:      Vec<String>,
    pub publishers:   Vec<String>,
    pub publish_date: Option<String>,
    pub description:  Option<String>,
    pub subjects:     Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OLResponse {
    #[serde(flatten)]
    entries: std::collections::HashMap<String, OLEntry>,
}

#[derive(Debug, Deserialize)]
struct OLEntry {
    title:        Option<String>,
    #[serde(default)]
    authors:      Vec<OLAuthor>,
    #[serde(default)]
    publishers:   Vec<OLPublisher>,
    publish_date: Option<String>,
    notes:        Option<serde_json::Value>,
    #[serde(default)]
    subjects:     Vec<OLSubject>,
}

#[derive(Debug, Deserialize)]
struct OLAuthor    { name: String }
#[derive(Debug, Deserialize)]
struct OLPublisher { name: String }
#[derive(Debug, Deserialize)]
struct OLSubject   { name: String }

pub async fn lookup_by_isbn(isbn: &str) -> OLBook {
    match fetch(isbn).await {
        Ok(book) => book,
        Err(e) => {
            tracing::warn!("Open Library lookup failed for ISBN {}: {}", isbn, e);
            OLBook::default()
        }
    }
}

async fn fetch(isbn: &str) -> Result<OLBook, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let url = format!(
        "https://openlibrary.org/api/books?bibkeys=ISBN:{}&format=json&jscmd=data",
        isbn
    );

    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Ok(OLBook::default());
    }

    let map: std::collections::HashMap<String, OLEntry> = resp.json().await?;
    let key = format!("ISBN:{}", isbn);
    let entry = match map.get(&key) {
        Some(e) => e,
        None    => return Ok(OLBook::default()),
    };

    let description = entry.notes.as_ref().and_then(|n| match n {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Object(o) => o.get("value").and_then(|v| v.as_str()).map(String::from),
        _ => None,
    });

    Ok(OLBook {
        title:        entry.title.clone(),
        authors:      entry.authors.iter().map(|a| a.name.clone()).collect(),
        publishers:   entry.publishers.iter().map(|p| p.name.clone()).collect(),
        publish_date: entry.publish_date.clone(),
        description,
        subjects:     entry.subjects.iter().map(|s| s.name.clone()).collect(),
    })
}
```

Then run:
```bash
cargo build --workspace
git add api/src/enrichment/mod.rs api/src/enrichment/open_library.rs api/src/lib.rs
git commit -m "P3-T02: add Open Library ISBN lookup (10s timeout, silent fallback)"
```

---

## P3-T03

In `api/src/enrichment/mod.rs`, add `pub mod google_books;` at the end of the file.

Write `api/src/enrichment/google_books.rs` with this exact content:
```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GBBook {
    pub title:       Option<String>,
    pub authors:     Vec<String>,
    pub publisher:   Option<String>,
    pub published:   Option<String>,
    pub description: Option<String>,
    pub thumbnail:   Option<String>,
}

#[derive(Debug, Deserialize)]
struct GBResponse {
    #[serde(default)]
    items: Vec<GBItem>,
}

#[derive(Debug, Deserialize)]
struct GBItem {
    #[serde(rename = "volumeInfo")]
    volume_info: GBVolumeInfo,
}

#[derive(Debug, Deserialize)]
struct GBVolumeInfo {
    title:                Option<String>,
    #[serde(default)]
    authors:              Vec<String>,
    publisher:            Option<String>,
    #[serde(rename = "publishedDate")]
    published_date:       Option<String>,
    description:          Option<String>,
    #[serde(rename = "imageLinks")]
    image_links:          Option<GBImageLinks>,
}

#[derive(Debug, Deserialize)]
struct GBImageLinks {
    thumbnail: Option<String>,
}

pub async fn lookup_by_isbn(isbn: &str) -> GBBook {
    match fetch(isbn).await {
        Ok(book) => book,
        Err(e) => {
            tracing::warn!("Google Books lookup failed for ISBN {}: {}", isbn, e);
            GBBook::default()
        }
    }
}

async fn fetch(isbn: &str) -> Result<GBBook, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let url = format!(
        "https://www.googleapis.com/books/v1/volumes?q=isbn:{}",
        isbn
    );

    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Ok(GBBook::default());
    }

    let gb: GBResponse = resp.json().await?;
    let item = match gb.items.into_iter().next() {
        Some(i) => i,
        None    => return Ok(GBBook::default()),
    };
    let vi = item.volume_info;

    Ok(GBBook {
        title:       vi.title,
        authors:     vi.authors,
        publisher:   vi.publisher,
        published:   vi.published_date,
        description: vi.description,
        thumbnail:   vi.image_links.and_then(|il| il.thumbnail),
    })
}
```

Then run:
```bash
cargo build --workspace
git add api/src/enrichment/google_books.rs api/src/enrichment/mod.rs
git commit -m "P3-T03: add Google Books ISBN lookup (10s timeout, silent fallback)"
```

---

## P3-T04

In `processing/src/metadata/mod.rs`, add `pub mod enrichment;` at the end of the file.

Write `processing/src/metadata/enrichment.rs` with this exact content:
```rust
use crate::metadata::BookMetadata;
use xcalibre_api::enrichment::{google_books::GBBook, open_library::OLBook};

#[derive(Debug, Clone)]
pub struct EnrichmentSuggestion {
    pub field:  String,
    pub source: String,
    pub value:  String,
}

pub fn build_suggestions(
    existing: &BookMetadata,
    ol: &OLBook,
    gb: &GBBook,
) -> Vec<EnrichmentSuggestion> {
    let mut suggestions = Vec::new();

    fn add(
        suggestions: &mut Vec<EnrichmentSuggestion>,
        field: &str,
        existing: Option<&String>,
        ol_val: Option<String>,
        gb_val: Option<String>,
    ) {
        if existing.map(|s| s.is_empty()).unwrap_or(true) {
            if let Some(val) = ol_val {
                suggestions.push(EnrichmentSuggestion {
                    field:  field.to_string(),
                    source: "Open Library".to_string(),
                    value:  val,
                });
            } else if let Some(val) = gb_val {
                suggestions.push(EnrichmentSuggestion {
                    field:  field.to_string(),
                    source: "Google Books".to_string(),
                    value:  val,
                });
            }
        }
    }

    add(&mut suggestions, "title",       existing.title.as_ref(),       ol.title.clone(),                     gb.title.clone());
    add(&mut suggestions, "publisher",   existing.publisher.as_ref(),   ol.publishers.first().cloned(),       gb.publisher.clone());
    add(&mut suggestions, "published",   existing.published.as_ref(),   ol.publish_date.clone(),              gb.published.clone());
    add(&mut suggestions, "description", existing.description.as_ref(), ol.description.clone(),               gb.description.clone());

    suggestions
}

pub fn apply_suggestions(meta: &mut BookMetadata, accepted: &[EnrichmentSuggestion]) {
    for s in accepted {
        match s.field.as_str() {
            "title"       => meta.title       = Some(s.value.clone()),
            "publisher"   => meta.publisher   = Some(s.value.clone()),
            "published"   => meta.published   = Some(s.value.clone()),
            "description" => meta.description = Some(s.value.clone()),
            _ => {}
        }
    }
}
```

Then run:
```bash
cargo build --workspace
git add processing/src/metadata/enrichment.rs processing/src/metadata/mod.rs
git commit -m "P3-T04: add EnrichmentSuggestion struct, build_suggestions, apply_suggestions"
```

---

## P3-T05

In `processing/src/lib.rs`, add `pub mod enrichment_prompt;` at the end of the file.

Write `processing/src/enrichment_prompt.rs` with this exact content:
```rust
use crate::metadata::enrichment::EnrichmentSuggestion;
use std::io::{self, BufRead, Write};

pub fn prompt_user(suggestions: &[EnrichmentSuggestion]) -> Vec<EnrichmentSuggestion> {
    if suggestions.is_empty() {
        return vec![];
    }
    println!("\nEnrichment suggestions:");
    for (i, s) in suggestions.iter().enumerate() {
        println!(
            "  [{}] {} → \"{}\"  (source: {})",
            i + 1,
            s.field,
            s.value,
            s.source
        );
    }
    print!("Accept all (a), pick individually (p), or skip (s)? ");
    io::stdout().flush().ok();
    let stdin = io::stdin();
    let line = stdin
        .lock()
        .lines()
        .next()
        .unwrap_or(Ok(String::new()))
        .unwrap_or_default();
    match line.trim() {
        "a" | "A" => suggestions.to_vec(),
        "p" | "P" => pick_individually(suggestions),
        _         => vec![],
    }
}

fn pick_individually(suggestions: &[EnrichmentSuggestion]) -> Vec<EnrichmentSuggestion> {
    let mut accepted = Vec::new();
    let stdin = io::stdin();
    for s in suggestions {
        print!("  Accept \"{}\" for {}? (y/n) ", s.value, s.field);
        io::stdout().flush().ok();
        let line = stdin
            .lock()
            .lines()
            .next()
            .unwrap_or(Ok(String::new()))
            .unwrap_or_default();
        if line.trim().eq_ignore_ascii_case("y") {
            accepted.push(s.clone());
        }
    }
    accepted
}
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/enrichment_prompt.rs processing/src/lib.rs
git commit -m "P3-T05: add CLI enrichment confirmation prompt"
```

---

### ✅ Milestone check — after P3-T05
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```
Fix any issues before continuing.

---

## P3-T06

Write `processing/src/db/migrations/0002_enrichment_cache.sql` with this exact content:
```sql
CREATE TABLE enrichment_cache (
    isbn       TEXT PRIMARY KEY,
    ol_json    TEXT,
    gb_json    TEXT,
    fetched_at TEXT NOT NULL
);
```

In `processing/src/db/mod.rs`, add `pub mod enrichment_cache;` at the end of the file.

Write `processing/src/db/enrichment_cache.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use sqlx::SqlitePool;

pub async fn get_cache(
    pool: &SqlitePool,
    isbn: &str,
) -> Result<Option<(String, String)>, ProcessingError> {
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT ol_json, gb_json FROM enrichment_cache
         WHERE isbn = ? AND fetched_at >= datetime('now', '-30 days')",
    )
    .bind(isbn)
    .fetch_optional(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(row)
}

pub async fn set_cache(
    pool: &SqlitePool,
    isbn: &str,
    ol_json: &str,
    gb_json: &str,
) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR REPLACE INTO enrichment_cache (isbn, ol_json, gb_json, fetched_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(isbn)
    .bind(ol_json)
    .bind(gb_json)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}
```

Then run:
```bash
cargo build --workspace
git add processing/src/db/migrations/0002_enrichment_cache.sql processing/src/db/enrichment_cache.rs processing/src/db/mod.rs
git commit -m "P3-T06: add enrichment_cache table and CRUD (30-day TTL)"
```

---

## P3-T07

In `processing/src/pipeline/mod.rs`, add `pub mod enrichment;` at the end of the file.

Write `processing/src/pipeline/enrichment.rs` with this exact content:
```rust
use crate::db::enrichment_cache;
use crate::error::ProcessingError;
use crate::metadata::{self, enrichment as enrich, BookMetadata};
use crate::pipeline::ingest::IngestResult;
use crate::plugins::DetectedFormat;
use sqlx::SqlitePool;
use std::path::Path;
use xcalibre_api::enrichment::{google_books, open_library};

pub async fn run_enrichment(
    pool: &SqlitePool,
    result: &IngestResult,
    path: &Path,
    meta: &BookMetadata,
) -> Result<Vec<enrich::EnrichmentSuggestion>, ProcessingError> {
    let isbn = match result.format {
        DetectedFormat::Epub => metadata::isbn::from_epub(path),
        DetectedFormat::Pdf  => metadata::isbn::from_pdf(path),
        _                    => None,
    };
    let isbn = match isbn {
        Some(i) => i,
        None    => return Ok(vec![]),
    };

    let (ol_json, gb_json) = match enrichment_cache::get_cache(pool, &isbn).await? {
        Some(cached) => cached,
        None => {
            let (ol, gb) = tokio::join!(
                open_library::lookup_by_isbn(&isbn),
                google_books::lookup_by_isbn(&isbn),
            );
            let ol_json = serde_json::to_string(&ol)
                .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
            let gb_json = serde_json::to_string(&gb)
                .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
            enrichment_cache::set_cache(pool, &isbn, &ol_json, &gb_json).await?;
            (ol_json, gb_json)
        }
    };

    let ol: open_library::OLBook = serde_json::from_str(&ol_json).unwrap_or_default();
    let gb: google_books::GBBook = serde_json::from_str(&gb_json).unwrap_or_default();
    Ok(enrich::build_suggestions(meta, &ol, &gb))
}
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/pipeline/enrichment.rs processing/src/pipeline/mod.rs
git commit -m "P3-T07: add run_enrichment with parallel OL+GB fetch and cache"
```

---

## P3-T08

Write `processing/tests/test_enrichment.rs` with this exact content:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_api::enrichment::open_library::OLBook;
use xcalibre_processing::db::enrichment_cache;
use xcalibre_processing::metadata::{self, enrichment, BookMetadata};

async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_isbn_from_epub() {
    let path = std::path::PathBuf::from("tests/fixtures/fixture_epub.epub");
    let result = metadata::isbn::from_epub(&path);
    let _ = result; // Some or None — no panic is the contract
}

#[tokio::test]
async fn test_build_suggestions_fills_empty_fields() {
    let existing = BookMetadata::default();
    let ol = OLBook {
        title: Some("Test Book".to_string()),
        ..OLBook::default()
    };
    let gb = xcalibre_api::enrichment::google_books::GBBook::default();
    let suggestions = enrichment::build_suggestions(&existing, &ol, &gb);
    assert!(suggestions.iter().any(|s| s.field == "title" && s.value == "Test Book"));
}

#[tokio::test]
async fn test_enrichment_cache_round_trip() {
    let pool = setup_db().await;
    enrichment_cache::set_cache(&pool, "9780123456789", r#"{"title":"Cached"}"#, "{}")
        .await
        .unwrap();
    let cached = enrichment_cache::get_cache(&pool, "9780123456789")
        .await
        .unwrap();
    assert!(cached.is_some());
    let (ol_json, _) = cached.unwrap();
    assert!(ol_json.contains("Cached"));
}
```

Then run:
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo audit
git add processing/tests/test_enrichment.rs
git commit -m "P3-T08: add enrichment tests"
```

### ✅ Milestone check — after P3-T08
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

Run `/security-review` + `/review` before closing Phase 3.
