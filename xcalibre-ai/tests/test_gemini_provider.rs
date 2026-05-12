use xcalibre_ai::{AiProvider, gemini::GeminiBackend};

fn make_backend() -> GeminiBackend {
    GeminiBackend::new(
        "https://generativelanguage.googleapis.com/v1beta",
        "test-key",
        "gemini-2.0-flash",
        "text-embedding-004",
    )
}

#[test]
fn test_gemini_backend_name() {
    assert_eq!(make_backend().name(), "gemini");
}

#[test]
fn test_gemini_backend_stores_model() {
    assert_eq!(make_backend().model(), "gemini-2.0-flash");
}

#[tokio::test]
async fn test_gemini_list_models_errors_gracefully() {
    let b = GeminiBackend::new(
        "https://generativelanguage.googleapis.com/v1beta",
        "invalid-key",
        "gemini-2.0-flash",
        "text-embedding-004",
    );
    assert!(b.list_models().await.is_err());
}
