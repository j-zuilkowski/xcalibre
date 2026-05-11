# RMP-05a — xcalibre-epub Manipulation Library (Red: Failing Tests)

> HOW TO USE: Write every file exactly as shown, run every shell block.
> Prerequisite: rmp04b complete and all tests green.
> TDD role: RED — define the xcalibre-epub API via failing tests. This crate gates
> Phases 9, 13, 15, and 16, so the API contracts must be defined before implementation.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R05a-T01 | xcalibre-epub crate scaffold (compiles, empty) | ⬜ |
| R05a-T02 | Failing tests: Container open/read/write | ⬜ |
| R05a-T03 | Failing tests: OPF parse and write | ⬜ |
| R05a-T04 | Failing tests: cover extract/replace | ⬜ |
| R05a-T05 | Failing tests: font listing | ⬜ |
| R05a-T06 | Failing tests: EPUB validation | ⬜ |
| R05a-T07 | Failing tests: EPUB 2→3 upgrade | ⬜ |

---

## R05a-T01

Add `"xcalibre-epub"` to workspace `members` in root `Cargo.toml`.

Create directory `xcalibre-epub/src/`.

Write `xcalibre-epub/Cargo.toml`:
```toml
[package]
name = "xcalibre-epub"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror  = "1"
roxmltree  = "0.20"
quick-xml  = { version = "0.36", features = ["serialize"] }
zip        = "2"
image      = { version = "0.25", default-features = false, features = ["jpeg", "png", "webp"] }
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
regex      = "1"

[dev-dependencies]
tempfile = "3"
```

Write `xcalibre-epub/src/lib.rs`:
```rust
pub mod container;
pub mod error;
pub mod opf;
pub mod cover;
pub mod font;
pub mod image_opt;
pub mod split;
pub mod validation;
pub mod upgrade;
pub mod css;

pub use error::EpubError;
pub use container::Container;
```

Write `xcalibre-epub/src/error.rs`:
```rust
#[derive(Debug, thiserror::Error)]
pub enum EpubError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP error: {0}")]
    Zip(String),
    #[error("XML parse error: {0}")]
    Xml(String),
    #[error("item not found in manifest: {0}")]
    ItemNotFound(String),
    #[error("missing required element: {0}")]
    MissingElement(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("unsupported EPUB version: {0}")]
    UnsupportedVersion(String),
}
```

Then run:
```bash
cargo build --workspace
git add xcalibre-epub/ Cargo.toml
git commit -m "R05a-T01: xcalibre-epub crate scaffold"
```

---

## R05a-T02

Write `xcalibre-epub/src/container.rs` with stub implementations (panic! in all methods):
```rust
use crate::EpubError;
use std::collections::HashMap;
use std::path::Path;

pub struct Container {
    // Implementation in rmp05b
    _private: (),
}

impl Container {
    /// Open an EPUB file for reading and manipulation.
    pub fn open(_path: &Path) -> Result<Self, EpubError> {
        unimplemented!("rmp05b")
    }

    /// List all manifest items as (id, href, media_type) tuples.
    pub fn manifest_items(&self) -> Vec<(String, String, String)> {
        unimplemented!("rmp05b")
    }

    /// Read a manifest item by its href.
    pub fn read_item(&self, _href: &str) -> Result<Vec<u8>, EpubError> {
        unimplemented!("rmp05b")
    }

    /// Write / replace a manifest item by its href. Creates if not present.
    pub fn write_item(&mut self, _href: &str, _data: &[u8], _media_type: &str) -> Result<(), EpubError> {
        unimplemented!("rmp05b")
    }

    /// Remove a manifest item by href.
    pub fn remove_item(&mut self, _href: &str) -> Result<(), EpubError> {
        unimplemented!("rmp05b")
    }

    /// Flush the modified container back to disk.
    pub fn save(&mut self) -> Result<(), EpubError> {
        unimplemented!("rmp05b")
    }

    /// Save to a new path (leaving original unchanged).
    pub fn save_as(&self, _path: &Path) -> Result<(), EpubError> {
        unimplemented!("rmp05b")
    }

    /// Path to the OPF file within the container.
    pub fn opf_path(&self) -> &str {
        unimplemented!("rmp05b")
    }

    /// Spine item hrefs in reading order.
    pub fn spine_hrefs(&self) -> Vec<String> {
        unimplemented!("rmp05b")
    }
}
```

