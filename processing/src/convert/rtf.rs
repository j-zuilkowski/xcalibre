use crate::convert::txt::strip_html_to_text;
use crate::error::ProcessingError;
use std::io::Write;
use std::path::Path;
use xcalibre_epub::Container;

pub fn epub_to_rtf(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let title = read_title(&container);
    let spine = container.spine_hrefs();

    let mut rtf = String::new();

    rtf.push_str("{\\rtf1\\ansi\\ansicpg1252\\deff0\n");
    rtf.push_str("{\\fonttbl{\\f0\\froman\\fcharset0 Times New Roman;}}\n");
    rtf.push_str("{\\colortbl;\\red0\\green0\\blue0;}\n");
    rtf.push_str("\\widowctrl\\hyphauto\n");

    rtf.push_str(&format!(
        "{{\\pard\\sb240\\sa120\\b\\fs36 {}\\par}}\n",
        rtf_escape(&title)
    ));

    for href in &spine {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let html = String::from_utf8_lossy(&bytes);
        let text = strip_html_to_text(&html);

        for para in text.split("\n\n").filter(|p| !p.trim().is_empty()) {
            rtf.push_str(&format!(
                "{{\\pard\\sb0\\sa120\\fs24 {}\\par}}\n",
                rtf_escape(para.trim())
            ));
        }
    }

    rtf.push('}');

    let mut file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    file.write_all(rtf.as_bytes())
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}

fn read_title(container: &Container) -> String {
    let opf_path = container.opf_path().to_string();
    if let Ok(bytes) = container.read_item(&opf_path) {
        if let Ok(xml) = String::from_utf8(bytes) {
            if let Ok(opf) = xcalibre_epub::opf::EpubOPF::parse(&xml) {
                if let Some(t) = opf.title {
                    return t;
                }
            }
        }
    }
    "Untitled".to_string()
}

fn rtf_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '{'  => out.push_str("\\{"),
            '}'  => out.push_str("\\}"),
            c if c.is_ascii() => out.push(c),
            c    => out.push_str(&format!("\\u{}?", c as u32)),
        }
    }
    out
}
