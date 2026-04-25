use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let data = std::fs::read(path).map_err(ProcessingError::IoError)?;

    let book = mobi::Mobi::new(&data)
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let html = book.content_as_string_lossy();

    let plain = strip_html(&html);
    let word_count = plain.split_whitespace().count();
    Ok(ExtractedText { full_text: plain, word_count })
}

fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
