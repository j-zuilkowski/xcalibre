use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

/// CHM text extraction via `chmlib` ITSF container traversal.
/// Iterates all objects in the CHM with `.htm` or `.html` extensions,
/// strips HTML tags, and concatenates the result.
/// Falls back to recover_readable_text() on any parse failure.
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let mut all_text = String::new();

    // Attempt ITSF container parse.
    if let Ok(mut chm) = chmlib::ChmFile::open(path) {
        // Collect HTML file paths first (can't mutate chm during for_each iteration
        // since the callback receives &mut ChmFile).
        let mut html_paths: Vec<std::path::PathBuf> = Vec::new();

        let _ = chm.for_each(chmlib::Filter::all(), |_chm, unit| {
            if unit.is_file() {
                if let Some(p) = unit.path() {
                    let name = p.to_string_lossy().to_lowercase();
                    if name.ends_with(".htm") || name.ends_with(".html") {
                        html_paths.push(p.to_path_buf());
                    }
                }
            }
            chmlib::Continuation::Continue
        });

        // Now read each HTML file
        for ref html_path in &html_paths {
            if let Some(unit) = chm.find(html_path) {
                let buf_len = unit.length() as usize;
                if buf_len > 0 && buf_len < 10_000_000 {
                    let mut buf = vec![0u8; buf_len];
                    if chm.read(&unit, 0, &mut buf).is_ok() {
                        if let Ok(html_str) = std::str::from_utf8(&buf) {
                            let stripped = strip_html_tags(html_str);
                            if !stripped.trim().is_empty() {
                                if !all_text.is_empty() {
                                    all_text.push('\n');
                                }
                                all_text.push_str(&stripped);
                            }
                        }
                    }
                }
            }
        }
    }

    // Fall back to heuristic if container traversal produced nothing.
    if all_text.trim().is_empty() {
        all_text = crate::utils::recover::recover_readable_text(path)?;
    }

    let word_count = all_text.split_whitespace().count();
    Ok(ExtractedText {
        full_text: all_text,
        word_count,
    })
}

fn strip_html_tags(html: &str) -> String {
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    let stripped = re.replace_all(html, " ");
    stripped
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
