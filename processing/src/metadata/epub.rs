use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let opf_path = {
        let mut container = archive
            .by_name("META-INF/container.xml")
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        let mut xml = String::new();
        container.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;
        let doc = roxmltree::Document::parse(&xml)
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        doc.descendants()
            .find(|n| n.tag_name().name() == "rootfile")
            .and_then(|n| n.attribute("full-path"))
            .map(|s| s.to_string())
            .ok_or_else(|| ProcessingError::MetadataError("no rootfile in container.xml".into()))?
    };

    let opf_xml = {
        let mut opf = archive
            .by_name(&opf_path)
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        let mut xml = String::new();
        opf.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;
        xml
    };

    let doc = roxmltree::Document::parse(&opf_xml)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut meta = BookMetadata::default();

    for node in doc.descendants() {
        match node.tag_name().name() {
            "title"       => meta.title       = node.text().map(str::trim).map(String::from),
            "creator"     => meta.authors.push(node.text().unwrap_or("").trim().to_string()),
            "language"    => meta.language    = node.text().map(str::trim).map(String::from),
            "publisher"   => meta.publisher   = node.text().map(str::trim).map(String::from),
            "date"        => meta.published   = node.text().map(str::trim).map(String::from),
            "description" => meta.description = node.text().map(str::trim).map(String::from),
            "identifier"  => {
                let scheme = node.attribute("opf:scheme")
                    .or_else(|| node.attribute("scheme"))
                    .unwrap_or("");
                if scheme.eq_ignore_ascii_case("isbn") {
                    meta.isbn = node.text().map(str::trim).map(String::from);
                }
            }
            _ => {}
        }
    }

    meta.authors.retain(|a| !a.is_empty());
    Ok(meta)
}