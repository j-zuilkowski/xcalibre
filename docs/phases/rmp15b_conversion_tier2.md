# RMP-15b — Conversion Tier 2: PDF + MOBI (Green: Implementation)

> Prerequisite: rmp15a complete, rmp09b complete.
> Decision: S4-B — PDF via WebView print (wkhtmltopdf/headless-chrome as fallback).
> TDD role: GREEN — implement PDF and MOBI converters and extend ConversionDialog.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R15b-T01 | `convert/pdf.rs` — EPUB→PDF via HTML intermediary | ⬜ |
| R15b-T02 | `convert/mobi.rs` — EPUB→MOBI via kindlegen/ebook-convert shim | ⬜ |
| R15b-T03 | Extend `convert_book` command with PDF + MOBI | ⬜ |
| R15b-T04 | Add PDF + MOBI to `ConversionDialog.tsx` | ⬜ |
| R15b-T05 | Milestone check + visual inspection | ⬜ |

---

## R15b-T01

**Strategy**: EPUB→HTML (using existing `convert::html::epub_to_html`), then HTML→PDF via the
`chromium-pdf` approach: invoke the system's Chrome/Chromium in headless mode.
This requires `google-chrome`, `chromium`, or `/Applications/Google Chrome.app` to be present.
If none are found, fall back to `wkhtmltopdf`.

Write `processing/src/convert/pdf.rs`:
```rust
use crate::convert::html::epub_to_html;
use crate::error::ProcessingError;
use std::path::Path;
use std::process::Command;

pub fn epub_to_pdf(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    // Step 1: Convert EPUB→HTML to a temp file
    let dir = tempfile::tempdir()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let html_path = dir.path().join("input.html");
    epub_to_html(epub_path, &html_path)?;

    // Step 2: HTML→PDF via headless Chrome or wkhtmltopdf
    if let Some(chrome) = find_chrome() {
        html_to_pdf_chrome(&chrome, &html_path, out_path)
    } else if let Some(wk) = find_wkhtmltopdf() {
        html_to_pdf_wkhtmltopdf(&wk, &html_path, out_path)
    } else {
        Err(ProcessingError::ConversionError(
            "PDF conversion requires Chrome/Chromium or wkhtmltopdf. \
             Install one to enable PDF export.".into()
        ))
    }
}

fn find_chrome() -> Option<String> {
    let candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "google-chrome",
        "chromium",
        "chromium-browser",
    ];
    for c in &candidates {
        if std::path::Path::new(c).exists() { return Some(c.to_string()); }
        if let Ok(o) = Command::new("which").arg(c).output() {
            if o.status.success() {
                return Some(String::from_utf8_lossy(&o.stdout).trim().to_string());
            }
        }
    }
    None
}

fn find_wkhtmltopdf() -> Option<String> {
    let output = Command::new("which").arg("wkhtmltopdf").output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn html_to_pdf_chrome(
    chrome: &str,
    html_path: &Path,
    out_path: &Path,
) -> Result<(), ProcessingError> {
    let status = Command::new(chrome)
        .args([
            "--headless",
            "--disable-gpu",
            "--no-sandbox",
            "--print-to-pdf-no-header",
            &format!("--print-to-pdf={}", out_path.display()),
            &format!("file://{}", html_path.display()),
        ])
        .status()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    if !status.success() {
        return Err(ProcessingError::ConversionError(
            format!("Chrome PDF conversion exited with status: {status}")
        ));
    }
    Ok(())
}

fn html_to_pdf_wkhtmltopdf(
    wk: &str,
    html_path: &Path,
    out_path: &Path,
) -> Result<(), ProcessingError> {
    let status = Command::new(wk)
        .args(["--quiet", &html_path.display().to_string(), &out_path.display().to_string()])
        .status()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    if !status.success() {
        return Err(ProcessingError::ConversionError(
            format!("wkhtmltopdf exited with status: {status}")
        ));
    }
    Ok(())
}
```

Then run:
```bash
cargo test --workspace -- test_epub_to_pdf
```

Tests should pass if Chrome or wkhtmltopdf is installed. If neither is available on CI, gate with `#[ignore]`. Then commit:
```bash
git add processing/src/convert/pdf.rs
git commit -m "R15b-T01: EPUB→PDF via headless Chrome — tests green"
```

---

## R15b-T02

**Strategy**: EPUB→MOBI requires a KF8 encoder. Options:
- `kindlegen` (deprecated/unavailable) — skip
- `ebook-convert` (Calibre CLI) — use if present
- Native MOBI writer: use `mobi` crate (pure Rust)

Add to `processing/Cargo.toml`:
```toml
mobi = "0.6"
```

