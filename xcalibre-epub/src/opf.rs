use crate::EpubError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EpubOPF {
    pub title:        Option<String>,
    pub authors:      Vec<String>,
    pub language:     Option<String>,
    pub publisher:    Option<String>,
    pub series:       Option<String>,
    pub series_index: Option<f32>,
    pub tags:         Vec<String>,
    pub epub_version: String,
    pub manifest:     Vec<ManifestItem>,
    pub spine:        Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestItem {
    pub id:         String,
    pub href:       String,
    pub media_type: String,
    pub properties: Option<String>,
}

impl EpubOPF {
    pub fn parse(xml: &str) -> Result<Self, EpubError> {
        let doc = roxmltree::Document::parse(xml)
            .map_err(|e| EpubError::Xml(e.to_string()))?;

        let package = doc.root_element();
        let version = package.attribute("version").unwrap_or("2.0").to_string();

        let mut opf = EpubOPF {
            epub_version: version,
            ..Default::default()
        };

        for node in doc.descendants() {
            match node.tag_name().name() {
                "title" => {
                    opf.title = node.text().map(str::trim).map(String::from);
                }
                "creator" => {
                    let author = node.text().unwrap_or("").trim().to_string();
                    if !author.is_empty() {
                        opf.authors.push(author);
                    }
                }
                "language" => {
                    opf.language = node.text().map(str::trim).map(String::from);
                }
                "publisher" => {
                    opf.publisher = node.text().map(str::trim).map(String::from);
                }
                "subject" => {
                    let tag = node.text().unwrap_or("").trim().to_string();
                    if !tag.is_empty() {
                        opf.tags.push(tag);
                    }
                }
                "item" => {
                    if let (Some(id), Some(href)) = (node.attribute("id"), node.attribute("href")) {
                        let mt = node.attribute("media-type").unwrap_or("application/octet-stream");
                        let props = node.attribute("properties").map(String::from);
                        opf.manifest.push(ManifestItem {
                            id: id.to_string(),
                            href: href.to_string(),
                            media_type: mt.to_string(),
                            properties: props,
                        });
                    }
                }
                "itemref" => {
                    if let Some(idref) = node.attribute("idref") {
                        opf.spine.push(idref.to_string());
                    }
                }
                "meta" => {
                    let name = node.attribute("name").unwrap_or("");
                    let content = node.attribute("content").unwrap_or("");
                    match name {
                        "calibre:series" => opf.series = Some(content.to_string()),
                        "calibre:series_index" => {
                            opf.series_index = content.parse().ok();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(opf)
    }

    pub fn to_xml(&self) -> Result<String, EpubError> {
        use quick_xml::{events::*, Writer};
        use std::io::Cursor;

        let mut w = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

        w.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))
            .map_err(|e| EpubError::Xml(e.to_string()))?;

        let mut pkg = BytesStart::new("package");
        pkg.push_attribute(("xmlns", "http://www.idpf.org/2007/opf"));
        pkg.push_attribute(("version", self.epub_version.as_str()));
        pkg.push_attribute(("unique-identifier", "uid"));
        w.write_event(Event::Start(pkg))
            .map_err(|e| EpubError::Xml(e.to_string()))?;

        let mut md = BytesStart::new("metadata");
        md.push_attribute(("xmlns:dc", "http://purl.org/dc/elements/1.1/"));
        md.push_attribute(("xmlns:opf", "http://www.idpf.org/2007/opf"));
        w.write_event(Event::Start(md))
            .map_err(|e| EpubError::Xml(e.to_string()))?;

        write_dc(&mut w, "title", self.title.as_deref().unwrap_or("Unknown"))?;
        for author in &self.authors {
            write_dc_attr(&mut w, "creator", author, &[("opf:role", "aut")])?;
        }
        if let Some(lang) = &self.language {
            write_dc(&mut w, "language", lang)?;
        }
        if let Some(pub_) = &self.publisher {
            write_dc(&mut w, "publisher", pub_)?;
        }
        if let Some(s) = &self.series {
            write_meta(&mut w, "calibre:series", s)?;
            if let Some(idx) = self.series_index {
                write_meta(&mut w, "calibre:series_index", &idx.to_string())?;
            }
        }
        for tag in &self.tags {
            write_dc(&mut w, "subject", tag)?;
        }

        w.write_event(Event::End(BytesEnd::new("metadata")))
            .map_err(|e| EpubError::Xml(e.to_string()))?;

        // manifest
        w.write_event(Event::Start(BytesStart::new("manifest")))
            .map_err(|e| EpubError::Xml(e.to_string()))?;
        for item in &self.manifest {
            let mut el = BytesStart::new("item");
            el.push_attribute(("id", item.id.as_str()));
            el.push_attribute(("href", item.href.as_str()));
            el.push_attribute(("media-type", item.media_type.as_str()));
            if let Some(ref props) = item.properties {
                el.push_attribute(("properties", props.as_str()));
            }
            w.write_event(Event::Empty(el))
                .map_err(|e| EpubError::Xml(e.to_string()))?;
        }
        w.write_event(Event::End(BytesEnd::new("manifest")))
            .map_err(|e| EpubError::Xml(e.to_string()))?;

        // spine
        w.write_event(Event::Start(BytesStart::new("spine")))
            .map_err(|e| EpubError::Xml(e.to_string()))?;
        for idref in &self.spine {
            let mut el = BytesStart::new("itemref");
            el.push_attribute(("idref", idref.as_str()));
            w.write_event(Event::Empty(el))
                .map_err(|e| EpubError::Xml(e.to_string()))?;
        }
        w.write_event(Event::End(BytesEnd::new("spine")))
            .map_err(|e| EpubError::Xml(e.to_string()))?;

        w.write_event(Event::End(BytesEnd::new("package")))
            .map_err(|e| EpubError::Xml(e.to_string()))?;

        let xml = String::from_utf8(w.into_inner().into_inner())
            .map_err(|e| EpubError::Xml(e.to_string()))?;
        Ok(xml)
    }
}

fn write_dc(w: &mut quick_xml::Writer<std::io::Cursor<Vec<u8>>>, name: &str, text: &str) -> Result<(), EpubError> {
    let tag = format!("dc:{}", name);
    w.write_event(quick_xml::events::Event::Start(quick_xml::events::BytesStart::new(&tag)))
        .map_err(|e| EpubError::Xml(e.to_string()))?;
    w.write_event(quick_xml::events::Event::Text(quick_xml::events::BytesText::new(text)))
        .map_err(|e| EpubError::Xml(e.to_string()))?;
    w.write_event(quick_xml::events::Event::End(quick_xml::events::BytesEnd::new(&tag)))
        .map_err(|e| EpubError::Xml(e.to_string()))
}

fn write_dc_attr(w: &mut quick_xml::Writer<std::io::Cursor<Vec<u8>>>, name: &str, text: &str, attrs: &[(&str, &str)]) -> Result<(), EpubError> {
    let tag = format!("dc:{}", name);
    let mut start = quick_xml::events::BytesStart::new(&tag);
    for (k, v) in attrs {
        start.push_attribute((*k, *v));
    }
    w.write_event(quick_xml::events::Event::Start(start))
        .map_err(|e| EpubError::Xml(e.to_string()))?;
    w.write_event(quick_xml::events::Event::Text(quick_xml::events::BytesText::new(text)))
        .map_err(|e| EpubError::Xml(e.to_string()))?;
    w.write_event(quick_xml::events::Event::End(quick_xml::events::BytesEnd::new(&tag)))
        .map_err(|e| EpubError::Xml(e.to_string()))
}

fn write_meta(w: &mut quick_xml::Writer<std::io::Cursor<Vec<u8>>>, name: &str, content: &str) -> Result<(), EpubError> {
    let mut start = quick_xml::events::BytesStart::new("meta");
    start.push_attribute(("name", name));
    start.push_attribute(("content", content));
    w.write_event(quick_xml::events::Event::Empty(start))
        .map_err(|e| EpubError::Xml(e.to_string()))
}
