# RMP-18b — Conversion Tier 3: FB2, RTF, HTMLZ (Green: Implementation)

> Prerequisite: rmp18a complete.
> TDD role: GREEN — implement FB2, RTF, HTMLZ converters and wire into UI.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R18b-T01 | `convert/fb2.rs` — EPUB→FB2 via quick-xml | ⬜ |
| R18b-T02 | `convert/rtf.rs` — EPUB→RTF via control-word builder | ⬜ |
| R18b-T03 | `convert/htmlz.rs` — EPUB→HTMLZ as flat ZIP | ⬜ |
| R18b-T04 | Extend convert_book command + ConversionDialog | ⬜ |
| R18b-T05 | Milestone check + visual inspection | ⬜ |

---

## R18b-T01

Add to `processing/Cargo.toml`:
```toml
quick-xml = { version = "0.36", features = ["serialize"] }
```

Write `processing/src/convert/fb2.rs`:
```rust
use crate::convert::txt::strip_html_to_text;
use crate::error::ProcessingError;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use std::io::{BufWriter, Write};
use std::path::Path;
use xcalibre_epub::Container;

pub fn epub_to_fb2(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let opf = container.opf()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let title   = opf.title().unwrap_or("Untitled");
    let authors = opf.authors();
    let spine   = container.spine_items()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let mut writer = Writer::new_with_indent(BufWriter::new(file), b' ', 2);

    // XML declaration
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // <FictionBook>
    let mut fb_start = BytesStart::new("FictionBook");
    fb_start.push_attribute(("xmlns", "http://www.gribuser.ru/xml/fictionbook/2.0"));
    fb_start.push_attribute(("xmlns:l", "http://www.w3.org/1999/xlink"));
    writer.write_event(Event::Start(fb_start))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // <description><title-info>
    writer.write_event(Event::Start(BytesStart::new("description")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    writer.write_event(Event::Start(BytesStart::new("title-info")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // <book-title>
    writer.write_event(Event::Start(BytesStart::new("book-title")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    writer.write_event(Event::Text(BytesText::new(title)))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    writer.write_event(Event::End(BytesEnd::new("book-title")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // <author> entries
    for author in &authors {
        writer.write_event(Event::Start(BytesStart::new("author")))
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        writer.write_event(Event::Start(BytesStart::new("nickname")))
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        writer.write_event(Event::Text(BytesText::new(author)))
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        writer.write_event(Event::End(BytesEnd::new("nickname")))
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        writer.write_event(Event::End(BytesEnd::new("author")))
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    }

    writer.write_event(Event::End(BytesEnd::new("title-info")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    writer.write_event(Event::End(BytesEnd::new("description")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // <body>
    writer.write_event(Event::Start(BytesStart::new("body")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    for (idx, href) in spine.iter().enumerate() {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let html  = String::from_utf8_lossy(&bytes);
        let text  = strip_html_to_text(&html);

        // <section>
        let mut sec = BytesStart::new("section");
        sec.push_attribute(("id", format!("ch{idx}").as_str()));
        writer.write_event(Event::Start(sec))
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

        for para in text.split("\n\n").filter(|p| !p.trim().is_empty()) {
            writer.write_event(Event::Start(BytesStart::new("p")))
                .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
            writer.write_event(Event::Text(BytesText::new(para.trim())))
                .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
            writer.write_event(Event::End(BytesEnd::new("p")))
                .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        }

        writer.write_event(Event::End(BytesEnd::new("section")))
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    }

    writer.write_event(Event::End(BytesEnd::new("body")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    writer.write_event(Event::End(BytesEnd::new("FictionBook")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    writer.into_inner().flush()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}
```

> **Note**: `strip_html_to_text` must be made `pub` in `convert/txt.rs`, or the common
> logic extracted to `convert/util.rs` as a shared helper.

In `processing/src/convert/mod.rs`, add:
```rust
pub mod fb2;
```

Then run:
```bash
cargo test --workspace -- test_epub_to_fb2 test_fb2
git add processing/src/convert/fb2.rs processing/src/convert/mod.rs processing/Cargo.toml
git commit -m "R18b-T01: EPUB→FB2 conversion — all FB2 tests green"
```

---

## R18b-T02

Write `processing/src/convert/rtf.rs`:
```rust
use crate::convert::txt::strip_html_to_text;
use crate::error::ProcessingError;
use std::io::Write;
use std::path::Path;
use xcalibre_epub::Container;

pub fn epub_to_rtf(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let opf   = container.opf().ok();
    let title = opf.as_ref().and_then(|o| o.title()).unwrap_or("Untitled");
    let spine = container.spine_items()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let mut rtf = String::new();

    // RTF header
    rtf.push_str("{\\rtf1\\ansi\\ansicpg1252\\deff0\n");
    rtf.push_str("{\\fonttbl{\\f0\\froman\\fcharset0 Times New Roman;}}\n");
    rtf.push_str("{\\colortbl;\\red0\\green0\\blue0;}\n");
    rtf.push_str("\\widowctrl\\hyphauto\n");

    // Title paragraph
    rtf.push_str(&format!(
        "{{\\pard\\sb240\\sa120\\b\\fs36 {}\\par}}\n",
        rtf_escape(title)
    ));

    for href in &spine {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let html  = String::from_utf8_lossy(&bytes);
        let text  = strip_html_to_text(&html);

        for para in text.split("\n\n").filter(|p| !p.trim().is_empty()) {
            rtf.push_str(&format!(
                "{{\\pard\\sb0\\sa120\\fs24 {}\\par}}\n",
                rtf_escape(para.trim())
            ));
        }
    }

    rtf.push('}');

    let mut file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    file.write_all(rtf.as_bytes())
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}

fn rtf_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '{'  => out.push_str("\\{"),
            '}'  => out.push_str("\\}"),
            c if c.is_ascii() => out.push(c),
            c => {
                // Unicode escape for non-ASCII
                out.push_str(&format!("\\u{}?", c as u32));
            }
        }
    }
    out
}
```

