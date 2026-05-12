use crate::error::ProcessingError;
use sqlx::SqlitePool;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct AiConfig {
    pub provider:           String,
    pub model:              String,
    pub embed_model:        String,
    pub base_url:           String,
    pub api_key:            Option<String>,
    pub reasoning_strategy: String,
    pub include_fields:     String,
}

pub async fn get_ai_config(pool: &SqlitePool, library_id: Option<&str>)
    -> Result<Option<AiConfig>, ProcessingError>
{
    sqlx::query_as::<_, AiConfig>(
        "SELECT provider, model, embed_model, base_url, api_key,
                reasoning_strategy, include_fields
         FROM ai_config WHERE library_id IS ? LIMIT 1",
    )
    .bind(library_id)
    .fetch_optional(pool).await.map_err(ProcessingError::DbError)
}

pub async fn upsert_ai_config(pool: &SqlitePool, library_id: Option<&str>, cfg: &AiConfig)
    -> Result<(), ProcessingError>
{
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO ai_config
         (library_id, provider, model, embed_model, base_url, api_key,
          reasoning_strategy, include_fields, updated_at)
         VALUES (?,?,?,?,?,?,?,?,?)
         ON CONFLICT(library_id) DO UPDATE SET
          provider=excluded.provider, model=excluded.model,
          embed_model=excluded.embed_model, base_url=excluded.base_url,
          api_key=COALESCE(excluded.api_key, ai_config.api_key),
          reasoning_strategy=excluded.reasoning_strategy,
          include_fields=excluded.include_fields, updated_at=excluded.updated_at",
    )
    .bind(library_id).bind(&cfg.provider).bind(&cfg.model).bind(&cfg.embed_model)
    .bind(&cfg.base_url).bind(&cfg.api_key).bind(&cfg.reasoning_strategy)
    .bind(&cfg.include_fields).bind(&now)
    .execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn insert_book_chunk(
    pool: &SqlitePool, book_id: &str, chunk_index: i64,
    chunk_text: &str, embedding: Option<&[u8]>, embed_model: Option<&str>,
) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO book_chunks (book_id, chunk_index, chunk_text, embedding, embed_model, created_at)
         VALUES (?,?,?,?,?,?)
         ON CONFLICT(book_id, chunk_index) DO UPDATE SET
          chunk_text=excluded.chunk_text, embedding=excluded.embedding,
          embed_model=excluded.embed_model",
    )
    .bind(book_id).bind(chunk_index).bind(chunk_text).bind(embedding).bind(embed_model).bind(&now)
    .execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn get_book_chunks(pool: &SqlitePool, book_id: &str)
    -> Result<Vec<(i64, String, Option<Vec<u8>>)>, ProcessingError>
{
    let rows: Vec<(i64, String, Option<Vec<u8>>)> = sqlx::query_as(
        "SELECT chunk_index, chunk_text, embedding FROM book_chunks
         WHERE book_id = ? ORDER BY chunk_index",
    )
    .bind(book_id)
    .fetch_all(pool).await.map_err(ProcessingError::DbError)?;
    Ok(rows)
}

pub async fn delete_book_chunks(pool: &SqlitePool, book_id: &str) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM book_chunks WHERE book_id = ?")
        .bind(book_id).execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}
