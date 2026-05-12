use crate::convert::txt::strip_html_to_text;
use crate::error::ProcessingError;
use std::io::Write;
use std::path::Path;
use xcalibre_epub::Container;

pub fn epub_to_snb_text(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let title = {
        let opf_path = container.opf_path().to_string();
        container.read_item(&opf_path).ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|xml| xcalibre_epub::opf::EpubOPF::parse(&xml).ok())
            .and_then(|opf| opf.title.map(|s| s))
            .unwrap_or_else(|| "Untitled".to_string())
    };

    let spine = container.spine_hrefs();

    let mut file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    writeln!(file, "SNB TEXT EXPORT")
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    writeln!(file, "Title: {title}")
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    writeln!(file, "---")
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    for (idx, href) in spine.iter().enumerate() {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let html = String::from_utf8_lossy(&bytes);
        let text = strip_html_to_text(&html);
        if text.trim().is_empty() { continue; }

        writeln!(file, "\n[CHAPTER {}]", idx + 1)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        writeln!(file, "{}", text.trim())
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    }
    Ok(())
}
