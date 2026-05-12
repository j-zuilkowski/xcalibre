use xcalibre_ai::{chunk_text, ChunkConfig};

#[test]
fn test_chunk_empty_text() {
    let chunks = chunk_text("", &ChunkConfig::default());
    assert!(chunks.is_empty());
}

#[test]
fn test_chunk_short_text_is_single_chunk() {
    let text = "This is a short paragraph.";
    let chunks = chunk_text(text, &ChunkConfig::default());
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].trim(), text.trim());
}

#[test]
fn test_chunk_splits_at_token_boundary() {
    // Generate text long enough to require multiple chunks at default 512-token limit
    let long = "word ".repeat(600);
    let chunks = chunk_text(&long, &ChunkConfig::default());
    assert!(chunks.len() >= 2, "600 words must produce at least 2 chunks");
}

#[test]
fn test_chunk_overlap() {
    let cfg = ChunkConfig { max_tokens: 10, overlap_tokens: 3 };
    let text = "one two three four five six seven eight nine ten eleven twelve thirteen";
    let chunks = chunk_text(text, &cfg);
    assert!(chunks.len() >= 2);
    // The last word of chunk N should appear in chunk N+1 (overlap)
    let words_0: Vec<&str> = chunks[0].split_whitespace().collect();
    let words_1: Vec<&str> = chunks[1].split_whitespace().collect();
    let last_of_0 = words_0.last().unwrap();
    assert!(words_1.iter().any(|w| w == last_of_0),
            "overlap: '{}' should appear in chunk 1: {:?}", last_of_0, words_1);
}

#[test]
fn test_chunk_respects_sentence_boundaries() {
    let cfg = ChunkConfig { max_tokens: 20, overlap_tokens: 0 };
    let text = "First sentence here. Second sentence follows. Third sentence ends.";
    let chunks = chunk_text(text, &cfg);
    // Chunks should not split mid-sentence
    for chunk in &chunks {
        let trimmed = chunk.trim();
        if !trimmed.is_empty() {
            assert!(trimmed.ends_with('.') || trimmed.ends_with('?') || trimmed.ends_with('!')
                    || chunks.last() == Some(chunk),
                "chunk should end at sentence boundary: '{}'", trimmed);
        }
    }
}
