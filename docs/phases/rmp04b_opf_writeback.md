# RMP-04b — OPF Write-back & Calibre Round-trip (Green: Implementation)

> HOW TO USE: Write every file exactly as shown, run every shell block.
> Prerequisite: rmp04a complete (failing tests committed).
> TDD role: GREEN — implement until all tests pass.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R04b-T01 | `opf/mod.rs` — OPFMetadata, read_opf(), write_opf_sidecar() | ⬜ |
| R04b-T02 | `db/queries.rs` — update_book_metadata_with_opf() | ⬜ |
| R04b-T03 | Tauri command: update_book_metadata | ⬜ |
| R04b-T04 | Milestone check + visual inspection | ⬜ |

---

## R04b-T01

In `processing/Cargo.toml`, add under `[dependencies]`:
```toml
quick-xml = { version = "0.36", features = ["serialize"] }
```

In `processing/src/lib.rs`, add `pub mod opf;`.

Write `processing/src/opf/mod.rs`:
```rust
use crate::error::ProcessingError;
use quick_xml::{events::*, Writer};
use std::io::Cursor;
use std::path::Path;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OPFMetadata {
    pub title:        Option<String>,
    pub authors:      Vec<String>,
    pub language:     Option<String>,
    pub publisher:    Option<String>,
    pub published:    Option<String>,
    pub description:  Option<String>,
    pub isbn:         Option<String>,
    pub series:       Option<String>,
    pub series_index: Option<f32>,
    pub tags:         Vec<String>,
    pub calibre_id:   Option<String>,
}

/// Read an OPF 2.x / 3.x file into OPFMetadata.
pub fn read_opf(path: &Path) -> Result<OPFMetadata, ProcessingError> {
    let xml = std::fs::read_to_string(path).map_err(ProcessingError::IoError)?;
    let doc = roxmltree::Document::parse(&xml)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    let mut meta = OPFMetadata::default();
    for node in doc.descendants() {
        match node.tag_name().name() {
            "title"       => meta.title       = node.text().map(str::trim).map(String::from),
            "creator"     => meta.authors.push(node.text().unwrap_or("").trim().to_string()),
            "language"    => meta.language    = node.text().map(str::trim).map(String::from),
            "publisher"   => meta.publisher   = node.text().map(str::trim).map(String::from),
            "date"        => meta.published   = node.text().map(str::trim).map(String::from),
            "description" => meta.description = node.text().map(str::trim).map(String::from),
            "identifier"  => {
                let scheme = node.attribute("opf:scheme")
                    .or_else(|| node.attribute("scheme")).unwrap_or("");
                if scheme.eq_ignore_ascii_case("isbn") {
                    meta.isbn = node.text().map(str::trim).map(String::from);
                }
            }
            "subject" => meta.tags.push(node.text().unwrap_or("").trim().to_string()),
            "meta" => {
                let name = node.attribute("name").unwrap_or("");
                let content = node.attribute("content").unwrap_or("");
                match name {
                    "calibre:series"       => meta.series = Some(content.to_string()),
                    "calibre:series_index" => meta.series_index = content.parse().ok(),
                    _                      => {}
                }
            }
            _ => {}
        }
    }
    meta.authors.retain(|a| !a.is_empty());
    meta.tags.retain(|t| !t.is_empty());
    Ok(meta)
}

/// Write OPFMetadata as a Calibre-compatible OPF 2.x sidecar file.
pub fn write_opf_sidecar(path: &Path, meta: &OPFMetadata) -> Result<(), ProcessingError> {
    let mut w = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    // XML declaration
    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut pkg = BytesStart::new("package");
    pkg.push_attribute(("xmlns", "http://www.idpf.org/2007/opf"));
    pkg.push_attribute(("version", "2.0"));
    pkg.push_attribute(("unique-identifier", "uuid_id"));
    w.write_event(Event::Start(pkg))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut md = BytesStart::new("metadata");
    md.push_attribute(("xmlns:dc", "http://purl.org/dc/elements/1.1/"));
    md.push_attribute(("xmlns:opf", "http://www.idpf.org/2007/opf"));
    md.push_attribute(("xmlns:calibre", "http://calibre.kovidgoyal.net/2009/metadata"));
    w.write_event(Event::Start(md))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    write_dc_element(&mut w, "title", meta.title.as_deref().unwrap_or("Unknown"))?;
    for author in &meta.authors {
        write_dc_element_with_attr(&mut w, "creator", author, &[("opf:role", "aut")])?;
    }
    write_dc_element(&mut w, "language", meta.language.as_deref().unwrap_or("en"))?;
    if let Some(p) = &meta.publisher   { write_dc_element(&mut w, "publisher",   p)?; }
    if let Some(d) = &meta.published   { write_dc_element(&mut w, "date",        d)?; }
    if let Some(d) = &meta.description { write_dc_element(&mut w, "description", d)?; }
    if let Some(isbn) = &meta.isbn {
        write_dc_element_with_attr(&mut w, "identifier", isbn, &[("opf:scheme", "ISBN")])?;
    }
    for tag in &meta.tags {
        write_dc_element(&mut w, "subject", tag)?;
    }
    if let Some(s) = &meta.series {
        write_meta(&mut w, "calibre:series", s)?;
        if let Some(idx) = meta.series_index {
            write_meta(&mut w, "calibre:series_index", &idx.to_string())?;
        }
    }

    w.write_event(Event::End(BytesEnd::new("metadata")))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    w.write_event(Event::End(BytesEnd::new("package")))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let xml = String::from_utf8(w.into_inner().into_inner())
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    std::fs::write(path, xml).map_err(ProcessingError::IoError)
}

fn write_dc_element(w: &mut Writer<Cursor<Vec<u8>>>, name: &str, text: &str)
    -> Result<(), ProcessingError>
{
    let tag = format!("dc:{}", name);
    w.write_event(Event::Start(BytesStart::new(&tag)))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    w.write_event(Event::Text(BytesText::new(text)))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    w.write_event(Event::End(BytesEnd::new(&tag)))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))
}

fn write_dc_element_with_attr(
    w: &mut Writer<Cursor<Vec<u8>>>, name: &str, text: &str, attrs: &[(&str, &str)],
) -> Result<(), ProcessingError> {
    let tag = format!("dc:{}", name);
    let mut start = BytesStart::new(&tag);
    for (k, v) in attrs { start.push_attribute((*k, *v)); }
    w.write_event(Event::Start(start))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    w.write_event(Event::Text(BytesText::new(text)))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    w.write_event(Event::End(BytesEnd::new(&tag)))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))
}

fn write_meta(w: &mut Writer<Cursor<Vec<u8>>>, name: &str, content: &str)
    -> Result<(), ProcessingError>
{
    let mut start = BytesStart::new("meta");
    start.push_attribute(("name", name));
    start.push_attribute(("content", content));
    w.write_event(Event::Empty(start))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))
}
```

