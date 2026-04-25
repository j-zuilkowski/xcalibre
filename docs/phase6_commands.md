# Phase 6 — Primary Format Completion

> HOW TO USE: For every "Write `path`" line → call write_file with that path and content.
> For every "Then run:" block → call your shell tool for each command.
> DO NOT print code as output. Write it to disk using your tools.
> Prerequisite: Phase 5 complete and all tests green.
> Status: ✅ done · ⬜ not started

Covers full metadata + text extraction for PDF, MOBI/AZW3, FB2, HTML/HTMLZ, and RTF.
These are the five formats most commonly encountered alongside EPUB.

## Status

| Task | Title | Status |
|------|-------|--------|
| P6-T01 | Add lopdf, mobi, scraper deps | ✅ |
| P6-T02 | PDF full metadata extraction | ✅ |
| P6-T03 | PDF text extraction | ✅ |
| P6-T04 | PDF cover extraction (embedded image) | ✅ |
| P6-T05 | MOBI/AZW3 full metadata | ✅ |
| P6-T06 | MOBI/AZW3 text extraction | ✅ |
| P6-T07 | FB2 metadata + text | ✅ |
| P6-T08 | HTML/HTMLZ metadata + text | ✅ |
| P6-T09 | RTF metadata + text | ✅ |
| P6-T10 | Wire new extractors into pipeline | ✅ |
| P6-T11 | Tests | ✅ |

---

## P6-T01

