use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let doc = match lopdf::Document::load(path) {
        Ok(doc) => doc,
        Err(_) => {
            return Ok(ExtractedText {
                full_text: String::new(),
                word_count: 0,
            })
        }
    };

    let mut parts: Vec<String> = vec![];

    for page_id in doc.page_iter() {
        if let Ok(content) = doc.get_and_decode_page_content(page_id) {
            let mut page_text = String::new();
            for op in content.operations {
                match op.operator.as_str() {
                    "Tj" | "TJ" | "'" | "\"" => {
                        for operand in &op.operands {
                            match operand {
                                lopdf::Object::String(bytes, _) => {
                                    if let Ok(s) = std::str::from_utf8(bytes) {
                                        page_text.push_str(s);
                                        page_text.push(' ');
                                    }
                                }
                                lopdf::Object::Array(arr) => {
                                    for item in arr {
                                        if let lopdf::Object::String(bytes, _) = item {
                                            if let Ok(s) = std::str::from_utf8(bytes) {
                                                page_text.push_str(s);
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            let trimmed = page_text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !trimmed.is_empty() {
                parts.push(trimmed);
            }
        }
    }

    let full_text  = parts.join("\n\n");
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}
