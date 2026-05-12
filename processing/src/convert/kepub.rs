use crate::error::ProcessingError;
use xcalibre_epub::Container;

pub fn epub_to_kepub(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let spine = container.spine_hrefs();
    let mut new_container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    for href in &spine {
        if let Ok(data) = container.read_item(href) {
            let html = String::from_utf8_lossy(&data);
            // Wrap text content in kobo spans
            let modified = wrap_kobo_spans(&html);
            let _ = new_container.write_item(href, modified.as_bytes(), "application/xhtml+xml");
        }
    }

    // Update OPF to add Kobo namespace
    if let Ok(opf_data) = container.read_item(container.opf_path()) {
        let opf_str = String::from_utf8_lossy(&opf_data);
        let updated = opf_str.replace(
            "<package",
            "<package xmlns:kobo=\"http://www.kobo.com\""
        );
        let _ = new_container.write_item(container.opf_path(), updated.as_bytes(), "application/oebps-package+xml");
    }

    new_container.save_as(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}

fn wrap_kobo_spans(html: &str) -> String {
    // Simple: wrap text paragraphs with kobo span markers
    html.replace("<p>", "<p><span class=\"kobo-span\" epub:type=\"…\">")
        .replace("</p>", "</span></p>")
}
