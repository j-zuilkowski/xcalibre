use crate::error::ProcessingError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use xcalibre_epub::Container;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookStats {
    pub word_count: u32,
    pub character_count: u32,
    pub page_count_estimate: u32,
    pub reading_time_minutes: u32,
}

pub fn compute_book_stats(epub_path: &Path) -> Result<BookStats, ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let spine = container.spine_hrefs();
    let mut full_text = String::new();

    for href in &spine {
        if let Ok(data) = container.read_item(href) {
            let html = String::from_utf8_lossy(&data);
            let text = strip_tags(&html);
            full_text.push_str(&text);
            full_text.push(' ');
        }
    }

    let word_count = full_text.split_whitespace().count() as u32;
    let character_count = full_text.chars().filter(|c| !c.is_whitespace()).count() as u32;
    let page_count_estimate = ((word_count as f64 / 250.0).ceil() as u32).max(1);
    let reading_time_minutes = ((word_count as f64 / 238.0).ceil() as u32).max(1);

    Ok(BookStats {
        word_count,
        character_count,
        page_count_estimate,
        reading_time_minutes,
    })
}

fn strip_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}
