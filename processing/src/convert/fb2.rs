use crate::convert::txt::strip_html_to_text;
use crate::error::ProcessingError;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use std::io::{BufWriter, Write};
use std::path::Path;
use xcalibre_epub::Container;

pub fn epub_to_fb2(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let (title, authors) = read_opf_metadata(&container);
    let spine = container.spine_hrefs();

    let file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let mut writer = Writer::new_with_indent(BufWriter::new(file), b' ', 2);

    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let mut fb_start = BytesStart::new("FictionBook");
    fb_start.push_attribute(("xmlns", "http://www.gribuser.ru/xml/fictionbook/2.0"));
    fb_start.push_attribute(("xmlns:l", "http://www.w3.org/1999/xlink"));
    writer.write_event(Event::Start(fb_start))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    writer.write_event(Event::Start(BytesStart::new("description")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    writer.write_event(Event::Start(BytesStart::new("title-info")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    writer.write_event(Event::Start(BytesStart::new("book-title")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    writer.write_event(Event::Text(BytesText::new(&title)))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    writer.write_event(Event::End(BytesEnd::new("book-title")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    for author in &authors {
        writer.write_event(Event::Start(BytesStart::new("author")))
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        writer.write_event(Event::Start(BytesStart::new("nickname")))
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        writer.write_event(Event::Text(BytesText::new(author)))
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        writer.write_event(Event::End(BytesEnd::new("nickname")))
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        writer.write_event(Event::End(BytesEnd::new("author")))
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    }

    writer.write_event(Event::End(BytesEnd::new("title-info")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    writer.write_event(Event::End(BytesEnd::new("description")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    writer.write_event(Event::Start(BytesStart::new("body")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    for (idx, href) in spine.iter().enumerate() {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let html = String::from_utf8_lossy(&bytes);
        let text = strip_html_to_text(&html);

        let mut sec = BytesStart::new("section");
        sec.push_attribute(("id", format!("ch{idx}").as_str()));
        writer.write_event(Event::Start(sec))
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

        for para in text.split("\n\n").filter(|p| !p.trim().is_empty()) {
            writer.write_event(Event::Start(BytesStart::new("p")))
                .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
            writer.write_event(Event::Text(BytesText::new(para.trim())))
                .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
            writer.write_event(Event::End(BytesEnd::new("p")))
                .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        }

        writer.write_event(Event::End(BytesEnd::new("section")))
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    }

    writer.write_event(Event::End(BytesEnd::new("body")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    writer.write_event(Event::End(BytesEnd::new("FictionBook")))
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    writer.into_inner().flush()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}

fn read_opf_metadata(container: &Container) -> (String, Vec<String>) {
    let opf_path = container.opf_path().to_string();
    if let Ok(bytes) = container.read_item(&opf_path) {
        if let Ok(xml) = String::from_utf8(bytes) {
            if let Ok(opf) = xcalibre_epub::opf::EpubOPF::parse(&xml) {
                let title = opf.title.unwrap_or_else(|| "Untitled".to_string());
                return (title, opf.authors);
            }
        }
    }
    ("Untitled".to_string(), vec![])
}
