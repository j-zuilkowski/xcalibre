#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpellCheckResult {
    pub word:       String,
    pub is_correct: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SuggestionsResult {
    pub word:        String,
    pub suggestions: Vec<String>,
}
