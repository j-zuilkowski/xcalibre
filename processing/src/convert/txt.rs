use crate::error::ProcessingError;
use std::io::Write;
use std::path::Path;
use xcalibre_epub::Container;

pub fn epub_to_txt(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let spine = container.spine_hrefs();

    let mut parts: Vec<String> = Vec::new();

    for href in &spine {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let html = String::from_utf8_lossy(&bytes);
        let text = strip_html_tags(&html);
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }

    let output = parts.join("\n\n");

    let mut file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    file.write_all(output.as_bytes())
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    Ok(())
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;

    let mut chars = html.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                in_tag = true;
                // Peek ahead for block-level tags to insert newlines
                let mut tag_buf = String::new();
                let mut tmp = chars.clone();
                while let Some(&c) = tmp.peek() {
                    if c == '>' || c == ' ' { break; }
                    tag_buf.push(c);
                    tmp.next();
                }
                let tag_lower = tag_buf.to_lowercase();
                if matches!(tag_lower.trim_start_matches('/'),
                    "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" |
                    "br" | "li" | "blockquote" | "tr")
                {
                    result.push('\n');
                }
                if tag_lower == "script" { in_script = true; }
                if tag_lower == "style"  { in_style  = true; }
                if tag_lower == "/script" { in_script = false; }
                if tag_lower == "/style"  { in_style  = false; }
            }
            '>' => {
                in_tag = false;
            }
            _ if in_tag || in_script || in_style => {}
            _ => result.push(ch),
        }
    }

    // Collapse multiple blank lines
    let mut collapsed = String::new();
    let mut blank_count = 0u32;
    for line in result.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 { collapsed.push('\n'); }
        } else {
            blank_count = 0;
            collapsed.push_str(line);
            collapsed.push('\n');
        }
    }
    collapsed
}
