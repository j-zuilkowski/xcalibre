use crate::error::ProcessingError;
use std::path::Path;

pub fn recover_readable_text(path: &Path) -> Result<String, ProcessingError> {
    let bytes = std::fs::read(path).map_err(ProcessingError::IoError)?;

    let mut candidates = vec![
        normalize_text(&String::from_utf8_lossy(&bytes)),
        normalize_text(&decode_utf16_le(&bytes)),
    ];

    if let Some(ascii) = recover_ascii_strings(&bytes) {
        candidates.push(normalize_text(&ascii));
    }

    candidates.retain(|candidate| !candidate.trim().is_empty());
    candidates.sort_by_key(|candidate| candidate.len());
    Ok(candidates.pop().unwrap_or_default())
}

pub fn recover_title(path: &Path) -> Result<Option<String>, ProcessingError> {
    let bytes = std::fs::read(path).map_err(ProcessingError::IoError)?;
    let mut candidates = Vec::new();

    if let Some(ascii) = recover_ascii_strings(&bytes) {
        candidates.extend(ascii.lines().map(|line| line.trim().to_string()));
    }

    let utf16 = decode_utf16_le(&bytes);
    candidates.extend(utf16.lines().map(|line| line.trim().to_string()));

    if let Some(best) = candidates
        .into_iter()
        .filter(|line| !line.is_empty())
        .min_by_key(|line| score_title_line(line))
    {
        let recovered = best.chars().take(120).collect::<String>();
        if !recovered.trim().is_empty() {
            return Ok(Some(recovered));
        }
    }

    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let stem = stem.replace(['_', '-'], " ");
    let trimmed = stem.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

fn decode_utf16_le(bytes: &[u8]) -> String {
    let mut words = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        words.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    String::from_utf16_lossy(&words)
}

fn recover_ascii_strings(bytes: &[u8]) -> Option<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for &byte in bytes {
        let ch = byte as char;
        if ch.is_ascii_graphic() || ch == ' ' {
            current.push(ch);
        } else if !current.trim().is_empty() {
            lines.push(current.trim().to_string());
            current.clear();
        } else {
            current.clear();
        }
    }

    if !current.trim().is_empty() {
        lines.push(current.trim().to_string());
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn normalize_text(text: &str) -> String {
    text.chars()
        .map(|ch| if ch.is_control() && !ch.is_whitespace() { ' ' } else { ch })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn score_title_line(line: &str) -> (usize, usize, usize) {
    let word_count = line.split_whitespace().count();
    let title_like_penalty = if (2..=6).contains(&word_count) { 0 } else { 1 };
    let punctuation_penalty = if line.ends_with('.') || line.ends_with(':') || line.ends_with(';') {
        1
    } else {
        0
    };
    let length = line.len();
    (title_like_penalty + punctuation_penalty, word_count, length)
}