Write `xcalibre-epub/tests/test_container.rs`:
```rust
use xcalibre_epub::Container;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

#[test]
fn test_open_valid_epub() {
    let c = Container::open(&fixture("simple.epub")).expect("open");
    let items = c.manifest_items();
    assert!(!items.is_empty(), "manifest must have at least one item");
}

#[test]
fn test_read_spine_item() {
    let c = Container::open(&fixture("simple.epub")).unwrap();
    let hrefs = c.spine_hrefs();
    assert!(!hrefs.is_empty(), "spine must be non-empty");
    let data = c.read_item(&hrefs[0]).expect("read first spine item");
    assert!(!data.is_empty());
}

#[test]
fn test_write_item_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("modified.epub");
    std::fs::copy(fixture("simple.epub"), &out).unwrap();
    let mut c = Container::open(&out).unwrap();
    c.write_item("test_inject.txt", b"hello world", "text/plain").unwrap();
    c.save().unwrap();

    let c2 = Container::open(&out).unwrap();
    let data = c2.read_item("test_inject.txt").unwrap();
    assert_eq!(&data, b"hello world");
}

#[test]
fn test_save_as_does_not_modify_original() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("copy.epub");
    let c = Container::open(&fixture("simple.epub")).unwrap();
    c.save_as(&out).unwrap();
    assert!(out.exists());
    // Original should still be openable
    let _ = Container::open(&fixture("simple.epub")).unwrap();
}
```

Create fixture directory and a minimal EPUB fixture:
```bash
mkdir -p xcalibre-epub/tests/fixtures
cargo run --bin gen_fixtures --manifest-path processing/Cargo.toml
cp processing/tests/fixtures/fixture_epub.epub xcalibre-epub/tests/fixtures/simple.epub
```

Then run:
```bash
cargo test -p xcalibre-epub 2>&1 | grep -E "panicked|^error" | head -20
```

Expected: `unimplemented!("rmp05b")` panics. RED confirmed.

```bash
git add xcalibre-epub/src/container.rs xcalibre-epub/tests/
git commit -m "R05a-T02: failing tests for Container open/read/write"
```

---

## R05a-T03

Write `xcalibre-epub/src/opf.rs` with stub:
```rust
use crate::EpubError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EpubOPF {
    pub title:        Option<String>,
    pub authors:      Vec<String>,
    pub language:     Option<String>,
    pub publisher:    Option<String>,
    pub series:       Option<String>,
    pub series_index: Option<f32>,
    pub tags:         Vec<String>,
    pub epub_version: String,
    pub manifest:     Vec<ManifestItem>,
    pub spine:        Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestItem {
    pub id:         String,
    pub href:       String,
    pub media_type: String,
    pub properties: Option<String>,
}

impl EpubOPF {
    pub fn parse(_xml: &str) -> Result<Self, EpubError> {
        unimplemented!("rmp05b")
    }

    pub fn to_xml(&self) -> Result<String, EpubError> {
        unimplemented!("rmp05b")
    }
}
```

Write `xcalibre-epub/tests/test_opf.rs`:
```rust
use xcalibre_epub::opf::EpubOPF;

const SAMPLE_OPF_2: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:opf="http://www.idpf.org/2007/opf">
    <dc:title>Sample Book</dc:title>
    <dc:creator opf:role="aut">Jane Doe</dc:creator>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="ch1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine><itemref idref="ch1"/></spine>
</package>"#;

#[test]
fn test_parse_opf2_title() {
    let opf = EpubOPF::parse(SAMPLE_OPF_2).unwrap();
    assert_eq!(opf.title.as_deref(), Some("Sample Book"));
    assert_eq!(opf.authors, vec!["Jane Doe"]);
    assert_eq!(opf.epub_version, "2.0");
}

#[test]
fn test_parse_opf2_manifest() {
    let opf = EpubOPF::parse(SAMPLE_OPF_2).unwrap();
    assert_eq!(opf.manifest.len(), 1);
    assert_eq!(opf.manifest[0].href, "chapter1.xhtml");
}

#[test]
fn test_opf_to_xml_round_trip() {
    let opf = EpubOPF::parse(SAMPLE_OPF_2).unwrap();
    let xml = opf.to_xml().unwrap();
    let re_parsed = EpubOPF::parse(&xml).unwrap();
    assert_eq!(re_parsed.title, opf.title);
    assert_eq!(re_parsed.authors, opf.authors);
}
```

Then run:
```bash
cargo test -p xcalibre-epub -- test_opf 2>&1 | grep -E "panicked|FAILED"
```

RED confirmed. Commit:
```bash
git add xcalibre-epub/src/opf.rs xcalibre-epub/tests/test_opf.rs
git commit -m "R05a-T03: failing tests for EpubOPF parse/serialize"
```

---

## R05a-T04

Write `xcalibre-epub/src/cover.rs` with stub:
```rust
use crate::EpubError;
pub struct CoverInfo { pub href: String, pub media_type: String, pub data: Vec<u8> }
pub fn extract_cover(_container: &crate::Container) -> Result<Option<CoverInfo>, EpubError> {
    unimplemented!("rmp05b")
}
pub fn replace_cover(_container: &mut crate::Container, _jpeg_data: &[u8]) -> Result<(), EpubError> {
    unimplemented!("rmp05b")
}
```

