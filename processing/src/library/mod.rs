use std::path::{Path, PathBuf};

/// Derive the managed on-disk path for a book.
/// Format: `<base>/<Author Last, First>/<Title (ID)>/`
/// Sanitizes path-unsafe characters to underscores.
pub fn managed_path(base: &Path, authors: &str, title: &str, book_id: i64) -> PathBuf {
    let author_dir = format_author(authors);
    let title_dir  = format_title(title, book_id);
    base.join(sanitize(&author_dir)).join(sanitize(&title_dir))
}

fn format_author(authors: &str) -> String {
    // Use first author; convert "First Last" → "Last, First"
    let primary = authors
        .split(['&', ','])
        .next()
        .unwrap_or(authors)
        .trim();
    let parts: Vec<&str> = primary.splitn(2, ' ').collect();
    if parts.len() == 2 {
        format!("{}, {}", parts[1].trim(), parts[0].trim())
    } else {
        primary.to_string()
    }
}

fn format_title(title: &str, id: i64) -> String {
    format!("{} ({})", title.trim(), id)
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn format_author_single() {
        assert_eq!(format_author("Terry Pratchett"), "Pratchett, Terry");
    }
    #[test]
    fn format_author_already_last_first() {
        assert_eq!(format_author("Pratchett"), "Pratchett");
    }
}