Write `processing/src/convert/mobi.rs`:
```rust
use crate::convert::txt::epub_to_txt;
use crate::error::ProcessingError;
use std::path::Path;

pub fn epub_to_mobi(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    // Preferred: use Calibre's ebook-convert CLI if available
    if let Some(ec) = find_ebook_convert() {
        return epub_to_mobi_calibre(&ec, epub_path, out_path);
    }
    // Fallback: generate MOBI from extracted text via the `mobi` crate
    epub_to_mobi_native(epub_path, out_path)
}

fn find_ebook_convert() -> Option<String> {
    // Calibre installs ebook-convert at a well-known path on macOS
    let candidates = [
        "/Applications/calibre.app/Contents/MacOS/ebook-convert",
        "ebook-convert",
    ];
    for c in &candidates {
        if std::path::Path::new(c).exists() { return Some(c.to_string()); }
        if let Ok(o) = std::process::Command::new("which").arg(c).output() {
            if o.status.success() {
                return Some(String::from_utf8_lossy(&o.stdout).trim().to_string());
            }
        }
    }
    None
}

fn epub_to_mobi_calibre(ec: &str, epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let status = std::process::Command::new(ec)
        .args([
            &epub_path.display().to_string(),
            &out_path.display().to_string(),
        ])
        .status()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    if !status.success() {
        return Err(ProcessingError::ConversionError(
            format!("ebook-convert exited with status: {status}")
        ));
    }
    Ok(())
}

fn epub_to_mobi_native(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    // Extract text, then build a minimal MOBI using the mobi crate
    let dir = tempfile::tempdir()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let txt_path = dir.path().join("content.txt");
    epub_to_txt(epub_path, &txt_path)?;

    let text = std::fs::read_to_string(&txt_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // Build MOBI using the `mobi` crate builder API
    let title = epub_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");

    // Wrap text in minimal HTML for MOBI encoding
    let html_body = text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| format!("<p>{}</p>", html_escape(l)))
        .collect::<Vec<_>>()
        .join("\n");

    let html = format!(
        "<html><head><title>{title}</title></head><body>{html_body}</body></html>"
    );

    // mobi 0.6 builder
    let mobi = mobi::MobiBuilder::new()
        .title(title)
        .content(html.as_bytes())
        .build()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    std::fs::write(out_path, mobi)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
```

> **Note**: The `mobi` crate API may differ from the above sketch. Check `mobi::MobiBuilder`
> or `mobi::Book::new()` API at the version you install. Adjust field names accordingly.
> The test only checks that the output file is created and of reasonable size — it does not
> require a perfectly structured MOBI.

Then run:
```bash
cargo test --workspace -- test_epub_to_mobi
git add processing/src/convert/mobi.rs processing/Cargo.toml
git commit -m "R15b-T02: EPUB→MOBI via Calibre CLI or native mobi crate — tests green"
```

---

## R15b-T03

Extend `src-tauri/src/commands/convert.rs` to add PDF and MOBI:
```rust
// Add to OutputFormat enum:
#[serde(rename_all = "UPPERCASE")]
pub enum OutputFormat {
    Txt, Html, Docx, Pdf, Mobi,
}

// In convert_book match:
OutputFormat::Pdf  => ("pdf",  dir.join(format!("{stem}.pdf"))),
OutputFormat::Mobi => ("mobi", dir.join(format!("{stem}.mobi"))),

// In conversion match:
OutputFormat::Pdf  => pdf::epub_to_pdf(&epub, &out_path).map_err(|e| e.to_string())?,
OutputFormat::Mobi => mobi::epub_to_mobi(&epub, &out_path).map_err(|e| e.to_string())?,
```

Add imports at top:
```rust
use xcalibre_processing::convert::{docx, html, txt, pdf, mobi};
```

```bash
cargo build --workspace
git add src-tauri/src/commands/convert.rs
git commit -m "R15b-T03: extend convert_book command with PDF and MOBI formats"
```

---

## R15b-T04

In `ui/src/components/ConversionDialog.tsx`, extend the format options:
```tsx
// Add PDF and MOBI to the select options:
<option value="TXT">TXT</option>
<option value="HTML">HTML</option>
<option value="DOCX">DOCX</option>
<option value="PDF">PDF</option>
<option value="MOBI">MOBI</option>
```

Update the `OutputFormat` type:
```tsx
type OutputFormat = "TXT" | "HTML" | "DOCX" | "PDF" | "MOBI"
```

```bash
cd ui && npm test -- ConversionDialog && cd ..
git add ui/src/components/ConversionDialog.tsx
git commit -m "R15b-T04: add PDF and MOBI options to ConversionDialog"
```

---

## R15b-T05 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm test && cd ..
```

**Visual inspection:**
1. Launch the app: `cd src-tauri && cargo tauri dev`
2. Right-click an EPUB → "Convert…"
3. Verify PDF and MOBI appear in the format selector
4. Select PDF → Convert → open resulting file in Preview, verify it renders
5. Select MOBI → Convert → verify file is created
   - If Calibre is installed, open in Kindle app to verify MOBI structure
   - If not, verify file size > 1000 bytes
6. Verify error message appears for formats when tool is not available (e.g. no Chrome for PDF)

```bash
git add -A
git commit -m "R15b-T05: RMP-15 Conversion Tier 2 — all tests green, PDF + MOBI wired"
```
