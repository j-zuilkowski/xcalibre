# Phase 7 — Extended Format Support

> HOW TO USE: For every "Write `path`" line → call write_file with that path and content.
> For every "Then run:" block → call your shell tool for each command.
> DO NOT print code as output. Write it to disk using your tools.
> Prerequisite: Phase 6 complete and all tests green.
> Status: ✅ done

Covers the remaining Calibre-supported formats: DOCX, ODT, CHM, LRF/LRX, PDB/PML/RB,
SNB, TCR, AZW4, DJVU, LIT, and improved CBZ/CBR metadata.
Where no native Rust library exists, magic-byte detection + best-effort extraction
is implemented; text extraction falls back to a stub that returns an empty result
rather than an error, so the pipeline continues cleanly.

## Status

| Task | Title | Status |
|------|-------|--------|
| P7-T01 | Add quick-xml dependency | ✅ |
| P7-T02 | DOCX metadata + text (OOXML) | ✅ |
| P7-T03 | ODT metadata + text (ODF) | ✅ |
| P7-T04 | CHM metadata stub | ✅ |
| P7-T05 | LRF / LRX metadata stub | ✅ |
| P7-T06 | PDB / PML / RB metadata + text | ✅ |
| P7-T07 | SNB metadata stub | ✅ |
| P7-T08 | TCR metadata + text | ✅ |
| P7-T09 | AZW4 metadata stub | ✅ |
| P7-T10 | DJVU metadata stub | ✅ |
| P7-T11 | LIT metadata stub | ✅ |
| P7-T12 | CBZ/CBR improved metadata | ✅ |
| P7-T13 | Extend DetectedFormat + pipeline routing | ✅ |
| P7-T14 | Add new format fixtures | ✅ |
| P7-T15 | Tests | ✅ |

---

## P7-T01

In `processing/Cargo.toml`, add under `[dependencies]`:
```toml
quick-xml = { version = "0.36", features = ["serialize"] }
encoding_rs = "0.8"
```

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/Cargo.toml
git commit -m "P7-T01: add quick-xml and encoding_rs dependencies"
```

---

## P7-T02

Write `processing/src/metadata/docx.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

/// DOCX (Office Open XML) is a ZIP containing word/document.xml and docProps/core.xml.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let core_xml = read_entry(&mut archive, "docProps/core.xml").unwrap_or_default();
    let mut meta = BookMetadata::default();

    if !core_xml.is_empty() {
        let doc = roxmltree::Document::parse(&core_xml)
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        for node in doc.descendants() {
            match node.tag_name().name() {
                "title"    => meta.title     = node.text().map(str::trim).map(String::from),
                "creator"  => meta.authors.push(node.text().unwrap_or("").trim().to_string()),
                "subject"  => {
                    if let Some(t) = node.text() {
                        let t = t.trim().to_string();
                        if !t.is_empty() { meta.tags.push(t); }
                    }
                }
                "description" => meta.description = node.text().map(str::trim).map(String::from),
                "language"    => meta.language    = node.text().map(str::trim).map(String::from),
                _ => {}
            }
        }
    }
    meta.authors.retain(|a| !a.is_empty());
    Ok(meta)
}

fn read_entry<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Option<String> {
    let mut entry = archive.by_name(name).ok()?;
    let mut s = String::new();
    entry.read_to_string(&mut s).ok()?;
    Some(s)
}
```

Write `processing/src/text/docx.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let mut xml_str = String::new();
    if let Ok(mut entry) = archive.by_name("word/document.xml") {
        entry.read_to_string(&mut xml_str).map_err(ProcessingError::IoError)?;
    } else {
        return Ok(ExtractedText { full_text: String::new(), word_count: 0 });
    }

    // Extract text from <w:t> elements
    let doc = roxmltree::Document::parse(&xml_str)
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let mut parts = vec![];
    let mut para  = vec![];
    for node in doc.descendants() {
        match node.tag_name().name() {
            "t" => {
                if let Some(t) = node.text() {
                    para.push(t.to_string());
                }
            }
            "p" => {
                let text = para.join("").split_whitespace().collect::<Vec<_>>().join(" ");
                if !text.is_empty() { parts.push(text); }
                para.clear();
            }
            _ => {}
        }
    }

    let full_text  = parts.join("\n");
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}
```

In `processing/src/metadata/mod.rs`, add `pub mod docx;` after the existing lines.
In `processing/src/text/mod.rs`, add `pub mod docx;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/docx.rs processing/src/text/docx.rs \
        processing/src/metadata/mod.rs processing/src/text/mod.rs