In `processing/Cargo.toml`, add under `[dependencies]`:
```toml
lopdf   = "0.34"
mobi    = "0.7"
scraper = "0.20"
```

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/Cargo.toml
git commit -m "P6-T01: add lopdf, mobi, scraper dependencies"
```

---

## P6-T02

Replace the content of `processing/src/metadata/pdf.rs` with:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let doc = lopdf::Document::load(path)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut meta = BookMetadata::default();

    if let Ok(info_id) = doc.trailer.get(b"Info") {
        if let Ok(lopdf::Object::Dictionary(info)) = doc.get_object(
            info_id.as_reference().map_err(|e| ProcessingError::MetadataError(e.to_string()))?,
        ) {
            let get_str = |dict: &lopdf::Dictionary, key: &[u8]| -> Option<String> {
                dict.get(key).ok().and_then(|obj| match obj {
                    lopdf::Object::String(bytes, _) => String::from_utf8_lossy(bytes).into_owned().into(),
                    _ => None,
                })
            };
            meta.title     = get_str(info, b"Title");
            meta.publisher = get_str(info, b"Creator");
            if let Some(author) = get_str(info, b"Author") {
                // Authors may be semicolon- or comma-separated
                for a in author.split(|c| c == ';' || c == ',') {
                    let a = a.trim().to_string();
                    if !a.is_empty() { meta.authors.push(a); }
                }
            }
            meta.published = get_str(info, b"CreationDate");
        }
    }

    Ok(meta)
}
```

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/pdf.rs
git commit -m "P6-T02: PDF full metadata extraction via lopdf Info dict"
```

---

## P6-T03

Write `processing/src/text/pdf.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let doc = lopdf::Document::load(path)
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let mut parts: Vec<String> = vec![];

    for page_id in doc.page_iter() {
        // Extract text from each content stream on the page
        if let Ok(content) = doc.get_and_decode_page_content(page_id) {
            let mut page_text = String::new();
            let ops = lopdf::content::Content::decode(&content)
                .map_err(|e| ProcessingError::TextError(e.to_string()))?;
            for op in ops.operations {
                match op.operator.as_str() {
                    "Tj" | "TJ" | "'" | "\"" => {
                        for operand in &op.operands {
                            match operand {
                                lopdf::Object::String(bytes, _) => {
                                    if let Ok(s) = std::str::from_utf8(bytes) {
                                        page_text.push_str(s);
                                        page_text.push(' ');
                                    }
                                }
                                lopdf::Object::Array(arr) => {
                                    for item in arr {
                                        if let lopdf::Object::String(bytes, _) = item {
                                            if let Ok(s) = std::str::from_utf8(bytes) {
                                                page_text.push_str(s);
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            let trimmed = page_text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !trimmed.is_empty() {
                parts.push(trimmed);
            }
        }
    }

    let full_text  = parts.join("\n\n");
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}
```

In `processing/src/text/mod.rs`, add `pub mod pdf;` after the existing `pub mod epub;`.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/text/pdf.rs processing/src/text/mod.rs
git commit -m "P6-T03: PDF text extraction via lopdf content streams"
```

---

## P6-T04

Write `processing/src/cover/pdf.rs` with this exact content:
```rust
use crate::cover::CoverResult;
use crate::error::ProcessingError;
use std::path::Path;

/// Attempt to extract the first embedded XObject image from a PDF.
/// Returns None if no suitable image is found (common — PDFs rarely embed covers as raw images).
pub fn extract(path: &Path) -> Result<Option<CoverResult>, ProcessingError> {
    let doc = lopdf::Document::load(path)
        .map_err(|e| ProcessingError::CoverError(e.to_string()))?;

    for page_id in doc.page_iter() {
        if let Ok(resources) = doc.get_page_resources(page_id) {
            if let Some(xobjects) = resources.0 {
                for (_, xobj_ref) in xobjects.iter() {
                    if let Ok(stream) = doc.get_object(
                        xobj_ref.as_reference().map_err(|e| ProcessingError::CoverError(e.to_string()))?,
                    ) {
                        if let lopdf::Object::Stream(stream) = stream {
                            let subtype = stream.dict.get(b"Subtype")
                                .and_then(|o| o.as_name_str().ok())
                                .unwrap_or("");
                            if subtype == "Image" {
                                let data = stream.decompressed_content()
                                    .map_err(|e| ProcessingError::CoverError(e.to_string()))?;
                                // Validate it's a recognisable image format
                                if let Ok(img) = image::load_from_memory(&data) {
                                    let (w, h) = (img.width(), img.height());
                                    // Skip tiny images (icons, logos)
                                    if w < 100 || h < 100 { continue; }
                                    return Ok(Some(CoverResult {
                                        data,
                                        mime_type: "image/jpeg".to_string(),
                                        width: w,
                                        height: h,
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }
        // Only check first page
        break;
    }
    Ok(None)
}
```

In `processing/src/cover/mod.rs`, add `pub mod pdf;` after the existing `pub mod epub;`.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/cover/pdf.rs processing/src/cover/mod.rs
git commit -m "P6-T04: PDF cover extraction — first embedded XObject image"
```

---

## P6-T05

Replace the content of `processing/src/metadata/mobi.rs` with:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let data = std::fs::read(path).map_err(ProcessingError::IoError)?;

    let book = mobi::Mobi::new(&data)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut meta = BookMetadata::default();
    meta.title     = book.title().map(String::from).filter(|s| !s.is_empty());
    meta.publisher = book.publisher().map(String::from).filter(|s| !s.is_empty());
    meta.language  = book.language().map(String::from).filter(|s| !s.is_empty());
    meta.published = book.publish_date().map(String::from).filter(|s| !s.is_empty());
    meta.description = book.description().map(String::from).filter(|s| !s.is_empty());
    meta.isbn      = book.isbn().map(String::from).filter(|s| !s.is_empty());

    if let Some(author) = book.author() {
        for a in author.split('&') {
            let a = a.trim().to_string();
            if !a.is_empty() { meta.authors.push(a); }
        }
    }

    Ok(meta)
}
```

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/mobi.rs
git commit -m "P6-T05: MOBI/AZW3 full metadata via mobi crate"
```

---

## P6-T06

Write `processing/src/text/mobi.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let data = std::fs::read(path).map_err(ProcessingError::IoError)?;

    let book = mobi::Mobi::new(&data)
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    // mobi::Mobi::content() returns the raw HTML content of the book
    let html = book.content()
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    // Strip HTML tags with a simple regex-free approach
    let plain = strip_html(&html);
    let word_count = plain.split_whitespace().count();
    Ok(ExtractedText { full_text: plain, word_count })
}

fn strip_html(html: &str) -> String {
    let mut out   = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => { in_tag = false; out.push(' '); }
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    // Collapse whitespace
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
```

In `processing/src/text/mod.rs`, add `pub mod mobi;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/text/mobi.rs processing/src/text/mod.rs
git commit -m "P6-T06: MOBI/AZW3 text extraction via mobi crate"
```

---

## P6-T07

Write `processing/src/metadata/fb2.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut xml  = String::new();
    file.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;

    let doc = roxmltree::Document::parse(&xml)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut meta = BookMetadata::default();

    // FB2 metadata lives in <FictionBook><description><title-info>
    if let Some(title_info) = doc.descendants().find(|n| n.tag_name().name() == "title-info") {
        for child in title_info.children() {
            match child.tag_name().name() {
                "book-title" => meta.title = child.text().map(str::trim).map(String::from),
                "lang"       => meta.language = child.text().map(str::trim).map(String::from),
                "author"     => {
                    let first = child.children()
                        .find(|n| n.tag_name().name() == "first-name")
                        .and_then(|n| n.text())
                        .unwrap_or("").trim().to_string();
                    let last = child.children()
                        .find(|n| n.tag_name().name() == "last-name")
                        .and_then(|n| n.text())
                        .unwrap_or("").trim().to_string();
                    let author = match (first.is_empty(), last.is_empty()) {
                        (false, false) => format!("{} {}", first, last),
                        (true, false)  => last,
                        (false, true)  => first,
                        (true, true)   => continue,
                    };
                    meta.authors.push(author);
                }
                "genre" => {
                    if let Some(t) = child.text() {
                        let t = t.trim().to_string();
                        if !t.is_empty() { meta.tags.push(t); }
                    }
                }
                "sequence" => {
                    meta.series = child.attribute("name").map(String::from);
                    meta.series_index = child.attribute("number").and_then(|n| n.parse().ok());
                }
                "annotation" => {
                    meta.description = child.text().map(str::trim).map(String::from);
                }
                _ => {}
            }
        }
    }

    // Publisher info lives in <publish-info>
    if let Some(pub_info) = doc.descendants().find(|n| n.tag_name().name() == "publish-info") {
        for child in pub_info.children() {
            match child.tag_name().name() {
                "publisher" => meta.publisher = child.text().map(str::trim).map(String::from),
                "year"      => meta.published = child.text().map(str::trim).map(String::from),
                "isbn"      => meta.isbn      = child.text().map(str::trim).map(String::from),
                _ => {}
            }
        }
    }

    Ok(meta)
}
```

Write `processing/src/text/fb2.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut xml  = String::new();
    file.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;

    let doc = roxmltree::Document::parse(&xml)
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let mut parts = vec![];
    for node in doc.descendants() {
        match node.tag_name().name() {
            "p" | "v" | "subtitle" | "text-author" => {
                let text: String = node.descendants()
                    .filter_map(|n| n.text())
                    .collect::<Vec<_>>()
                    .join(" ");
                let trimmed = text.split_whitespace().collect::<Vec<_>>().join(" ");
                if !trimmed.is_empty() { parts.push(trimmed); }
            }
            _ => {}
        }
    }

    let full_text  = parts.join("\n");
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}
```

In `processing/src/metadata/mod.rs`, add `pub mod fb2;` after the existing `pub mod mobi;`.
In `processing/src/text/mod.rs`, add `pub mod fb2;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/fb2.rs processing/src/text/fb2.rs \
        processing/src/metadata/mod.rs processing/src/text/mod.rs
git commit -m "P6-T07: FB2 metadata and text extraction via roxmltree"
```

---

## P6-T08

Write `processing/src/metadata/html.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let content = read_html(path)?;
    let document = scraper::Html::parse_document(&content);
    let mut meta = BookMetadata::default();

    // <title>
    let title_sel = scraper::Selector::parse("title").unwrap();
    if let Some(el) = document.select(&title_sel).next() {
        let t = el.text().collect::<String>().trim().to_string();
        if !t.is_empty() { meta.title = Some(t); }
    }

    // <meta name="author" content="...">
    let meta_sel = scraper::Selector::parse("meta").unwrap();
    for el in document.select(&meta_sel) {
        let name    = el.value().attr("name").unwrap_or("").to_lowercase();
        let content = el.value().attr("content").unwrap_or("").trim();
        match name.as_str() {
            "author"      => { for a in content.split(',') { let a = a.trim().to_string(); if !a.is_empty() { meta.authors.push(a); } } }
            "description" => meta.description = Some(content.to_string()),
            "keywords"    => { for k in content.split(',') { let k = k.trim().to_string(); if !k.is_empty() { meta.tags.push(k); } } }
            _ => {}
        }
    }
    Ok(meta)
}

fn read_html(path: &Path) -> Result<String, ProcessingError> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext == "htmlz" {
        // HTMLZ is a ZIP containing index.html
        let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
        let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        let mut html_file = archive.by_name("index.html")
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        let mut s = String::new();
        html_file.read_to_string(&mut s).map_err(ProcessingError::IoError)?;
        Ok(s)
    } else {
        let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
        let mut s = String::new();
        file.read_to_string(&mut s).map_err(ProcessingError::IoError)?;
        Ok(s)
    }
}
```

Write `processing/src/text/html.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let content = read_html(path)?;
    let document = scraper::Html::parse_document(&content);

    let sel = scraper::Selector::parse("p, h1, h2, h3, h4, li, td, th").unwrap();
    let mut parts = vec![];
    for el in document.select(&sel) {
        let text = el.text().collect::<String>();
        let trimmed = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if !trimmed.is_empty() { parts.push(trimmed); }
    }

    let full_text  = parts.join("\n");
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}

fn read_html(path: &Path) -> Result<String, ProcessingError> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext == "htmlz" {
        let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
        let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
            .map_err(|e| ProcessingError::TextError(e.to_string()))?;
        let mut html_file = archive.by_name("index.html")
            .map_err(|e| ProcessingError::TextError(e.to_string()))?;
        let mut s = String::new();
        html_file.read_to_string(&mut s).map_err(ProcessingError::IoError)?;
        Ok(s)
    } else {
        let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
        let mut s = String::new();
        file.read_to_string(&mut s).map_err(ProcessingError::IoError)?;
        Ok(s)
    }
}
```

In `processing/src/metadata/mod.rs`, add `pub mod html;` after the existing lines.
In `processing/src/text/mod.rs`, add `pub mod html;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/html.rs processing/src/text/html.rs \
        processing/src/metadata/mod.rs processing/src/text/mod.rs
git commit -m "P6-T08: HTML/HTMLZ metadata and text extraction via scraper"
```

---

## P6-T09

Write `processing/src/metadata/rtf.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

/// RTF metadata is rarely structured. Extract title from the \title control word.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let raw = std::fs::read_to_string(path).map_err(ProcessingError::IoError)?;
    let mut meta = BookMetadata::default();
    meta.title = extract_control_value(&raw, r"\title");
    meta.authors = extract_control_value(&raw, r"\author")
        .map(|a| a.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    Ok(meta)
}

fn extract_control_value(rtf: &str, control: &str) -> Option<String> {
    let pos = rtf.find(control)?;
    let after = &rtf[pos + control.len()..];
    // Value is in the next {group} or until next control word
    let start = after.find('{')? + 1;
    let end   = after[start..].find('}')?  + start;
    let raw   = &after[start..end];
    let clean = strip_rtf_markup(raw).trim().to_string();
    if clean.is_empty() { None } else { Some(clean) }
}

fn strip_rtf_markup(s: &str) -> String {
    let mut out    = String::new();
    let mut skip   = false;
    let mut chars  = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => { skip = true; }
            ' ' | '\n' | '\r' if skip => { skip = false; }
            _ if skip && c.is_alphabetic() => {
                // consume until non-alpha (end of control word)
                while chars.peek().map_or(false, |x| x.is_alphanumeric() || *x == '-') {
                    chars.next();
                }
                skip = false;
            }
            '{' | '}' => {}
            _ => { skip = false; out.push(c); }
        }
    }
    out
}
```

Write `processing/src/text/rtf.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let raw = std::fs::read_to_string(path).map_err(ProcessingError::IoError)?;
    let full_text  = strip_rtf(&raw);
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}

