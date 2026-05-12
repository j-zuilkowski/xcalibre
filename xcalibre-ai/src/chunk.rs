/// Configuration for the chunking algorithm.
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Maximum tokens per chunk (approximate: 1 token ≈ 4 chars).
    pub max_tokens:     usize,
    /// Number of tokens to overlap between consecutive chunks.
    pub overlap_tokens: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self { Self { max_tokens: 512, overlap_tokens: 64 } }
}

/// Split text into overlapping chunks, respecting sentence boundaries where possible.
pub fn chunk_text(text: &str, cfg: &ChunkConfig) -> Vec<String> {
    if text.trim().is_empty() { return vec![]; }

    let max_chars     = cfg.max_tokens * 4;
    let overlap_chars = cfg.overlap_tokens * 4;

    // Split into sentences first
    let sentences = split_sentences(text);
    // Then split long sentences into sub-pieces at word boundaries
    let pieces = split_long_sentences(&sentences, max_chars);

    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();

    for piece in &pieces {
        if current.len() + piece.len() > max_chars && !current.is_empty() {
            chunks.push(current.trim().to_string());
            // Start next chunk with overlap from the end of the previous chunk
            let tail = last_chars(&current, overlap_chars);
            current = tail;
        }
        current.push_str(piece);
        current.push(' ');
    }
    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }
    chunks.into_iter().filter(|c| !c.is_empty()).collect()
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut buf = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        buf.push(chars[i]);
        if matches!(chars[i], '.' | '!' | '?') && i + 1 < chars.len() && chars[i + 1] == ' '  {
                sentences.push(buf.clone());
                buf.clear();
                i += 2;
                continue;
            }
        i += 1;
    }
    if !buf.trim().is_empty() { sentences.push(buf); }
    sentences
}

fn split_long_sentences(sentences: &[String], max_chars: usize) -> Vec<String> {
    let mut result = Vec::new();
    for s in sentences {
        if s.len() <= max_chars {
            result.push(s.clone());
        } else {
            // Split at word boundaries
            let words: Vec<&str> = s.split_whitespace().collect();
            let mut piece = String::new();
            for w in words {
                if piece.len() + w.len() > max_chars && !piece.is_empty() {
                    result.push(piece.trim().to_string());
                    piece = String::new();
                }
                piece.push_str(w);
                piece.push(' ');
            }
            if !piece.trim().is_empty() {
                result.push(piece.trim().to_string());
            }
        }
    }
    result
}

fn last_chars(s: &str, n: usize) -> String {
    if s.len() <= n { return s.to_string(); }
    let start = s.len() - n;
    // Find a word boundary
    let boundary = s[start..].find(' ').map(|p| start + p + 1).unwrap_or(start);
    s[boundary..].to_string()
}