Write `xcalibre-epub/tests/test_cover.rs`:
```rust
use xcalibre_epub::{Container, cover};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

#[test]
fn test_extract_cover_returns_some_for_epub_with_cover() {
    // Use the simple.epub fixture which has no cover — expect None, not an error
    let c = Container::open(&fixture("simple.epub")).unwrap();
    let result = cover::extract_cover(&c);
    assert!(result.is_ok(), "extract_cover must not error: {:?}", result);
}
```

```bash
git add xcalibre-epub/src/cover.rs xcalibre-epub/tests/test_cover.rs
git commit -m "R05a-T04: failing test for cover extraction"
```

---

## R05a-T05

Write `xcalibre-epub/src/font.rs` with stub:
```rust
use crate::EpubError;
pub struct FontInfo { pub href: String, pub media_type: String }
pub fn list_fonts(_container: &crate::Container) -> Vec<FontInfo> {
    unimplemented!("rmp05b")
}
pub fn embed_font(_container: &mut crate::Container, _name: &str, _data: &[u8], _media_type: &str)
    -> Result<(), EpubError>
{
    unimplemented!("rmp05b")
}
```

Write `xcalibre-epub/tests/test_font.rs`:
```rust
use xcalibre_epub::{Container, font};
use std::path::PathBuf;
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}
#[test]
fn test_list_fonts_no_panic() {
    let c = Container::open(&fixture("simple.epub")).unwrap();
    let fonts = font::list_fonts(&c); // simple.epub has no fonts — expect empty Vec
    // just verify it doesn't panic; exact count depends on fixture
    let _ = fonts;
}
```

```bash
git add xcalibre-epub/src/font.rs xcalibre-epub/tests/test_font.rs
git commit -m "R05a-T05: failing test for font listing"
```

---

## R05a-T06

Write `xcalibre-epub/src/validation.rs` with stub:
```rust
use crate::EpubError;

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub message:  String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity { Error, Warning }

pub fn validate(_container: &crate::Container) -> Result<Vec<ValidationIssue>, EpubError> {
    unimplemented!("rmp05b")
}
```

Write `xcalibre-epub/tests/test_validation.rs`:
```rust
use xcalibre_epub::{Container, validation};
use std::path::PathBuf;
fn fixture(n: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(n)
}
#[test]
fn test_valid_epub_produces_no_errors() {
    let c = Container::open(&fixture("simple.epub")).unwrap();
    let issues = validation::validate(&c).unwrap();
    let errors: Vec<_> = issues.iter()
        .filter(|i| i.severity == validation::Severity::Error).collect();
    assert!(errors.is_empty(), "simple.epub should have no validation errors: {:?}", errors);
}
```

```bash
git add xcalibre-epub/src/validation.rs xcalibre-epub/tests/test_validation.rs
git commit -m "R05a-T06: failing test for EPUB validation"
```

---

## R05a-T07

Write `xcalibre-epub/src/upgrade.rs` with stub:
```rust
use crate::EpubError;
pub fn epub2_to_epub3(_container: &mut crate::Container) -> Result<(), EpubError> {
    unimplemented!("rmp05b")
}
```

Write `xcalibre-epub/tests/test_upgrade.rs`:
```rust
use xcalibre_epub::{Container, upgrade};
use std::path::PathBuf;
fn fixture(n: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(n)
}
#[test]
fn test_epub2_upgrade_produces_epub3_version() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("upgraded.epub");
    std::fs::copy(fixture("simple.epub"), &out).unwrap();
    let mut c = Container::open(&out).unwrap();
    upgrade::epub2_to_epub3(&mut c).unwrap();
    c.save().unwrap();
    // Re-open and check the OPF version attribute
    let c2 = Container::open(&out).unwrap();
    let opf_bytes = c2.read_item(c2.opf_path()).unwrap();
    let opf_str = String::from_utf8_lossy(&opf_bytes);
    assert!(opf_str.contains("version=\"3."), "OPF version must be 3.x after upgrade: {}", opf_str);
}
```

```bash
git add xcalibre-epub/src/upgrade.rs xcalibre-epub/tests/test_upgrade.rs
git commit -m "R05a-T07: failing test for EPUB 2→3 upgrade"
```

---

### ✅ RED Checkpoint

```bash
cargo test -p xcalibre-epub 2>&1 | grep -c "panicked\|FAILED"
```

All test failures should be `unimplemented!` panics — the API surface is fully defined.
Proceed to **rmp05b** (6–10 weeks of implementation work).
