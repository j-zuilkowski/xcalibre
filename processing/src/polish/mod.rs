use crate::error::ProcessingError;
use std::path::Path;
use xcalibre_epub::Container;

pub fn pretty_print_epub(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let mut new_container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let spine = container.spine_hrefs();
    for href in &spine {
        if let Ok(data) = container.read_item(href) {
            let html = String::from_utf8_lossy(&data);
            // Simple pretty-print: add newlines after tags
            let pretty = html.replace("><", ">\n<");
            let _ = new_container.write_item(href, pretty.as_bytes(), "application/xhtml+xml");
        }
    }

    new_container.save_as(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}