In `processing/src/convert/mod.rs`, add:
```rust
pub mod rtf;
```

Then run:
```bash
cargo test --workspace -- test_epub_to_rtf test_rtf
git add processing/src/convert/rtf.rs processing/src/convert/mod.rs
git commit -m "R18b-T02: EPUB→RTF conversion — all RTF tests green"
```

---

## R18b-T03

Write `processing/src/convert/htmlz.rs`:
```rust
use crate::convert::html::epub_to_html;
use crate::error::ProcessingError;
use std::io::Write;
use std::path::Path;
use xcalibre_epub::Container;
use zip::write::{FileOptions, SimpleFileOptions, ZipWriter};

pub fn epub_to_htmlz(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let opf = container.opf()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // Generate single-file HTML
    let html_dir = tempfile::tempdir()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let html_path = html_dir.path().join("index.html");
    epub_to_html(epub_path, &html_path)?;
    let html_bytes = std::fs::read(&html_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // Generate minimal OPF metadata
    let title   = opf.title().unwrap_or("Untitled");
    let authors = opf.authors();
    let author_str = authors.first().map(|s| s.as_str()).unwrap_or("Unknown");
    let metadata_opf = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0">
  <metadata>
    <dc:title xmlns:dc="http://purl.org/dc/elements/1.1/">{title}</dc:title>
    <dc:creator xmlns:dc="http://purl.org/dc/elements/1.1/">{author}</dc:creator>
  </metadata>
</package>"#,
        title  = xml_escape(title),
        author = xml_escape(author_str),
    );

    // Write ZIP (HTMLZ)
    let file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let mut zip = ZipWriter::new(file);
    let opts: SimpleFileOptions = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("index.html", opts)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    zip.write_all(&html_bytes)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    zip.start_file("metadata.opf", opts)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    zip.write_all(metadata_opf.as_bytes())
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // Copy cover image if present
    if let Ok(cover) = container.cover_bytes() {
        zip.start_file("cover.jpg", opts)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        zip.write_all(&cover)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    }

    zip.finish()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
```

In `processing/src/convert/mod.rs`, add:
```rust
pub mod htmlz;
```

Then run:
```bash
cargo test --workspace -- test_epub_to_htmlz test_htmlz
git add processing/src/convert/htmlz.rs processing/src/convert/mod.rs
git commit -m "R18b-T03: EPUB→HTMLZ conversion — all HTMLZ tests green"
```

---

## R18b-T04

In `src-tauri/src/commands/convert.rs`, add:
```rust
use xcalibre_processing::convert::{fb2, rtf, htmlz};

// Add to OutputFormat enum:
Fb2, Rtf, Htmlz,

// In path/extension match:
OutputFormat::Fb2   => ("fb2",   dir.join(format!("{stem}.fb2"))),
OutputFormat::Rtf   => ("rtf",   dir.join(format!("{stem}.rtf"))),
OutputFormat::Htmlz => ("htmlz", dir.join(format!("{stem}.htmlz"))),

// In conversion match:
OutputFormat::Fb2   => fb2::epub_to_fb2(&epub, &out_path).map_err(|e| e.to_string())?,
OutputFormat::Rtf   => rtf::epub_to_rtf(&epub, &out_path).map_err(|e| e.to_string())?,
OutputFormat::Htmlz => htmlz::epub_to_htmlz(&epub, &out_path).map_err(|e| e.to_string())?,
```

In `ui/src/components/ConversionDialog.tsx`:
```tsx
type OutputFormat = "TXT" | "HTML" | "DOCX" | "PDF" | "MOBI" | "KEPUB" | "FB2" | "RTF" | "HTMLZ"

// Add to <select> options:
<option value="FB2">FB2</option>
<option value="RTF">RTF</option>
<option value="HTMLZ">HTMLZ</option>
```

```bash
cargo build --workspace
cd ui && npm test -- ConversionDialog && cd ..
git add src-tauri/src/commands/convert.rs ui/src/components/ConversionDialog.tsx
git commit -m "R18b-T04: FB2/RTF/HTMLZ in convert_book command and ConversionDialog"
```

---

## R18b-T05 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm test && cd ..
```

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

1. Verify the UI:
2. Right-click an EPUB → "Convert…"
3. Verify FB2, RTF, HTMLZ appear in format selector
4. Convert to FB2 → open in a text editor, verify `<FictionBook>` XML structure
5. Convert to RTF → open in TextEdit/Word, verify readable formatted text
6. Convert to HTMLZ → rename to `.zip`, extract, open `index.html` in browser

```bash
pkill -x xcalibre 2>/dev/null || true
```

```bash
git add -A
git commit -m "R18b-T05: RMP-18 Conversion Tier 3 — all tests green, UI wired"
```
