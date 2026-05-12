//! Tests for the spell check Tauri command API contract.
//! These tests are compile-time only — the real checking is OS-native,
//! so we just verify the command signatures compile correctly.
//! FAIL until rmp06b adds the commands to src-tauri.

// This test verifies that the spell check command types are defined
// in the processing crate's type exports.
use xcalibre_processing::spellcheck::{SpellCheckResult, SuggestionsResult};

#[test]
fn test_spell_check_result_type() {
    let r = SpellCheckResult { word: "colour".into(), is_correct: false };
    assert!(!r.is_correct);
    assert_eq!(r.word, "colour");
}

#[test]
fn test_suggestions_result_type() {
    let r = SuggestionsResult {
        word: "colour".into(),
        suggestions: vec!["color".into(), "coulor".into()],
    };
    assert_eq!(r.suggestions.len(), 2);
}