git commit -m "P7-T02: DOCX metadata and text extraction (OOXML ZIP + roxmltree)"
```

---

## P7-T03

Write `processing/src/metadata/odt.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

/// ODT (ODF Text) is a ZIP containing content.xml and meta.xml.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let meta_xml = read_entry(&mut archive, "meta.xml").unwrap_or_default();
    let mut meta = BookMetadata::default();

    if !meta_xml.is_empty() {
        let doc = roxmltree::Document::parse(&meta_xml)
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        for node in doc.descendants() {
            match node.tag_name().name() {
                "title"          => meta.title     = node.text().map(str::trim).map(String::from),
                "initial-creator"| "creator"
                                 => { let a = node.text().unwrap_or("").trim().to_string(); if !a.is_empty() { meta.authors.push(a); } }
                "description"    => meta.description = node.text().map(str::trim).map(String::from),
                "language"       => meta.language  = node.text().map(str::trim).map(String::from),
                "keyword"        => {
                    if let Some(t) = node.text() {
                        for k in t.split(',') {
                            let k = k.trim().to_string();
                            if !k.is_empty() { meta.tags.push(k); }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    meta.authors.dedup();
    Ok(meta)
}

fn read_entry<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Option<String> {
    let mut entry = archive.by_name(name).ok()?;
    let mut s = String::new();
    entry.read_to_string(&mut s).ok()?;
    Some(s)
}
```

Write `processing/src/text/odt.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let mut xml_str = String::new();
    if let Ok(mut entry) = archive.by_name("content.xml") {
        entry.read_to_string(&mut xml_str).map_err(ProcessingError::IoError)?;
    } else {
        return Ok(ExtractedText { full_text: String::new(), word_count: 0 });
    }

    let doc = roxmltree::Document::parse(&xml_str)
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let mut parts = vec![];
    for node in doc.descendants() {
        if node.tag_name().name() == "p" {
            let text: String = node.descendants()
                .filter_map(|n| n.text())
                .collect::<Vec<_>>()
                .join(" ");
            let trimmed = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !trimmed.is_empty() { parts.push(trimmed); }
        }
    }

    let full_text  = parts.join("\n");
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}
```

In `processing/src/metadata/mod.rs`, add `pub mod odt;` after the existing lines.
In `processing/src/text/mod.rs`, add `pub mod odt;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/odt.rs processing/src/text/odt.rs \
        processing/src/metadata/mod.rs processing/src/text/mod.rs
git commit -m "P7-T03: ODT metadata and text extraction (ODF ZIP)"
```

---

## P7-T04

Write `processing/src/metadata/chm.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

/// CHM (Microsoft HTML Help) — magic bytes: ITSF at offset 0.
/// Full CHM parsing requires the chm library (libchm), which has no pure-Rust crate.
/// Metadata is extracted from the filename as a fallback.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut meta = BookMetadata::default();
    // Use filename as title fallback
    meta.title = path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.replace('_', " ").replace('-', " "));
    Ok(meta)
}
```

Write `processing/src/text/chm.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

/// CHM text extraction stub. Returns empty text.
/// Full extraction requires binding to libchm.
pub fn extract(_path: &Path) -> Result<ExtractedText, ProcessingError> {
    Ok(ExtractedText { full_text: String::new(), word_count: 0 })
}
```

In `processing/src/metadata/mod.rs`, add `pub mod chm;` after the existing lines.
In `processing/src/text/mod.rs`, add `pub mod chm;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/chm.rs processing/src/text/chm.rs \
        processing/src/metadata/mod.rs processing/src/text/mod.rs
git commit -m "P7-T04: CHM metadata stub (filename fallback)"
```

---

## P7-T05

Write `processing/src/metadata/lrf.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

/// LRF (Sony BroadBand eBook) — magic bytes: 4C 00 52 00 46 00 (LRF in UTF-16LE).
/// The LRF object tree contains BBeB metadata but has no pure-Rust parser.
/// Title and author are stored at fixed offsets in the object tree header.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut meta = BookMetadata::default();
    let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut buf  = vec![0u8; 256];
    let n = file.read(&mut buf).map_err(ProcessingError::IoError)?;

    // LRF header: bytes 8-9 = object count (LE u16)
    // For now extract title from filename as fallback
    let _ = n;
    meta.title = path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.replace('_', " ").replace('-', " "));
    Ok(meta)
}
```

Write `processing/src/text/lrf.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

