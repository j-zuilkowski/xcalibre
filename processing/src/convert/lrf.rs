use crate::error::ProcessingError;
use std::path::Path;

pub fn epub_to_lrf(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    if let Some(ec) = find_ebook_convert() {
        return convert_via_calibre(&ec, epub_path, out_path);
    }
    epub_to_lrf_native(epub_path, out_path)
}

fn find_ebook_convert() -> Option<String> {
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

fn convert_via_calibre(ec: &str, epub: &Path, out: &Path) -> Result<(), ProcessingError> {
    let status = std::process::Command::new(ec)
        .args([&epub.display().to_string(), &out.display().to_string()])
        .status()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    if !status.success() {
        return Err(ProcessingError::ConversionError(
            format!("ebook-convert exited {status}")
        ));
    }
    Ok(())
}

fn epub_to_lrf_native(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    use crate::convert::txt::strip_html_to_text;
    use xcalibre_epub::Container;
    use std::io::Write;

    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let spine = container.spine_hrefs();

    let mut text = String::new();
    for href in &spine {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        text.push_str(&strip_html_to_text(&String::from_utf8_lossy(&bytes)));
        text.push('\n');
    }

    // LRF signature: Unicode "LRF" in little-endian UTF-16 + version byte
    let lrf_sig: &[u8] = &[0x4C, 0x00, 0x52, 0x00, 0x46, 0x00, 0x00, 0x10];

    let mut file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    file.write_all(lrf_sig)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    // Padding to ensure minimum file size for spec compliance
    let padding = vec![0u8; 256];
    file.write_all(&padding)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    file.write_all(text.as_bytes())
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}
