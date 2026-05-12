use crate::convert::txt::strip_html_to_text;
use crate::error::ProcessingError;
use std::io::Write;
use std::path::Path;
use xcalibre_epub::Container;

pub fn epub_to_pml(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let spine = container.spine_hrefs();

    let mut file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    for (idx, href) in spine.iter().enumerate() {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let html  = String::from_utf8_lossy(&bytes);
        let text  = strip_html_to_text(&html);

        writeln!(file, "\\c Chapter {}", idx + 1)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        for para in text.split("\n\n").filter(|p| !p.trim().is_empty()) {
            writeln!(file, "\\p {}", para.trim())
                .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        }
    }
    Ok(())
}

pub fn epub_to_pdb(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let dir = tempfile::tempdir()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let pml_path = dir.path().join("content.pml");
    epub_to_pml(epub_path, &pml_path)?;
    let pml_bytes = std::fs::read(&pml_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let mut pdb = Vec::new();
    let name = b"xcalibre_export\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    pdb.extend_from_slice(&name[..32]);
    pdb.extend_from_slice(&[0x00, 0x00]); // attributes
    pdb.extend_from_slice(&[0x00, 0x00]); // version
    pdb.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // creation date
    pdb.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // modification date
    pdb.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // backup date
    pdb.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // modification number
    pdb.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // app info offset
    pdb.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // sort info offset
    pdb.extend_from_slice(b"TEXT");                    // type
    pdb.extend_from_slice(b"REAd");                    // creator
    pdb.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // unique ID seed
    pdb.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // next record list
    pdb.extend_from_slice(&[0x00, 0x01]);              // number of records
    let record_offset: u32 = 86;
    pdb.extend_from_slice(&record_offset.to_be_bytes());
    pdb.extend_from_slice(&[0x40, 0x00, 0x00, 0x00]); // record attributes
    pdb.extend_from_slice(&[0x00, 0x00]);              // padding
    pdb.extend_from_slice(&pml_bytes);

    std::fs::write(out_path, &pdb)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}

pub fn epub_to_rb(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let title = {
        let opf_path = container.opf_path().to_string();
        container.read_item(&opf_path).ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|xml| xcalibre_epub::opf::EpubOPF::parse(&xml).ok())
            .and_then(|opf| opf.title)
            .unwrap_or_else(|| "Untitled".to_string())
    };

    let spine = container.spine_hrefs();

    let mut html = format!("<html><head><title>{}</title></head><body>", title);
    for href in &spine {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        html.push_str(&String::from_utf8_lossy(&bytes));
    }
    html.push_str("</body></html>");

    let mut file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    file.write_all(b"BBeB")
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    file.write_all(html.as_bytes())
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}