Then run:
```bash
cargo test --workspace -- test_opf_write
cargo test --workspace -- test_opf_roundtrip
git add processing/src/opf/mod.rs processing/src/lib.rs processing/Cargo.toml
git commit -m "R04b-T01: OPF read/write — OPF serialization and round-trip tests green"
```

---

## R04b-T02

In `processing/src/db/queries.rs`, append the following function:
```rust
use crate::opf::{write_opf_sidecar, OPFMetadata};
use crate::metadata::BookMetadata;

pub async fn update_book_metadata_with_opf(
    pool: &SqlitePool,
    book_id: &str,
    meta: &BookMetadata,
) -> Result<(), ProcessingError> {
    let authors_json = serde_json::to_string(&meta.authors)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    let tags_json = serde_json::to_string(&meta.tags)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "UPDATE local_books SET title=COALESCE(?,title), authors_json=?, publisher=?,
         series=?, series_index=?, language=?, tags_json=?, updated_at=?
         WHERE id=?",
    )
    .bind(&meta.title).bind(&authors_json).bind(&meta.publisher)
    .bind(&meta.series).bind(meta.series_index).bind(&meta.language)
    .bind(&tags_json).bind(&now).bind(book_id)
    .execute(pool).await.map_err(ProcessingError::DbError)?;

    // Write OPF sidecar if a file_path is known
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT file_path FROM local_books WHERE id = ?",
    )
    .bind(book_id)
    .fetch_optional(pool).await.map_err(ProcessingError::DbError)?;

    if let Some((file_path,)) = row {
        if let Some(book_dir) = std::path::Path::new(&file_path).parent() {
            let opf_path = book_dir.join("metadata.opf");
            let opf_meta = OPFMetadata {
                title:        meta.title.clone(),
                authors:      meta.authors.clone(),
                language:     meta.language.clone(),
                publisher:    meta.publisher.clone(),
                published:    meta.published.clone(),
                description:  meta.description.clone(),
                isbn:         meta.isbn.clone(),
                series:       meta.series.clone(),
                series_index: meta.series_index,
                tags:         meta.tags.clone(),
                calibre_id:   None,
            };
            let _ = write_opf_sidecar(&opf_path, &opf_meta); // best-effort; don't fail the update
        }
    }
    Ok(())
}
```

Then run:
```bash
cargo test --workspace -- test_opf_trigger
git add processing/src/db/queries.rs
git commit -m "R04b-T02: update_book_metadata_with_opf — trigger tests green"
```

---

## R04b-T03

In `src-tauri/src/commands.rs`, update or add:
```rust
use xcalibre_processing::metadata::BookMetadata;

#[tauri::command]
pub async fn update_book_metadata(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    book_id: String,
    title: Option<String>,
    authors: Vec<String>,
    publisher: Option<String>,
    series: Option<String>,
    series_index: Option<f32>,
    language: Option<String>,
    tags: Vec<String>,
    description: Option<String>,
    isbn: Option<String>,
) -> Result<(), String> {
    let meta = BookMetadata {
        title, authors, publisher, series, series_index,
        language, tags, description, isbn, published: None,
    };
    xcalibre_processing::db::queries::update_book_metadata_with_opf(
        pool.inner().as_ref(), &book_id, &meta,
    )
    .await
    .map_err(|e| e.to_string())
}
```

Register in `generate_handler!`. Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "R04b-T03: update_book_metadata Tauri command"
```

---

## R04b-T04 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm run build && npm test && cd ..
```

**Visual inspection:**
1. Ingest an EPUB from a Calibre managed library (one that already has a `metadata.opf` sidecar)
2. Edit the book's title in xCalibre's metadata editor
3. Verify that the `metadata.opf` sidecar in the book's directory has been updated with the new title
4. Re-import the book into Calibre and verify the title change is reflected

```bash
git add -A
git commit -m "R04b-T04: RMP-04 OPF write-back — all tests green, round-trip verified"
```
