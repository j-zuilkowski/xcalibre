use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

/// RTF metadata is rarely structured. Extract title from the \title control word.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let raw = std::fs::read_to_string(path).map_err(ProcessingError::IoError)?;
    let authors = extract_control_value(&raw, r"\author")
        .map(|a| a.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    let meta = BookMetadata {
        title: extract_control_value(&raw, r"\title"),
        authors,
        ..BookMetadata::default()
    };
    Ok(meta)
}

fn extract_control_value(rtf: &str, control: &str) -> Option<String> {
    let pos = rtf.find(control)?;
    let after = &rtf[pos + control.len()..];
    let start = after.find('{')? + 1;
    let end = after[start..].find('}')? + start;
    let raw = &after[start..end];
    let clean = strip_rtf_markup(raw).trim().to_string();
    if clean.is_empty() { None } else { Some(clean) }
}

fn strip_rtf_markup(s: &str) -> String {
    let mut out = String::new();
    let mut skip = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => { skip = true; }
            ' ' | '\n' | '\r' if skip => { skip = false; }
            _ if skip && c.is_alphabetic() => {
                while chars.peek().is_some_and(|x| x.is_alphanumeric() || *x == '-') {
                    chars.next();
                }
                skip = false;
            }
            '{' | '}' => {}
            _ => { skip = false; out.push(c); }
        }
    }
    out
}
