# RMP-09b — Conversion Output Tier 1: TXT, HTML, DOCX (Green: Implementation)

> Prerequisite: rmp09a complete (failing tests defined), rmp05b complete (xcalibre-epub crate).
> TDD role: GREEN — implement conversion backends until all tests pass.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R09b-T01 | `convert/mod.rs` + `convert/txt.rs` — EPUB→TXT | ⬜ |
| R09b-T02 | `convert/html.rs` — EPUB→HTML | ⬜ |
| R09b-T03 | `convert/docx.rs` — EPUB→DOCX | ⬜ |
| R09b-T04 | `convert_book` Tauri command | ⬜ |
| R09b-T05 | `ConversionDialog.tsx` component | ⬜ |
| R09b-T06 | Milestone check + visual inspection | ⬜ |

---

## R09b-T01

Add to `processing/Cargo.toml` under `[dependencies]`:
```toml
xcalibre-epub = { path = "../xcalibre-epub" }
```

In `processing/src/lib.rs`, add:
```rust
pub mod convert;
```

Write `processing/src/convert/mod.rs`:
```rust
pub mod docx;
pub mod html;
pub mod txt;
```

Write `processing/src/convert/txt.rs`:
```rust
use crate::error::ProcessingError;
use std::io::Write;
use std::path::Path;
use xcalibre_epub::Container;

pub fn epub_to_txt(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let spine = container.spine_items()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let mut parts: Vec<String> = Vec::new();

    for href in &spine {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let html = String::from_utf8_lossy(&bytes);
        let text = strip_html_tags(&html);
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }

    let output = parts.join("\n\n");

    let mut file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    file.write_all(output.as_bytes())
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    Ok(())
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut buf = String::new();

    let mut chars = html.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                // Collect tag name to detect script/style
                buf.clear();
                in_tag = true;
                // Peek ahead for block-level tags to insert newlines
                let mut tag_buf = String::new();
                let mut tmp = chars.clone();
                while let Some(&c) = tmp.peek() {
                    if c == '>' || c == ' ' { break; }
                    tag_buf.push(c);
                    tmp.next();
                }
                let tag_lower = tag_buf.to_lowercase();
                if matches!(tag_lower.trim_start_matches('/'),
                    "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" |
                    "br" | "li" | "blockquote" | "tr")
                {
                    result.push('\n');
                }
                if tag_lower == "script" { in_script = true; }
                if tag_lower == "style"  { in_style  = true; }
                if tag_lower == "/script" { in_script = false; }
                if tag_lower == "/style"  { in_style  = false; }
            }
            '>' => {
                in_tag = false;
            }
            _ if in_tag || in_script || in_style => {}
            _ => result.push(ch),
        }
    }

    // Collapse multiple blank lines
    let mut collapsed = String::new();
    let mut blank_count = 0u32;
    for line in result.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 { collapsed.push('\n'); }
        } else {
            blank_count = 0;
            collapsed.push_str(line);
            collapsed.push('\n');
        }
    }
    collapsed
}
```

Then run:
```bash
cargo test --workspace -- test_epub_to_txt
```

All three TXT tests must pass. Then commit:
```bash
git add processing/src/convert/mod.rs processing/src/convert/txt.rs \
        processing/src/lib.rs processing/Cargo.toml
git commit -m "R09b-T01: EPUB→TXT conversion — all TXT tests green"
```

---

## R09b-T02

Write `processing/src/convert/html.rs`:
```rust
use crate::error::ProcessingError;
use std::io::Write;
use std::path::Path;
use xcalibre_epub::Container;

pub fn epub_to_html(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let opf = container.opf()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let title = opf.title().unwrap_or("Untitled");
    let spine = container.spine_items()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let mut body_parts: Vec<String> = Vec::new();

    for href in &spine {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let fragment = String::from_utf8_lossy(&bytes);
        // Extract <body> content if present, otherwise use whole item
        let content = extract_body_content(&fragment).unwrap_or_else(|| fragment.to_string());
        body_parts.push(content);
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<style>
  body {{ font-family: Georgia, serif; max-width: 800px; margin: 0 auto; padding: 2rem; line-height: 1.6; }}
  h1, h2, h3 {{ margin-top: 2rem; }}
  p {{ margin: 0.8rem 0; }}
</style>
</head>
<body>
{body}
</body>
</html>"#,
        title = escape_html(title),
        body = body_parts.join("\n<hr>\n"),
    );

    let mut file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    file.write_all(html.as_bytes())
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    Ok(())
}

fn extract_body_content(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let body_start = lower.find("<body")?;
    let content_start = html[body_start..].find('>')? + body_start + 1;
    let body_end = lower.rfind("</body>")?;
    Some(html[content_start..body_end].to_string())
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}
```

Then run:
```bash
cargo test --workspace -- test_epub_to_html
```