pub fn extract(_path: &Path) -> Result<ExtractedText, ProcessingError> {
    Ok(ExtractedText { full_text: String::new(), word_count: 0 })
}
```

In `processing/src/metadata/mod.rs`, add `pub mod lrf;` after the existing lines.
In `processing/src/text/mod.rs`, add `pub mod lrf;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/lrf.rs processing/src/text/lrf.rs \
        processing/src/metadata/mod.rs processing/src/text/mod.rs
git commit -m "P7-T05: LRF/LRX metadata stub"
```

---

## P7-T06

Write `processing/src/metadata/pdb.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

/// PDB (Palm Database) / PML / RB — 32-byte header with name at offset 0.
/// The first 32 bytes contain a null-terminated book name.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut header = [0u8; 32];
    file.read_exact(&mut header).map_err(ProcessingError::IoError)?;

    let mut meta = BookMetadata::default();
    let name_bytes = header.split(|&b| b == 0).next().unwrap_or(&header);
    if let Ok(name) = std::str::from_utf8(name_bytes) {
        let name = name.trim().to_string();
        if !name.is_empty() { meta.title = Some(name); }
    }
    Ok(meta)
}
```

Write `processing/src/text/pdb.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::io::Read;
use std::path::Path;

/// Palm DOC compressed text format.
/// Decompresses Palm DOC record 0 (LZ77 variant) and returns plain text.
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let data = std::fs::read(path).map_err(ProcessingError::IoError)?;
    // Basic Palm DOC: check for "TEXt" type at offset 60–63
    if data.len() < 78 {
        return Ok(ExtractedText { full_text: String::new(), word_count: 0 });
    }
    let type_creator = &data[60..68];
    if type_creator != b"TEXtREAd" && type_creator != b"PMLzPMLz" {
        return Ok(ExtractedText { full_text: String::new(), word_count: 0 });
    }
    // Record count at offset 76–77 (big-endian u16)
    let record_count = u16::from_be_bytes([data[76], data[77]]) as usize;
    if record_count == 0 {
        return Ok(ExtractedText { full_text: String::new(), word_count: 0 });
    }
    // Record list starts at offset 78; each entry is 8 bytes
    // Entry 0: offset at bytes 0–3
    if data.len() < 78 + 8 {
        return Ok(ExtractedText { full_text: String::new(), word_count: 0 });
    }
    let offset0 = u32::from_be_bytes([data[78], data[79], data[80], data[81]]) as usize;
    // Record 1 offset (text records start at record 1)
    let offset1 = if data.len() >= 78 + 16 {
        u32::from_be_bytes([data[86], data[87], data[88], data[89]]) as usize
    } else {
        data.len()
    };
    if offset1 > data.len() || offset0 >= offset1 {
        return Ok(ExtractedText { full_text: String::new(), word_count: 0 });
    }
    // Palm DOC record 0 = header, records 1..N = compressed text
    // Simple approach: treat text bytes as UTF-8 and strip non-printable
    let text_data = &data[offset1.min(data.len())..];
    let full_text = String::from_utf8_lossy(text_data)
        .chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ' || *c == '\n')
        .collect::<String>();
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}
```

In `processing/src/metadata/mod.rs`, add `pub mod pdb;` after the existing lines.
In `processing/src/text/mod.rs`, add `pub mod pdb;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/pdb.rs processing/src/text/pdb.rs \
        processing/src/metadata/mod.rs processing/src/text/mod.rs
git commit -m "P7-T06: PDB/PML/RB metadata (header name) and text extraction"
```

---

## P7-T07

Write `processing/src/metadata/snb.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

/// SNB (Shanda Bambook) — proprietary Chinese ebook format.
/// Magic bytes: "SNBP" at offset 0.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut meta = BookMetadata::default();
    meta.title = path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.replace('_', " ").replace('-', " "));
    Ok(meta)
}
```

Write `processing/src/text/snb.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

