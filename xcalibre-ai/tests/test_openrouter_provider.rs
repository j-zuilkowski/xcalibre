use xcalibre_ai::{AiProvider, openrouter::OpenRouterBackend};

fn make_backend() -> OpenRouterBackend {
    OpenRouterBackend::new(
        "https://openrouter.ai/api/v1",
        "test-key",
        "mistralai/mistral-7b-instruct",
        "openai/text-embedding-ada-002",
    )
}

#[test]
fn test_openrouter_backend_name() {
    assert_eq!(make_backend().name(), "openrouter");
}

#[test]
fn test_openrouter_backend_model() {
    assert_eq!(make_backend().model(), "mistralai/mistral-7b-instruct");
}

#[tokio::test]
async fn test_openrouter_errors_with_bad_key() {
    let b = OpenRouterBackend::new(
        "https://openrouter.ai/api/v1",
        "invalid",
        "mistralai/mistral-7b-instruct",
        "openai/text-embedding-ada-002",
    );
    assert!(b.list_models().await.is_err());
}
