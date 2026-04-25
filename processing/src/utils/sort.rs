static ARTICLES: &[&str] = &["the ", "a ", "an "];

/// Convert a display title to a sortable form.
/// "The Lord of the Rings" → "Lord of the Rings, The"
pub fn title_sort(title: &str) -> String {
    let lower = title.to_lowercase();
    for article in ARTICLES {
        if let Some(_rest) = lower.strip_prefix(article) {
            let split = article.len();
            let article_trimmed = title[..split].trim_end();
            let rest_display = title[split..].trim_start();
            return format!("{}, {}", rest_display, article_trimmed);
        }
    }
    title.to_string()
}

/// Convert an author name to sortable "Last, First" form.
/// "J.R.R. Tolkien" → "Tolkien, J.R.R."
/// "Tolkien, J.R.R." is returned unchanged.
pub fn author_sort(name: &str) -> String {
    if name.contains(',') {
        return name.to_string();
    }
    let parts: Vec<&str> = name.split_whitespace().collect();
    match parts.len() {
        0 => String::new(),
        1 => parts[0].to_string(),
        _ => {
            let last = parts[parts.len() - 1];
            let first = parts[..parts.len() - 1].join(" ");
            format!("{}, {}", last, first)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_sort_article() {
        assert_eq!(title_sort("The Great Gatsby"), "Great Gatsby, The");
    }

    #[test]
    fn test_title_sort_no_article() {
        assert_eq!(title_sort("Dune"), "Dune");
    }

    #[test]
    fn test_author_sort_two_names() {
        assert_eq!(author_sort("Frank Herbert"), "Herbert, Frank");
    }

    #[test]
    fn test_author_sort_already_sorted() {
        assert_eq!(author_sort("Herbert, Frank"), "Herbert, Frank");
    }
}