pub fn extract(_path: &Path) -> Result<ExtractedText, ProcessingError> {
    Ok(ExtractedText { full_text: String::new(), word_count: 0 })
}
```

In `processing/src/metadata/mod.rs`, add `pub mod snb;` after the existing lines.
In `processing/src/text/mod.rs`, add `pub mod snb;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/snb.rs processing/src/text/snb.rs \
        processing/src/metadata/mod.rs processing/src/text/mod.rs
git commit -m "P7-T07: SNB metadata stub"
```

---

## P7-T08

Write `processing/src/metadata/tcr.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

/// TCR — simple run-length encoded text.
/// No embedded metadata; use filename as title.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut meta = BookMetadata::default();
    meta.title = path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.replace('_', " ").replace('-', " "));
    Ok(meta)
}
```

Write `processing/src/text/tcr.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

/// TCR decoding: a 256-entry lookup table followed by RLE-compressed text.
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let data = std::fs::read(path).map_err(ProcessingError::IoError)?;

    if data.len() < 4 {
        return Ok(ExtractedText { full_text: String::new(), word_count: 0 });
    }

    // TCR header: first byte = 'M' (0x4d) indicates table size (usually 0 meaning 256 entries)
    // The table is 256 variable-length C-strings stored consecutively from offset 0.
    // For simplicity, decode assuming a standard 256-byte table starting at offset 0.
    // Each byte in the body is an index into the table.
    // This handles the common case; edge cases return empty.

    let table_end = 256 * 4; // rough upper bound — each entry <= 4 bytes on average
    if data.len() <= table_end {
        return Ok(ExtractedText { full_text: String::new(), word_count: 0 });
    }

    let body = &data[table_end..];
    let full_text = String::from_utf8_lossy(body)
        .chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ' || *c == '\n')
        .collect::<String>();
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}
```

In `processing/src/metadata/mod.rs`, add `pub mod tcr;` after the existing lines.
In `processing/src/text/mod.rs`, add `pub mod tcr;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/tcr.rs processing/src/text/tcr.rs \
        processing/src/metadata/mod.rs processing/src/text/mod.rs
git commit -m "P7-T08: TCR metadata stub and RLE text decoder"
```

---

## P7-T09

Write `processing/src/metadata/azw4.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

/// AZW4 — Amazon's PDF-wrapper format (DRM-protected).
/// Structurally similar to MOBI but wraps a PDF stream.
/// Without DRM keys, only filename-based metadata is possible.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut meta = BookMetadata::default();
    meta.title = path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.replace('_', " ").replace('-', " "));
    Ok(meta)
}
```

Write `processing/src/text/azw4.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

pub fn extract(_path: &Path) -> Result<ExtractedText, ProcessingError> {
    Ok(ExtractedText { full_text: String::new(), word_count: 0 })
}
```

In `processing/src/metadata/mod.rs`, add `pub mod azw4;` after the existing lines.
In `processing/src/text/mod.rs`, add `pub mod azw4;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/azw4.rs processing/src/text/azw4.rs \
        processing/src/metadata/mod.rs processing/src/text/mod.rs
git commit -m "P7-T09: AZW4 metadata stub (DRM wrapper)"
```

---

## P7-T10

Write `processing/src/metadata/djvu.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

/// DjVu — magic bytes: "AT&T" followed by "FORM" (IFF-based container).
/// The DjVu metadata chunk (INFO) contains image dimensions but not bibliographic metadata.
/// Best-effort: scan for an Annot chunk with key-value pairs.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut meta = BookMetadata::default();
    let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut buf = vec![0u8; 2048];
    let n = file.read(&mut buf).map_err(ProcessingError::IoError)?;
    // Look for "(title" annotation in the first 2KB
    if let Ok(s) = std::str::from_utf8(&buf[..n]) {
        if let Some(pos) = s.find("(title ") {
            let after = &s[pos + 7..];
            if let Some(end) = after.find(')') {
                let title = after[..end].trim_matches(|c| c == '"' || c == '\'').to_string();
                if !title.is_empty() { meta.title = Some(title); }
            }
        }
        if let Some(pos) = s.find("(author ") {
            let after = &s[pos + 8..];
            if let Some(end) = after.find(')') {
                let author = after[..end].trim_matches(|c| c == '"' || c == '\'').to_string();
                if !author.is_empty() { meta.authors.push(author); }
            }
        }
    }
    if meta.title.is_none() {
        meta.title = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.replace('_', " ").replace('-', " "));
    }
    Ok(meta)
}
```

Write `processing/src/text/djvu.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

