use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub chunk_index:     usize,
    pub chunk_text:      String,
    pub relevance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitedResponse {
    pub response_text: String,
    pub citations:     Vec<Citation>,
}

/// Extract citations by computing word-overlap between the response and each chunk.
/// Returns chunks whose overlap score exceeds a minimum threshold.
pub fn extract_citations(response: &str, chunks: &[&str]) -> CitedResponse {
    const MIN_SCORE: f32 = 0.05;

    let resp_words: std::collections::HashSet<String> = tokenize(response);

    let mut citations: Vec<Citation> = chunks
        .iter()
        .enumerate()
        .filter_map(|(idx, chunk)| {
            let chunk_words: std::collections::HashSet<String> = tokenize(chunk);
            if chunk_words.is_empty() || resp_words.is_empty() {
                return None;
            }

            let overlap = resp_words.intersection(&chunk_words).count();
            let score = overlap as f32 / resp_words.len().max(chunk_words.len()) as f32;

            if score >= MIN_SCORE {
                Some(Citation {
                    chunk_index:     idx,
                    chunk_text:      chunk.to_string(),
                    relevance_score: score.min(1.0),
                })
            } else {
                None
            }
        })
        .collect();

    citations.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

    CitedResponse {
        response_text: response.to_string(),
        citations,
    }
}

fn tokenize(text: &str) -> std::collections::HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3)
        .map(|w| w.to_string())
        .collect()
}
