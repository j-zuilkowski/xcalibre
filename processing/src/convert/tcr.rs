use crate::convert::txt::strip_html_to_text;
use crate::error::ProcessingError;
use std::io::Write;
use std::path::Path;
use xcalibre_epub::Container;

const TCR_MAGIC: &[u8] = b"!!8-Bit!!";

pub fn epub_to_tcr(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let spine = container.spine_hrefs();

    let mut text = String::new();
    for href in &spine {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let html = String::from_utf8_lossy(&bytes);
        text.push_str(&strip_html_to_text(&html));
        text.push('\n');
    }

    let compressed = tcr_compress(text.as_bytes());

    let mut file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    file.write_all(&compressed)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}

fn tcr_compress(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(TCR_MAGIC.len() + 2 + data.len());
    out.extend_from_slice(TCR_MAGIC);
    out.push(0x00);
    out.push(0x00);
    out.extend_from_slice(data);
    out
}