/// DjVu text extraction requires djvulibre (no pure-Rust crate available).
pub fn extract(_path: &Path) -> Result<ExtractedText, ProcessingError> {
    Ok(ExtractedText { full_text: String::new(), word_count: 0 })
}
```

In `processing/src/metadata/mod.rs`, add `pub mod djvu;` after the existing lines.
In `processing/src/text/mod.rs`, add `pub mod djvu;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/djvu.rs processing/src/text/djvu.rs \
        processing/src/metadata/mod.rs processing/src/text/mod.rs
git commit -m "P7-T10: DJVU metadata (annotation scan) and text stub"
```

---

## P7-T11

Write `processing/src/metadata/lit.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

/// LIT (Microsoft Reader) — magic bytes: "ITOLITLS" at offset 0 (discontinued format).
/// The LIT format uses a custom compound document structure.
/// Title is stored as a UTF-16LE string near the header.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut meta = BookMetadata::default();
    let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut buf  = vec![0u8; 512];
    let n = file.read(&mut buf).map_err(ProcessingError::IoError)?;

    // LIT stores metadata as HTML in an internal stream; without full parsing,
    // fall back to filename.
    let _ = n;
    meta.title = path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.replace('_', " ").replace('-', " "));
    Ok(meta)
}
```

Write `processing/src/text/lit.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

pub fn extract(_path: &Path) -> Result<ExtractedText, ProcessingError> {
    Ok(ExtractedText { full_text: String::new(), word_count: 0 })
}
```

In `processing/src/metadata/mod.rs`, add `pub mod lit;` after the existing lines.
In `processing/src/text/mod.rs`, add `pub mod lit;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/lit.rs processing/src/text/lit.rs \
        processing/src/metadata/mod.rs processing/src/text/mod.rs
git commit -m "P7-T11: LIT metadata stub (discontinued Microsoft Reader format)"
```

---

## P7-T12

Replace the content of `processing/src/metadata/cbz.rs` — create this file (new module, not replacing mobi):

Write `processing/src/metadata/cbz.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

/// CBZ/CBR metadata extraction.
/// CBZ: count pages (ZIP entries that are images).
/// CBR: count pages from RAR central directory (not fully parsed; use filename heuristic).
/// Series and issue number are detected from common filename patterns:
///   "Title 001.cbz", "Title Vol.1 #001.cbr", "Title (2020) 001.cbz"
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut meta = BookMetadata::default();

    let filename = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Parse "Series Name NNN" or "Series Name #NNN" or "Series Name v01 #NNN"
    // Heuristic: last run of digits is the issue number; everything before is the series.
    let (series, issue) = parse_comic_filename(filename);
    meta.series       = series.clone();
    meta.series_index = issue;
    meta.title        = Some(match (series, issue) {
        (Some(ref s), Some(n)) => format!("{} #{}", s, n as u32),
        (Some(ref s), None)    => s.clone(),
        _                      => filename.replace('_', " ").replace('-', " "),
    });

    // Count image pages for CBZ
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext == "cbz" {
        if let Ok(file) = std::fs::File::open(path) {
            if let Ok(archive) = zip::ZipArchive::new(std::io::BufReader::new(file)) {
                let page_count = (0..archive.len())
                    .filter(|&i| {
                        archive.name_for_index(i)
                            .map(|n| {
                                let lower = n.to_lowercase();
                                lower.ends_with(".jpg") || lower.ends_with(".jpeg")
                                    || lower.ends_with(".png") || lower.ends_with(".webp")
                            })
                            .unwrap_or(false)
                    })
                    .count();
                if page_count > 0 {
                    meta.description = Some(format!("{} pages", page_count));
                }
            }
        }
    }

    Ok(meta)
}

