use crate::error::ProcessingError;
use quick_xml::{events::*, Writer};
use std::io::Cursor;
use std::path::Path;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OPFMetadata {
    pub title:        Option<String>,
    pub authors:      Vec<String>,
    pub language:     Option<String>,
    pub publisher:    Option<String>,
    pub published:    Option<String>,
    pub description:  Option<String>,
    pub isbn:         Option<String>,
    pub series:       Option<String>,
    pub series_index: Option<f32>,
    pub tags:         Vec<String>,
    pub calibre_id:   Option<String>,
}

/// Read an OPF 2.x / 3.x file into OPFMetadata.
pub fn read_opf(path: &Path) -> Result<OPFMetadata, ProcessingError> {
    let xml = std::fs::read_to_string(path).map_err(ProcessingError::IoError)?;
    let doc = roxmltree::Document::parse(&xml)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    let mut meta = OPFMetadata::default();
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
                    .or_else(|| node.attribute("scheme")).unwrap_or("");
                if scheme.eq_ignore_ascii_case("isbn") {
                    meta.isbn = node.text().map(str::trim).map(String::from);
                }
            }
            "subject" => meta.tags.push(node.text().unwrap_or("").trim().to_string()),
            "meta" => {
                let name = node.attribute("name").unwrap_or("");
                let content = node.attribute("content").unwrap_or("");
                match name {
                    "calibre:series"       => meta.series = Some(content.to_string()),
                    "calibre:series_index" => meta.series_index = content.parse().ok(),
                    _                      => {}
                }
            }
            _ => {}
        }
    }
    meta.authors.retain(|a| !a.is_empty());
    meta.tags.retain(|t| !t.is_empty());
    Ok(meta)
}

/// Write OPFMetadata as a Calibre-compatible OPF 2.x sidecar file.
pub fn write_opf_sidecar(path: &Path, meta: &OPFMetadata) -> Result<(), ProcessingError> {
    let mut w = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    // XML declaration
    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut pkg = BytesStart::new("package");
    pkg.push_attribute(("xmlns", "http://www.idpf.org/2007/opf"));
    pkg.push_attribute(("version", "2.0"));
    pkg.push_attribute(("unique-identifier", "uuid_id"));
    w.write_event(Event::Start(pkg))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut md = BytesStart::new("metadata");
    md.push_attribute(("xmlns:dc", "http://purl.org/dc/elements/1.1/"));
    md.push_attribute(("xmlns:opf", "http://www.idpf.org/2007/opf"));
    md.push_attribute(("xmlns:calibre", "http://calibre.kovidgoyal.net/2009/metadata"));
    w.write_event(Event::Start(md))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    write_dc_element(&mut w, "title", meta.title.as_deref().unwrap_or("Unknown"))?;
    for author in &meta.authors {
        write_dc_element_with_attr(&mut w, "creator", author, &[("opf:role", "aut")])?;
    }
    write_dc_element(&mut w, "language", meta.language.as_deref().unwrap_or("en"))?;
    if let Some(p) = &meta.publisher   { write_dc_element(&mut w, "publisher",   p)?; }
    if let Some(d) = &meta.published   { write_dc_element(&mut w, "date",        d)?; }
    if let Some(d) = &meta.description { write_dc_element(&mut w, "description", d)?; }
    if let Some(isbn) = &meta.isbn {
        write_dc_element_with_attr(&mut w, "identifier", isbn, &[("opf:scheme", "ISBN")])?;
    }
    for tag in &meta.tags {
        write_dc_element(&mut w, "subject", tag)?;
    }
    if let Some(s) = &meta.series {
        write_meta(&mut w, "calibre:series", s)?;
        if let Some(idx) = meta.series_index {
            write_meta(&mut w, "calibre:series_index", &idx.to_string())?;
        }
    }

    w.write_event(Event::End(BytesEnd::new("metadata")))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    w.write_event(Event::End(BytesEnd::new("package")))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let xml = String::from_utf8(w.into_inner().into_inner())
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    std::fs::write(path, xml).map_err(ProcessingError::IoError)
}

fn write_dc_element(w: &mut Writer<Cursor<Vec<u8>>>, name: &str, text: &str)
    -> Result<(), ProcessingError>
{
    let tag = format!("dc:{}", name);
    w.write_event(Event::Start(BytesStart::new(&tag)))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    w.write_event(Event::Text(BytesText::new(text)))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    w.write_event(Event::End(BytesEnd::new(&tag)))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))
}

fn write_dc_element_with_attr(
    w: &mut Writer<Cursor<Vec<u8>>>, name: &str, text: &str, attrs: &[(&str, &str)],
) -> Result<(), ProcessingError> {
    let tag = format!("dc:{}", name);
    let mut start = BytesStart::new(&tag);
    for (k, v) in attrs { start.push_attribute((*k, *v)); }
    w.write_event(Event::Start(start))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    w.write_event(Event::Text(BytesText::new(text)))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    w.write_event(Event::End(BytesEnd::new(&tag)))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))
}

fn write_meta(w: &mut Writer<Cursor<Vec<u8>>>, name: &str, content: &str)
    -> Result<(), ProcessingError>
{
    let mut start = BytesStart::new("meta");
    start.push_attribute(("name", name));
    start.push_attribute(("content", content));
    w.write_event(Event::Empty(start))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))
}
