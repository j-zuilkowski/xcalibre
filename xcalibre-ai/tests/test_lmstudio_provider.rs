use xcalibre_ai::{AiProvider, lmstudio::LmStudioBackend};

fn make_backend() -> LmStudioBackend {
    LmStudioBackend::new("http://localhost:1234", "loaded-model-id", "nomic-embed-text")
}

#[test]
fn test_lmstudio_backend_name() {
    assert_eq!(make_backend().name(), "lmstudio");
}

#[test]
fn test_lmstudio_backend_model() {
    assert_eq!(make_backend().model(), "loaded-model-id");
}

#[tokio::test]
async fn test_lmstudio_errors_when_not_running() {
    let b = LmStudioBackend::new("http://localhost:19999", "model", "embed");
    let result = b.list_models().await;
    assert!(result.is_err(), "should fail when LM Studio is not running");
}