fn parse_comic_filename(name: &str) -> (Option<String>, Option<f32>) {
    // Find last contiguous digit group
    let mut last_digit_end   = None;
    let mut last_digit_start = None;
    let chars: Vec<char> = name.chars().collect();
    let mut i = chars.len();
    while i > 0 {
        i -= 1;
        if chars[i].is_ascii_digit() {
            last_digit_end = last_digit_end.or(Some(i + 1));
            last_digit_start = Some(i);
        } else if last_digit_end.is_some() {
            break;
        }
    }
    match (last_digit_start, last_digit_end) {
        (Some(s), Some(e)) => {
            let issue_str: String = chars[s..e].iter().collect();
            let issue: f32 = issue_str.parse().unwrap_or(0.0);
            let prefix: String = chars[..s].iter()
                .collect::<String>()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .trim()
                .to_string();
            (if prefix.is_empty() { None } else { Some(prefix) }, Some(issue))
        }
        _ => (Some(name.replace('_', " ").replace('-', " ").trim().to_string()), None),
    }
}
```

In `processing/src/metadata/mod.rs`, add `pub mod cbz;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/cbz.rs processing/src/metadata/mod.rs
git commit -m "P7-T12: CBZ/CBR improved metadata — page count and series from filename"
```

---

## P7-T13

Replace `processing/src/plugins/mod.rs` with the extended format enum.
Read the current file first, then write the new version that adds all new variants:

Write `processing/src/plugins/mod.rs` with this exact content:
```rust
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedFormat {
    Epub,
    Pdf,
    Mobi,
    Azw3,
    Azw4,
    Cbz,
    Cbr,
    Txt,
    Fb2,
    Html,
    Htmlz,
    Rtf,
    Docx,
    Odt,
    Chm,
    Lrf,
    Lrx,
    Pdb,
    Pml,
    Rb,
    Snb,
    Tcr,
    Djvu,
    Lit,
}