Both HTML tests must pass. Then commit:
```bash
git add processing/src/convert/html.rs
git commit -m "R09b-T02: EPUB→HTML conversion — all HTML tests green"
```

---

## R09b-T03

Add to `processing/Cargo.toml` under `[dependencies]`:
```toml
docx-rs = "0.4"
```

Remove `docx-rs` from `[dev-dependencies]` (it moved to deps).

Write `processing/src/convert/docx.rs`:
```rust
use crate::error::ProcessingError;
use docx_rs::{
    Docx, Paragraph, Run, RunProperty, ParagraphStyle,
};
use std::io::{BufWriter, Write};
use std::path::Path;
use xcalibre_epub::Container;

pub fn epub_to_docx(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let spine = container.spine_items()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let mut doc = Docx::new();

    for href in &spine {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let html = String::from_utf8_lossy(&bytes);
        let paragraphs = html_to_paragraphs(&html);

        for para_text in paragraphs {
            if para_text.trim().is_empty() { continue; }
            let run = Run::new().add_text(&para_text);
            let para = Paragraph::new().add_run(run);
            doc = doc.add_paragraph(para);
        }
    }

    let file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let mut writer = BufWriter::new(file);
    let bytes = doc.build()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    writer.write_all(&bytes)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    Ok(())
}

fn html_to_paragraphs(html: &str) -> Vec<String> {
    // Split on block-level elements, strip tags within each paragraph
    let para_re = regex::Regex::new(r"<(?:p|div|h[1-6]|li|blockquote)[^>]*>(.*?)</(?:p|div|h[1-6]|li|blockquote)>").unwrap();
    let tag_re  = regex::Regex::new(r"<[^>]+>").unwrap();

    let mut result: Vec<String> = Vec::new();
    for cap in para_re.captures_iter(html) {
        let inner = &cap[1];
        let text  = tag_re.replace_all(inner, "");
        let text  = decode_html_entities(&text);
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            result.push(trimmed);
        }
    }

    // Fallback: if no block tags found, strip all tags
    if result.is_empty() {
        let stripped = tag_re.replace_all(html, " ");
        let text = decode_html_entities(&stripped);
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            result.push(trimmed);
        }
    }

    result
}

fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;",  "&")
     .replace("&lt;",   "<")
     .replace("&gt;",   ">")
     .replace("&quot;", "\"")
     .replace("&#39;",  "'")
     .replace("&nbsp;", " ")
}
```

Then run:
```bash
cargo test --workspace -- test_epub_to_docx
```

Both DOCX tests must pass. Then commit:
```bash
git add processing/src/convert/docx.rs processing/Cargo.toml
git commit -m "R09b-T03: EPUB→DOCX conversion — all DOCX tests green"
```

---

## R09b-T04

Add to `processing/src/error.rs`:
```rust
// Add to ProcessingError enum:
#[error("conversion error: {0}")]
ConversionError(String),
```

In `src-tauri/src/commands/`, write `convert.rs`:
```rust
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use xcalibre_processing::convert::{docx, html, txt};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OutputFormat {
    Txt,
    Html,
    Docx,
}

#[tauri::command]
pub async fn convert_book(
    epub_path: String,
    output_format: OutputFormat,
    output_dir: Option<String>,
    app: AppHandle,
) -> Result<String, String> {
    let epub = std::path::PathBuf::from(&epub_path);
    if !epub.exists() {
        return Err(format!("source file not found: {epub_path}"));
    }

    let stem = epub.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let dir = output_dir
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            app.path().download_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("/tmp"))
        });

    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let (ext, out_path) = match output_format {
        OutputFormat::Txt  => ("txt",  dir.join(format!("{stem}.txt"))),
        OutputFormat::Html => ("html", dir.join(format!("{stem}.html"))),
        OutputFormat::Docx => ("docx", dir.join(format!("{stem}.docx"))),
    };

    match output_format {
        OutputFormat::Txt  => txt::epub_to_txt(&epub, &out_path).map_err(|e| e.to_string())?,
        OutputFormat::Html => html::epub_to_html(&epub, &out_path).map_err(|e| e.to_string())?,
        OutputFormat::Docx => docx::epub_to_docx(&epub, &out_path).map_err(|e| e.to_string())?,
    }

    Ok(out_path.to_string_lossy().to_string())
}
```

Register in `src-tauri/src/main.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::convert::convert_book,
])
```

Then run:
```bash
cargo build --workspace
git add src-tauri/src/commands/convert.rs src-tauri/src/main.rs \
        processing/src/error.rs
git commit -m "R09b-T04: convert_book Tauri command for TXT/HTML/DOCX"
```

---

## R09b-T05

