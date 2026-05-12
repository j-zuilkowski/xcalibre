//! OS-native spell check (S2-A: WebView layer only; no Rust pipeline check).

#[cfg(target_os = "macos")]
mod macos {
    /// Check if a word is spelled correctly using NSSpellChecker.
    pub fn check_word(word: &str) -> bool {
        let _ = word;
        true // stub — replace with real objc2 call when objc2 is added to Cargo.toml
    }

    pub fn suggestions(word: &str) -> Vec<String> {
        let _ = word;
        vec![] // stub
    }

    pub fn add_to_dictionary(word: &str) {
        let _ = word;
    }
}

#[cfg(not(target_os = "macos"))]
mod macos {
    pub fn check_word(_word: &str) -> bool { true }
    pub fn suggestions(_word: &str) -> Vec<String> { vec![] }
    pub fn add_to_dictionary(_word: &str) {}
}

pub use macos::{add_to_dictionary, check_word, suggestions};
