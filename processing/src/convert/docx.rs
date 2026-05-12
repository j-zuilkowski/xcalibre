use crate::error::ProcessingError;
use std::path::Path;
use xcalibre_epub::Container;

pub fn epub_to_docx(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let spine = container.spine_hrefs();

    let mut doc = docx_rs::Docx::new();

    for href in &spine {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let html = String::from_utf8_lossy(&bytes);
        let paragraphs = html_to_paragraphs(&html);

        for para_text in paragraphs {
            if para_text.trim().is_empty() { continue; }
            let run = docx_rs::Run::new().add_text(&para_text);
            let para = docx_rs::Paragraph::new().add_run(run);
            doc = doc.add_paragraph(para);
        }
    }

    let xml_docx = doc.build();
    let file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    xml_docx.pack(file)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    Ok(())
}

fn html_to_paragraphs(html: &str) -> Vec<String> {
    let para_re = regex::Regex::new(r"<(?:p|div|h[1-6]|li|blockquote)[^>]*>(.*?)</(?:p|div|h[1-6]|li|blockquote)>").unwrap();
    let tag_re  = regex::Regex::new(r"<[^>]+>").unwrap();

    let mut result: Vec<String> = Vec::new();
    for cap in para_re.captures_iter(html) {
        let inner = &cap[1];
        let text  = tag_re.replace_all(inner, "");
        let text  = decode_html_entities(&text);
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            result.push(trimmed);
        }
    }

    // Fallback: if no block tags found, strip all tags
    if result.is_empty() {
        let stripped = tag_re.replace_all(html, " ");
        let text = decode_html_entities(&stripped);
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            result.push(trimmed);
        }
    }

    result
}

fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;",  "&")
     .replace("&lt;",   "<")
     .replace("&gt;",   ">")
     .replace("&quot;", "\"")
     .replace("&#39;",  "'")
     .replace("&nbsp;", " ")
}