Write `ui/src/components/ConversionDialog.tsx`:
```tsx
import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"

type OutputFormat = "TXT" | "HTML" | "DOCX"

interface Book {
  id: string
  title: string
  authors: string[]
  format: string
  cover_path: string | null
  progress_percent: number
  last_opened_at: string | null
}

interface Props {
  book: Book & { file_path?: string }
  onClose: () => void
}

export function ConversionDialog({ book, onClose }: Props) {
  const [format, setFormat]   = useState<OutputFormat>("TXT")
  const [loading, setLoading] = useState(false)
  const [outputPath, setOutputPath] = useState<string | null>(null)
  const [error, setError]     = useState<string | null>(null)

  async function handleConvert() {
    if (!book.file_path) {
      setError("Book file path is not available.")
      return
    }
    setLoading(true)
    setError(null)
    try {
      const path = await invoke<string>("convert_book", {
        epubPath:     book.file_path,
        outputFormat: format,
        outputDir:    null,
      })
      setOutputPath(path)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Convert book"
      style={{
        position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)",
        display: "flex", alignItems: "center", justifyContent: "center",
        zIndex: 1000,
      }}
    >
      <div style={{
        background: "var(--bg-surface, #1e1e2e)",
        borderRadius: "12px", padding: "2rem", minWidth: "360px",
        color: "var(--text-primary, #cdd6f4)",
      }}>
        <h2 style={{ margin: "0 0 1.5rem", fontSize: "1.2rem" }}>
          Convert "{book.title}"
        </h2>

        <label style={{ display: "block", marginBottom: "0.5rem", fontSize: "0.9rem" }}>
          Output format
        </label>
        <select
          data-testid="output-format-select"
          value={format}
          onChange={e => setFormat(e.target.value as OutputFormat)}
          style={{
            width: "100%", padding: "0.5rem",
            background: "var(--bg-overlay, #313244)",
            color: "inherit", border: "1px solid var(--border, #45475a)",
            borderRadius: "6px", marginBottom: "1.5rem",
          }}
        >
          <option value="TXT">TXT</option>
          <option value="HTML">HTML</option>
          <option value="DOCX">DOCX</option>
        </select>

        {error && (
          <p style={{ color: "var(--red, #f38ba8)", marginBottom: "1rem", fontSize: "0.9rem" }}>
            {error}
          </p>
        )}

        {outputPath && (
          <div
            data-testid="conversion-success"
            style={{
              background: "var(--green-dim, #1e3a2a)", borderRadius: "6px",
              padding: "0.75rem", marginBottom: "1rem", fontSize: "0.9rem",
            }}
          >
            Saved to: <code style={{ wordBreak: "break-all" }}>{outputPath}</code>
          </div>
        )}

        <div style={{ display: "flex", gap: "0.75rem", justifyContent: "flex-end" }}>
          <button
            onClick={onClose}
            style={{
              padding: "0.5rem 1.25rem",
              background: "var(--bg-overlay, #313244)",
              border: "none", borderRadius: "6px", cursor: "pointer",
              color: "inherit",
            }}
          >
            {outputPath ? "Close" : "Cancel"}
          </button>
          {!outputPath && (
            <button
              data-testid="convert-btn"
              onClick={handleConvert}
              disabled={loading}
              style={{
                padding: "0.5rem 1.25rem",
                background: "var(--blue, #89b4fa)",
                border: "none", borderRadius: "6px", cursor: "pointer",
                color: "#1e1e2e", fontWeight: 600,
                opacity: loading ? 0.6 : 1,
              }}
            >
              {loading ? "Converting…" : "Convert"}
            </button>
          )}
        </div>
      </div>
    </div>
  )
}
```

Then run:
```bash
cd ui && npm test -- ConversionDialog && cd ..
```

All four ConversionDialog tests must pass.

Wire the dialog into the book context menu / detail panel. In `ui/src/components/BookCard.tsx` (or wherever the per-book context menu lives), add:
```tsx
import { ConversionDialog } from "./ConversionDialog"

// In component state:
const [showConvert, setShowConvert] = useState(false)

// In context menu:
<button onClick={() => setShowConvert(true)}>Convert…</button>

// At render bottom:
{showConvert && (
  <ConversionDialog book={book} onClose={() => setShowConvert(false)} />
)}
```

```bash
git add ui/src/components/ConversionDialog.tsx
git commit -m "R09b-T05: ConversionDialog component — all UI tests green"
```

---

## R09b-T06 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm test && cd ..
```

All three test suites must pass at zero warnings.

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
2. Import an EPUB book from the library
3. Right-click the book → "Convert…"
4. Verify the ConversionDialog appears with format selector showing TXT / HTML / DOCX
5. Select TXT → Convert → verify success message with file path
6. Open the file in Finder — confirm it contains readable text with no HTML tags
7. Repeat for HTML — open in browser, confirm valid styled page
8. Repeat for DOCX — open in Pages/Word, confirm readable text

```bash
pkill -x xcalibre 2>/dev/null || true
```

```bash
git add -A
git commit -m "R09b-T06: RMP-09 Conversion Tier 1 — all tests green, UI wired"
```
