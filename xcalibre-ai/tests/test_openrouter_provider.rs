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
async fn test_openrouter_errors_on_unreachable_url() {
    // OpenRouter's /models endpoint is public and returns 200 regardless of the
    // API key, so we cannot use a bad-key test. Instead, point at an address
    // that is guaranteed to refuse connections to verify that network errors
    // propagate as Err rather than panicking.
    let b = OpenRouterBackend::new(
        "http://127.0.0.1:1",
        "invalid",
        "mistralai/mistral-7b-instruct",
        "openai/text-embedding-ada-002",
    );
    assert!(b.list_models().await.is_err());
}
