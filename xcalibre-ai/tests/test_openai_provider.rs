use xcalibre_ai::{
    AiProvider, ChatMessage, MessageRole,
    openai::OpenAiBackend,
};

fn make_backend() -> OpenAiBackend {
    OpenAiBackend::new(
        "https://api.openai.com/v1",
        "test-key",
        "gpt-4o-mini",
        "text-embedding-3-small",
    )
}

#[test]
fn test_openai_backend_name() {
    let b = make_backend();
    assert_eq!(b.name(), "openai");
}

#[test]
fn test_openai_backend_stores_config() {
    let b = make_backend();
    assert_eq!(b.model(), "gpt-4o-mini");
    assert_eq!(b.embed_model(), "text-embedding-3-small");
}

#[tokio::test]
async fn test_openai_list_models_returns_error_with_bad_key() {
    let b = OpenAiBackend::new(
        "https://api.openai.com/v1",
        "sk-invalid-key-for-testing",
        "gpt-4o",
        "text-embedding-3-small",
    );
    let result = b.list_models().await;
    assert!(result.is_err(), "invalid key must return error");
}
