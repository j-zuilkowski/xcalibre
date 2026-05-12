use xcalibre_ai::citations::{extract_citations, Citation, CitedResponse};

#[test]
fn test_no_citations_when_no_chunks() {
    let response = "This is a simple response with no citations.";
    let chunks: Vec<&str> = vec![];
    let cited = extract_citations(response, &chunks);
    assert!(cited.citations.is_empty());
    assert_eq!(cited.response_text, response);
}

#[test]
fn test_citations_extracted_from_matching_chunks() {
    let response = "The spice must flow. It is the key to space travel.";
    let chunks = vec![
        "The spice melange is the most valuable substance in the universe.",
        "Without the spice, the Spacing Guild cannot fold space.",
        "Rain fell on the mountains.",
    ];
    let cited = extract_citations(response, &chunks);
    assert!(!cited.citations.is_empty(), "should find at least one relevant citation");
}

#[test]
fn test_citation_has_chunk_index() {
    let response = "Paul is the Kwisatz Haderach.";
    let chunks = vec!["Paul Atreides was trained as the Kwisatz Haderach by the Bene Gesserit."];
    let cited = extract_citations(response, &chunks);
    if !cited.citations.is_empty() {
        assert!(cited.citations[0].chunk_index < chunks.len());
    }
}

#[test]
fn test_citation_relevance_score_in_range() {
    let response = "Arrakis is a desert planet.";
    let chunks = vec!["Arrakis, known as Dune, is a harsh desert world."];
    let cited = extract_citations(response, &chunks);
    if !cited.citations.is_empty() {
        let score = cited.citations[0].relevance_score;
        assert!(score > 0.0 && score <= 1.0, "score must be in (0,1]: {score}");
    }
}
