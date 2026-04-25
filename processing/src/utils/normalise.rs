pub fn normalise(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}