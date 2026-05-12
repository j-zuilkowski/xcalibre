use crate::convert::html::epub_to_html;
use crate::error::ProcessingError;
use std::io::Write;
use std::path::Path;
use xcalibre_epub::Container;
use zip::write::{FileOptions, ZipWriter};

pub fn epub_to_htmlz(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let (title, author) = read_title_author(&container);

    let html_dir = tempfile::tempdir()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let html_path = html_dir.path().join("index.html");
    epub_to_html(epub_path, &html_path)?;
    let html_bytes = std::fs::read(&html_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let metadata_opf = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0">
  <metadata>
    <dc:title xmlns:dc="http://purl.org/dc/elements/1.1/">{}</dc:title>
    <dc:creator xmlns:dc="http://purl.org/dc/elements/1.1/">{}</dc:creator>
  </metadata>
</package>"#,
        xml_escape(&title),
        xml_escape(&author),
    );

    let file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let mut zip = ZipWriter::new(file);
    let opts = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("index.html", opts)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    zip.write_all(&html_bytes)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    zip.start_file("metadata.opf", opts)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    zip.write_all(metadata_opf.as_bytes())
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    zip.finish()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}

fn read_title_author(container: &Container) -> (String, String) {
    let opf_path = container.opf_path().to_string();
    if let Ok(bytes) = container.read_item(&opf_path) {
        if let Ok(xml) = String::from_utf8(bytes) {
            if let Ok(opf) = xcalibre_epub::opf::EpubOPF::parse(&xml) {
                let title  = opf.title.unwrap_or_else(|| "Untitled".to_string());
                let author = opf.authors.into_iter().next().unwrap_or_else(|| "Unknown".to_string());
                return (title, author);
            }
        }
    }
    ("Untitled".to_string(), "Unknown".to_string())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
