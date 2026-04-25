use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

/// CBZ/CBR metadata extraction.
/// CBZ: count pages (ZIP entries that are images).
/// CBR: count pages from RAR central directory (not fully parsed; use filename heuristic).
/// Series and issue number are detected from common filename patterns:
///   "Title 001.cbz", "Title Vol.1 #001.cbr", "Title (2020) 001.cbz"
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    let (series, issue) = parse_comic_filename(filename);
    let title = match (series.as_ref(), issue) {
        (Some(series), Some(number)) => Some(format!("{} #{}", series, number as u32)),
        (Some(series), None) => Some(series.clone()),
        _ => Some(filename.replace(['_', '-'], " ")),
    };

    let mut description = None;
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    if extension == "cbz" {
        if let Ok(file) = std::fs::File::open(path) {
            if let Ok(mut archive) = zip::ZipArchive::new(std::io::BufReader::new(file)) {
                let page_count = (0..archive.len())
                    .filter(|index| {
                        archive
                            .by_index(*index)
                            .map(|entry| {
                                let lower = entry.name().to_lowercase();
                                lower.ends_with(".jpg")
                                    || lower.ends_with(".jpeg")
                                    || lower.ends_with(".png")
                                    || lower.ends_with(".webp")
                            })
                            .unwrap_or(false)
                    })
                    .count();
                if page_count > 0 {
                    description = Some(format!("{} pages", page_count));
                }
            }
        }
    }

    Ok(BookMetadata {
        title,
        series,
        series_index: issue,
        description,
        ..BookMetadata::default()
    })
}

fn parse_comic_filename(name: &str) -> (Option<String>, Option<f32>) {
    let chars: Vec<char> = name.chars().collect();
    let mut last_digit_start = None;
    let mut last_digit_end = None;
    let mut i = chars.len();

    while i > 0 {
        i -= 1;
        if chars[i].is_ascii_digit() {
            if last_digit_end.is_none() {
                last_digit_end = Some(i + 1);
            }
            last_digit_start = Some(i);
        } else if last_digit_end.is_some() {
            break;
        }
    }

    match (last_digit_start, last_digit_end) {
        (Some(start), Some(end)) => {
            let issue_str: String = chars[start..end].iter().collect();
            let issue = issue_str.parse::<f32>().ok();
            let prefix: String = chars[..start]
                .iter()
                .collect::<String>()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .trim()
                .to_string();
            let series = if prefix.is_empty() { None } else { Some(prefix) };
            (series, issue)
        }
        _ => (
            Some(name.replace(['_', '-'], " ").trim().to_string()),
            None,
        ),
    }
}
