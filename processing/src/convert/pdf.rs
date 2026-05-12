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
            "PDF conversion requires Chrome/Chromium or wkhtmltopdf.".into()
        ))
    }
}

fn find_chrome() -> Option<String> {
    let candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "google-chrome", "chromium", "chromium-browser",
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

fn html_to_pdf_chrome(chrome: &str, html_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let status = Command::new(chrome)
        .args([
            "--headless", "--disable-gpu", "--no-sandbox",
            "--print-to-pdf-no-header",
            &format!("--print-to-pdf={}", out_path.display()),
            &format!("file://{}", html_path.display()),
        ])
        .status()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    if !status.success() {
        return Err(ProcessingError::ConversionError(format!("Chrome PDF conversion failed: {status}")));
    }
    Ok(())
}

fn html_to_pdf_wkhtmltopdf(wk: &str, html_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let status = Command::new(wk)
        .args(["--quiet", &html_path.display().to_string(), &out_path.display().to_string()])
        .status()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    if !status.success() {
        return Err(ProcessingError::ConversionError(format!("wkhtmltopdf failed: {status}")));
    }
    Ok(())
}
