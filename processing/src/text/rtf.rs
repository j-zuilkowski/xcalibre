use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let raw = std::fs::read_to_string(path).map_err(ProcessingError::IoError)?;
    let full_text = strip_rtf(&raw);
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}

/// Strip all RTF control words and return plain text.
fn strip_rtf(rtf: &str) -> String {
    let mut out = String::with_capacity(rtf.len() / 2);
    let mut chars = rtf.chars().peekable();
    let mut depth = 0i32;
    let mut skip_group = 0i32;

    while let Some(c) = chars.next() {
        match c {
            '{' => {
                depth += 1;
                if chars.peek() == Some(&'\\') {
                    let mut peek_iter = rtf.chars();
                    let _ = peek_iter.next();
                    if peek_iter.next() == Some('*') {
                        skip_group += 1;
                    }
                }
            }
            '}' => {
                if skip_group > 0 { skip_group -= 1; }
                depth -= 1;
            }
            '\\' => {
                match chars.peek() {
                    Some(&'\n') | Some(&'\r') => { out.push('\n'); chars.next(); }
                    Some(&'\\') => { if skip_group == 0 { out.push('\\'); } chars.next(); }
                    Some(&'{')  => { if skip_group == 0 { out.push('{'); } chars.next(); }
                    Some(&'}')  => { if skip_group == 0 { out.push('}'); } chars.next(); }
                    Some(&c2) if c2.is_alphabetic() => {
                        let mut word = String::new();
                        while chars.peek().is_some_and(|x| x.is_alphanumeric() || *x == '-') {
                            if let Some(ch) = chars.next() {
                                word.push(ch);
                            }
                        }
                        if chars.peek() == Some(&' ') { chars.next(); }
                        match word.as_str() {
                            "par" | "line" | "page" if skip_group == 0 => { out.push('\n'); }
                            "tab" if skip_group == 0 => { out.push('\t'); }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ if skip_group == 0 && depth > 0 => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
