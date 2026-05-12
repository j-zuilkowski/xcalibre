use crate::error::ProcessingError;
use std::io::Write;
use std::path::Path;
use xcalibre_epub::Container;

pub fn epub_to_html(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let spine = container.spine_hrefs();

    let title = "Untitled"; // fallback

    let mut body_parts: Vec<String> = Vec::new();

    for href in &spine {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let fragment = String::from_utf8_lossy(&bytes);
        // Extract <body> content if present, otherwise use whole item
        let content = extract_body_content(&fragment).unwrap_or_else(|| fragment.to_string());
        body_parts.push(content);
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<style>
  body {{ font-family: Georgia, serif; max-width: 800px; margin: 0 auto; padding: 2rem; line-height: 1.6; }}
  h1, h2, h3 {{ margin-top: 2rem; }}
  p {{ margin: 0.8rem 0; }}
</style>
</head>
<body>
{body}
</body>
</html>"#,
        title = escape_html(title),
        body = body_parts.join("\n<hr>\n"),
    );

    let mut file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    file.write_all(html.as_bytes())
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    Ok(())
}

fn extract_body_content(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let body_start = lower.find("<body")?;
    let content_start = html[body_start..].find('>')? + body_start + 1;
    let body_end = lower.rfind("</body>")?;
    Some(html[content_start..body_end].to_string())
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}