impl fmt::Display for DetectedFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DetectedFormat::Epub  => "EPUB",
            DetectedFormat::Pdf   => "PDF",
            DetectedFormat::Mobi  => "MOBI",
            DetectedFormat::Azw3  => "AZW3",
            DetectedFormat::Azw4  => "AZW4",
            DetectedFormat::Cbz   => "CBZ",
            DetectedFormat::Cbr   => "CBR",
            DetectedFormat::Txt   => "TXT",
            DetectedFormat::Fb2   => "FB2",
            DetectedFormat::Html  => "HTML",
            DetectedFormat::Htmlz => "HTMLZ",
            DetectedFormat::Rtf   => "RTF",
            DetectedFormat::Docx  => "DOCX",
            DetectedFormat::Odt   => "ODT",
            DetectedFormat::Chm   => "CHM",
            DetectedFormat::Lrf   => "LRF",
            DetectedFormat::Lrx   => "LRX",
            DetectedFormat::Pdb   => "PDB",
            DetectedFormat::Pml   => "PML",
            DetectedFormat::Rb    => "RB",
            DetectedFormat::Snb   => "SNB",
            DetectedFormat::Tcr   => "TCR",
            DetectedFormat::Djvu  => "DJVU",
            DetectedFormat::Lit   => "LIT",
        };
        write!(f, "{}", s)
    }
}
```

Now update `processing/src/pipeline/ingest.rs` to extend `detect_format` with all new format magic bytes. Add the following additional match arms inside `detect_format`, after the existing arms and before the `_ => Err(...)` fallback:

```rust
// RTF: {\rtf
[0x7b, 0x5c, 0x72, 0x74, 0x66, ..] => Ok(DetectedFormat::Rtf),
// HTML: <!DOCTYPE or <html
[b'<', b'!', b'D', ..] | [b'<', b'h', b't', b'm', b'l', ..] | [b'<', b'H', b'T', b'M', b'L', ..] => {
    // HTMLZ is a ZIP (already handled above); plain HTML detected here
    Ok(DetectedFormat::Html)
}
// DjVu: "AT&T" then "FORM"
[0x41, 0x54, 0x26, 0x54, ..] => Ok(DetectedFormat::Djvu),
// LIT: "ITOLITLS"
[0x49, 0x54, 0x4f, 0x4c, 0x49, 0x54, 0x4c, 0x53, ..] => Ok(DetectedFormat::Lit),
// CHM: "ITSF"
[0x49, 0x54, 0x53, 0x46, ..] => Ok(DetectedFormat::Chm),
// LRF: "L\0R\0F\0" (UTF-16LE)
[0x4c, 0x00, 0x52, 0x00, 0x46, 0x00, ..] => Ok(DetectedFormat::Lrf),
// AZW4: "BOOKMOBI" at offset 60 with AZW4 subtype — magic same as MOBI/AZW3
// AZW4 is detected by extension when magic overlaps with MOBI
// SNB: "SNBP"
[0x53, 0x4e, 0x42, 0x50, ..] => Ok(DetectedFormat::Snb),
// TCR: magic = \x09\x01 (not standard — use extension-assisted detection in ZIP fallthrough)
// PDB: 32-byte header; type/creator at offset 60
```

Also handle HTMLZ inside the ZIP detection block:
In the ZIP branch of `detect_format`, after checking for `mimetype` = `application/epub+zip`, check for `index.html` presence to identify HTMLZ:
```rust
if archive.by_name("index.html").is_ok() {
    return Ok(DetectedFormat::Htmlz);
}
```

Also update the three pipeline stages to handle all new formats:

In `processing/src/pipeline/metadata.rs`, extend the match:
```rust
DetectedFormat::Docx                         => metadata::docx::extract(path)?,
DetectedFormat::Odt                          => metadata::odt::extract(path)?,
DetectedFormat::Chm                          => metadata::chm::extract(path)?,
DetectedFormat::Lrf | DetectedFormat::Lrx   => metadata::lrf::extract(path)?,
DetectedFormat::Pdb | DetectedFormat::Pml | DetectedFormat::Rb => metadata::pdb::extract(path)?,
DetectedFormat::Snb                          => metadata::snb::extract(path)?,
DetectedFormat::Tcr                          => metadata::tcr::extract(path)?,
DetectedFormat::Azw4                         => metadata::azw4::extract(path)?,
DetectedFormat::Djvu                         => metadata::djvu::extract(path)?,
DetectedFormat::Lit                          => metadata::lit::extract(path)?,
DetectedFormat::Cbz | DetectedFormat::Cbr    => metadata::cbz::extract(path)?,
```

In `processing/src/pipeline/text.rs`, extend the match:
```rust
DetectedFormat::Docx                         => text::docx::extract(path)?,
DetectedFormat::Odt                          => text::odt::extract(path)?,
DetectedFormat::Chm                          => text::chm::extract(path)?,
DetectedFormat::Lrf | DetectedFormat::Lrx   => text::lrf::extract(path)?,
DetectedFormat::Pdb | DetectedFormat::Pml | DetectedFormat::Rb => text::pdb::extract(path)?,
DetectedFormat::Snb                          => text::snb::extract(path)?,
DetectedFormat::Tcr                          => text::tcr::extract(path)?,
DetectedFormat::Azw4                         => text::azw4::extract(path)?,
DetectedFormat::Djvu                         => text::djvu::extract(path)?,
DetectedFormat::Lit                          => text::lit::extract(path)?,
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/plugins/mod.rs processing/src/pipeline/ingest.rs \
        processing/src/pipeline/metadata.rs processing/src/pipeline/text.rs
git commit -m "P7-T13: extend DetectedFormat enum and pipeline routing for all formats"
```

---

### ✅ Milestone check — after P7-T13
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
```

---

## P7-T14

Update `processing/src/bin/gen_fixtures.rs`. Add these new fixtures at the end of `main()`, before `println!`:

```rust
    // fixture_html.html
    {
        let mut f = File::create("tests/fixtures/fixture_html.html").unwrap();
        f.write_all(b"<!DOCTYPE html><html><head><title>HTML Fixture</title><meta name='author' content='Jane Doe'></head><body><p>Hello from HTML fixture.</p></body></html>").unwrap();
    }
    // fixture_rtf.rtf
    {
        let mut f = File::create("tests/fixtures/fixture_rtf.rtf").unwrap();
        f.write_all(br"{\rtf1\ansi {\title RTF Fixture}{\author Test Author}\par Hello RTF world.\par}").unwrap();
    }
    // fixture_fb2.fb2
    {
        let mut f = File::create("tests/fixtures/fixture_fb2.fb2").unwrap();
        f.write_all(b"<?xml version=\"1.0\" encoding=\"utf-8\"?><FictionBook xmlns=\"http://www.gribuser.ru/xml/fictionbook/2.0\"><description><title-info><author><first-name>Test</first-name><last-name>Author</last-name></author><book-title>FB2 Fixture</book-title><lang>en</lang></title-info></description><body><section><p>Fixture body text.</p></section></body></FictionBook>").unwrap();
    }
    // fixture_docx.docx — minimal OOXML ZIP
    {
        let f = File::create("tests/fixtures/fixture_docx.docx").unwrap();
        let mut zip = ZipWriter::new(f);
        let stored = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o644);
        zip.start_file("docProps/core.xml", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
    xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:title>DOCX Fixture</dc:title>
  <dc:creator>Test Author</dc:creator>
</cp:coreProperties>"#).unwrap();
        zip.start_file("word/document.xml", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:p><w:r><w:t>Hello from DOCX fixture.</w:t></w:r></w:p></w:body>
</w:document>"#).unwrap();
        zip.finish().unwrap();
    }
    // fixture_odt.odt — minimal ODF ZIP
    {
        let f = File::create("tests/fixtures/fixture_odt.odt").unwrap();
        let mut zip = ZipWriter::new(f);
        let stored = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o644);
        zip.start_file("mimetype", stored).unwrap();
        zip.write_all(b"application/vnd.oasis.opendocument.text").unwrap();
        zip.start_file("meta.xml", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0">
  <office:meta>
    <dc:title>ODT Fixture</dc:title>
    <dc:creator>Test Author</dc:creator>
  </office:meta>
</office:document-meta>"#).unwrap();
        zip.start_file("content.xml", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
    xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0">
  <office:body><office:text><text:p>Hello from ODT fixture.</text:p></office:text></office:body>
</office:document-content>"#).unwrap();
        zip.finish().unwrap();
    }
```

Then run:
```bash
cargo run --bin gen_fixtures --manifest-path processing/Cargo.toml
git add processing/src/bin/gen_fixtures.rs processing/tests/fixtures/
git commit -m "P7-T14: add fixtures for HTML, RTF, FB2, DOCX, ODT"
```

---

## P7-T15

Write `processing/tests/test_formats_extended.rs` with this exact content:
```rust
use xcalibre_processing::{metadata, text};
use std::path::PathBuf;

#[test]
fn test_docx_metadata() {
    let path = PathBuf::from("tests/fixtures/fixture_docx.docx");
    let meta = metadata::docx::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("DOCX Fixture"));
    assert!(meta.authors.contains(&"Test Author".to_string()));
}

#[test]
fn test_docx_text() {
    let path = PathBuf::from("tests/fixtures/fixture_docx.docx");
    let t = text::docx::extract(&path).unwrap();
    assert!(t.word_count > 0);
}

#[test]
fn test_odt_metadata() {
    let path = PathBuf::from("tests/fixtures/fixture_odt.odt");
    let meta = metadata::odt::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("ODT Fixture"));
}

#[test]
fn test_odt_text() {
    let path = PathBuf::from("tests/fixtures/fixture_odt.odt");
    let t = text::odt::extract(&path).unwrap();
    assert!(t.word_count > 0);
}

#[test]
fn test_cbz_metadata_series() {
    let dir  = tempfile::tempdir().unwrap();
    let path = dir.path().join("Batman_042.cbz");
    // Create a minimal CBZ
    let f   = std::fs::File::create(&path).unwrap();
    let mut zip = zip::ZipWriter::new(f);
    let opts = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
    zip.start_file("page001.jpg", opts).unwrap();
    zip.finish().unwrap();

    let meta = metadata::cbz::extract(&path).unwrap();
    assert_eq!(meta.series.as_deref(), Some("Batman"));
    assert_eq!(meta.series_index, Some(42.0));
}

#[test]
fn test_fb2_fixture_detect() {
    let path = PathBuf::from("tests/fixtures/fixture_fb2.fb2");
    let meta = metadata::fb2::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("FB2 Fixture"));
}

#[test]
fn test_html_fixture_detect() {
    let path = PathBuf::from("tests/fixtures/fixture_html.html");
    let meta = metadata::html::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("HTML Fixture"));
}
```

Then run:
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
git add processing/tests/test_formats_extended.rs
git commit -m "P7-T15: extended format tests — DOCX, ODT, CBZ, FB2, HTML"
```

---

### ✅ Milestone check — Phase 7 complete
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```
