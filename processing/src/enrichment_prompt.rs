use crate::metadata::enrichment::EnrichmentSuggestion;
use std::io::{self, BufRead, Write};

pub fn prompt_user(suggestions: &[EnrichmentSuggestion]) -> Vec<EnrichmentSuggestion> {
    if suggestions.is_empty() {
        return vec![];
    }
    println!("\nEnrichment suggestions:");
    for (i, s) in suggestions.iter().enumerate() {
        println!(
            "  [{}] {} → \"{}\"  (source: {})",
            i + 1,
            s.field,
            s.value,
            s.source
        );
    }
    print!("Accept all (a), pick individually (p), or skip (s)? ");
    io::stdout().flush().ok();
    let stdin = io::stdin();
    let line = stdin
        .lock()
        .lines()
        .next()
        .unwrap_or(Ok(String::new()))
        .unwrap_or_default();
    match line.trim() {
        "a" | "A" => suggestions.to_vec(),
        "p" | "P" => pick_individually(suggestions),
        _         => vec![],
    }
}

fn pick_individually(suggestions: &[EnrichmentSuggestion]) -> Vec<EnrichmentSuggestion> {
    let mut accepted = Vec::new();
    let stdin = io::stdin();
    for s in suggestions {
        print!("  Accept \"{}\" for {}? (y/n) ", s.value, s.field);
        io::stdout().flush().ok();
        let line = stdin
            .lock()
            .lines()
            .next()
            .unwrap_or(Ok(String::new()))
            .unwrap_or_default();
        if line.trim().eq_ignore_ascii_case("y") {
            accepted.push(s.clone());
        }
    }
    accepted
}