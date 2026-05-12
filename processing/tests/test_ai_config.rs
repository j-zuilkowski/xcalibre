use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::ai_queries::{
    get_ai_config, upsert_ai_config, AiConfig,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_upsert_and_get_ai_config() {
    let pool = setup().await;
    let cfg = AiConfig {
        provider:            "ollama".into(),
        model:               "llama3".into(),
        embed_model:         "nomic-embed-text".into(),
        base_url:            "http://localhost:11434".into(),
        api_key:             None,
        reasoning_strategy:  "auto".into(),
        include_fields:      "[\"title\"]".into(),
    };
    upsert_ai_config(&pool, None, &cfg).await.expect("upsert");
    let loaded = get_ai_config(&pool, None).await.expect("get").expect("should exist");
    assert_eq!(loaded.provider, "ollama");
    assert_eq!(loaded.embed_model, "nomic-embed-text");
}

#[tokio::test]
async fn test_ai_config_defaults_to_ollama() {
    let pool = setup().await;
    // No config inserted — should return None
    let loaded = get_ai_config(&pool, None).await.expect("get");
    assert!(loaded.is_none(), "no config yet should return None");
}