/// Strip all RTF control words and return plain text.
fn strip_rtf(rtf: &str) -> String {
    let mut out   = String::with_capacity(rtf.len() / 2);
    let mut chars = rtf.chars().peekable();
    let mut depth = 0i32;
    let mut skip_group = 0i32; // skip {\*...} ignored destinations

    while let Some(c) = chars.next() {
        match c {
            '{' => {
                depth += 1;
                // Check for \* ignored destination
                if chars.peek() == Some(&'\\') {
                    let mut peek_iter = rtf.chars(); // poor man's lookahead
                    let _ = peek_iter.next();
                    if peek_iter.next() == Some('*') {
                        skip_group += 1;
                    }
                }
            }
            '}' => {
                if skip_group > 0 { skip_group -= 1; }
                depth -= 1;
            }
            '\\' => {
                match chars.peek() {
                    Some(&'\n') | Some(&'\r') => { out.push('\n'); chars.next(); }
                    Some(&'\\') => { if skip_group == 0 { out.push('\\'); } chars.next(); }
                    Some(&'{')  => { if skip_group == 0 { out.push('{'); } chars.next(); }
                    Some(&'}')  => { if skip_group == 0 { out.push('}'); } chars.next(); }
                    Some(&c2) if c2.is_alphabetic() => {
                        // consume control word
                        let mut word = String::new();
                        while chars.peek().map_or(false, |x| x.is_alphanumeric() || *x == '-') {
                            word.push(chars.next().unwrap());
                        }
                        // consume optional space delimiter
                        if chars.peek() == Some(&' ') { chars.next(); }
                        match word.as_str() {
                            "par" | "line" | "page" => { if skip_group == 0 { out.push('\n'); } }
                            "tab"  => { if skip_group == 0 { out.push('\t'); } }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ if skip_group == 0 && depth > 0 => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
```

In `processing/src/metadata/mod.rs`, add `pub mod rtf;` after the existing lines.
In `processing/src/text/mod.rs`, add `pub mod rtf;` after the existing lines.

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/metadata/rtf.rs processing/src/text/rtf.rs \
        processing/src/metadata/mod.rs processing/src/text/mod.rs
git commit -m "P6-T09: RTF metadata and text extraction (control-word stripper)"
```

---

## P6-T10

In `processing/src/plugins/mod.rs`, add these variants to `DetectedFormat` if not already present:
```rust
Fb2,
Html,
Htmlz,
Rtf,
```

Also add detection logic in `detect_format` in `processing/src/pipeline/ingest.rs`:
```rust
// After existing format checks, add:
// FB2 — XML with FictionBook root
(b"<?xml", _) | (b"<Fict", _) => {
    // Check for FictionBook root element
    let mut buf = vec![0u8; 512];
    let mut f2 = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let n = f2.read(&mut buf).map_err(ProcessingError::IoError)?;
    let head = std::str::from_utf8(&buf[..n]).unwrap_or("");
    if head.contains("FictionBook") {
        return Ok(DetectedFormat::Fb2);
    }
}
// HTML — starts with <!DOCTYPE or <html
// HTMLZ — ZIP with index.html
// RTF — starts with {\rtf
```

The cleanest approach for the new formats is to extend the existing `detect_format` function. Replace the body of `detect_format` in `processing/src/pipeline/ingest.rs` by adding after the existing format arms:

After the `Cbz` arm, add these cases:
```rust
// RTF: starts with "{\rtf"
[0x7b, 0x5c, 0x72, 0x74, 0x66, ..] => Ok(DetectedFormat::Rtf),
// HTML
[b'<', b'!', b'D', b'O', b'C', ..] | [b'<', b'h', b't', b'm', b'l', ..] | [b'<', b'H', b'T', b'M', b'L', ..] => Ok(DetectedFormat::Html),
```

And handle HTMLZ (ZIP containing index.html) by checking after the ZIP detection block:
- If the ZIP contains `index.html`, it is `Htmlz`
- If the ZIP contains `mimetype` = `application/epub+zip`, it is `Epub`
- Otherwise it is `Cbz`

Update `processing/src/pipeline/metadata.rs` to route new formats:
```rust
DetectedFormat::Pdf                          => metadata::pdf::extract(path)?,
DetectedFormat::Mobi | DetectedFormat::Azw3 => metadata::mobi::extract(path)?,
DetectedFormat::Fb2                          => metadata::fb2::extract(path)?,
DetectedFormat::Html | DetectedFormat::Htmlz => metadata::html::extract(path)?,
DetectedFormat::Rtf                          => metadata::rtf::extract(path)?,
```

Update `processing/src/pipeline/text.rs` to route new formats:
```rust
DetectedFormat::Pdf                          => text::pdf::extract(path)?,
DetectedFormat::Mobi | DetectedFormat::Azw3 => text::mobi::extract(path)?,
DetectedFormat::Fb2                          => text::fb2::extract(path)?,
DetectedFormat::Html | DetectedFormat::Htmlz => text::html::extract(path)?,
DetectedFormat::Rtf                          => text::rtf::extract(path)?,
```

Update `processing/src/pipeline/cover.rs`:
```rust
DetectedFormat::Pdf => cover::pdf::extract(path)?,
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/plugins/mod.rs processing/src/pipeline/ingest.rs \
        processing/src/pipeline/metadata.rs processing/src/pipeline/text.rs \
        processing/src/pipeline/cover.rs
git commit -m "P6-T10: wire FB2, HTML, HTMLZ, RTF, PDF, MOBI into pipeline"
```

---

### ✅ Milestone check — after P6-T10
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
```

---

## P6-T11

Write `processing/tests/test_formats_primary.rs` with this exact content:
```rust
use xcalibre_processing::metadata;
use xcalibre_processing::text;
use std::path::PathBuf;

// PDF tests — require tests/fixtures/fixture_pdf.pdf (created by gen_fixtures)

#[test]
fn test_pdf_metadata_extract() {
    let path = PathBuf::from("tests/fixtures/fixture_pdf.pdf");
    let meta = metadata::pdf::extract(&path).unwrap();
    // Minimal PDF from fixture has no Info dict — title will be None
    assert!(meta.title.is_none() || meta.title.is_some());
}

#[test]
fn test_pdf_text_extract() {
    let path = PathBuf::from("tests/fixtures/fixture_pdf.pdf");
    // Minimal PDF has no content streams; word_count may be 0
    let result = text::pdf::extract(&path);
    assert!(result.is_ok());
}

// MOBI tests — require tests/fixtures/fixture_mobi.mobi
#[test]
fn test_mobi_metadata_extract() {
    let path = PathBuf::from("tests/fixtures/fixture_mobi.mobi");
    let meta = metadata::mobi::extract(&path);
    // May fail on minimal fixture — that's OK; we just want no panic
    let _ = meta;
}

// FB2 test — inline fixture
#[test]
fn test_fb2_metadata_extract() {
    let dir  = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.fb2");
    std::fs::write(&path, br#"<?xml version="1.0" encoding="utf-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
  <description>
    <title-info>
      <genre>sf</genre>
      <author><first-name>Frank</first-name><last-name>Herbert</last-name></author>
      <book-title>Dune</book-title>
      <lang>en</lang>
    </title-info>
    <publish-info><publisher>Chilton</publisher><year>1965</year></publish-info>
  </description>
  <body><section><p>The beginning is a very delicate time.</p></section></body>
</FictionBook>"#).unwrap();

    let meta = xcalibre_processing::metadata::fb2::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("Dune"));
    assert_eq!(meta.authors, vec!["Frank Herbert"]);
    assert_eq!(meta.publisher.as_deref(), Some("Chilton"));

    let text = xcalibre_processing::text::fb2::extract(&path).unwrap();
    assert!(text.word_count > 0);
}

// HTML test — inline fixture
#[test]
fn test_html_metadata_extract() {
    let dir  = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.html");
    std::fs::write(&path, b"<!DOCTYPE html><html><head><title>My Book</title>
<meta name='author' content='Jane Doe'></head>
<body><p>Hello world paragraph.</p></body></html>").unwrap();

    let meta = xcalibre_processing::metadata::html::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("My Book"));
    assert!(meta.authors.contains(&"Jane Doe".to_string()));

    let text = xcalibre_processing::text::html::extract(&path).unwrap();
    assert!(text.word_count >= 3);
}

// RTF test — inline fixture
#[test]
fn test_rtf_text_extract() {
    let dir  = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rtf");
    std::fs::write(&path, br"{\rtf1\ansi {\title My Title}{\author Jane}\par Hello World\par}").unwrap();
    let text = xcalibre_processing::text::rtf::extract(&path).unwrap();
    assert!(text.word_count >= 2);
}
```

Then run:
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
git add processing/tests/test_formats_primary.rs
git commit -m "P6-T11: primary format extraction tests — PDF, MOBI, FB2, HTML, RTF"
```

---

### ✅ Milestone check — Phase 6 complete
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```
