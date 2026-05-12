//! Database operations for AI configuration and book chunk embeddings.
//!
//! ## Tables
//!
//! ### `ai_config`
//!
//! Stores per-library AI provider settings. The `library_id` column has a unique
//! index (migration `0021_ai_config_unique_library.sql`) ensuring at most one
//! config row per library. `NULL` `library_id` means the global (default) config.
//!
//! ```sql
//! CREATE TABLE ai_config (
//!     id                 TEXT PRIMARY KEY,
//!     library_id         TEXT,                     -- NULL = global config
//!     provider           TEXT NOT NULL,
//!     model              TEXT NOT NULL DEFAULT '',
//!     embed_model        TEXT NOT NULL DEFAULT '',
//!     base_url           TEXT NOT NULL,
//!     api_key            TEXT,                     -- NULL if provider needs no key
//!     reasoning_strategy TEXT NOT NULL DEFAULT 'auto',
//!     include_fields     TEXT NOT NULL DEFAULT '["title","authors","tags"]',
//!     updated_at         TEXT NOT NULL
//! );
//! ```
//!
//! ### `book_chunks`
//!
//! Stores overlapping text chunks extracted from book content for RAG (Retrieval-
//! Augmented Generation) AI queries. Each chunk also stores an optional embedding
//! vector (serialised as `BLOB`) for semantic search.
//!
//! ```sql
//! CREATE TABLE book_chunks (
//!     book_id      TEXT NOT NULL,
//!     chunk_index  INTEGER NOT NULL,
//!     chunk_text   TEXT NOT NULL,
//!     embedding    BLOB,       -- f32 array, NULL if not yet embedded
//!     embed_model  TEXT,       -- model that produced the embedding
//!     created_at   TEXT NOT NULL,
//!     PRIMARY KEY (book_id, chunk_index)
//! );
//! ```
//!
//! ## API key handling
//!
//! API keys are stored **in plaintext** in the SQLite database rather than the OS
//! keychain because they are per-library (not per-user) and must be portable in
//! library backups. The `AiConfigPublic` struct in commands.rs redacts the key
//! before sending config to the WebView — the frontend never sees the raw value.
//!
//! The `ON CONFLICT(library_id) DO UPDATE SET ... api_key = COALESCE(excluded.api_key,
//! ai_config.api_key)` pattern in [`upsert_ai_config`] preserves the existing key
//! when the user saves settings with the masked placeholder (`"••••••••"`) still in
//! the API key field. The frontend sends `null` for unchanged keys; `COALESCE` keeps
//! the stored value in that case.

use crate::error::ProcessingError;
use sqlx::SqlitePool;

/// Full AI configuration row as stored in the database.
///
/// Never returned to the WebView directly — use `AiConfigPublic` in commands.rs
/// which replaces `api_key` with `has_api_key: bool`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct AiConfig {
    /// Which AI backend to use. One of `"ollama"`, `"openai"`, `"gemini"`,
    /// `"lmstudio"`, `"openrouter"`. Passed to `xcalibre_ai::factory::make_provider`.
    pub provider: String,

    /// The model identifier sent to the provider (e.g. `"llama3.2"`, `"gpt-4o"`).
    pub model: String,

    /// The embedding model identifier used when chunking books for RAG.
    /// May be empty if the provider does not support embeddings.
    pub embed_model: String,

    /// Base URL of the provider's API (e.g. `"http://localhost:11434"` for Ollama).
    pub base_url: String,

    /// Bearer token or API key. `None` for local providers (Ollama, LM Studio)
    /// that require no authentication.
    pub api_key: Option<String>,

    /// Controls extended thinking / chain-of-thought. One of `"none"`, `"low"`,
    /// `"medium"`, `"high"`, or `"auto"`. Passed to `xcalibre_ai::reasoning`.
    pub reasoning_strategy: String,

    /// JSON array of metadata field names to include in the AI context for book-
    /// level queries (e.g. `["title","authors","tags","description"]`).
    pub include_fields: String,
}

/// Fetch the AI config for the given library, or `None` if not yet configured.
///
/// Pass `library_id = None` to get the global (default) config.
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

/// Insert or update the AI config for a library.
///
/// The `ON CONFLICT(library_id)` clause guarantees upsert semantics: if a row
/// already exists for this library, all fields are updated except `api_key`.
///
/// The `api_key` field uses `COALESCE(excluded.api_key, ai_config.api_key)`:
/// - If the caller passes a new non-null key, it replaces the stored key.
/// - If the caller passes `NULL` (meaning "unchanged"), the existing key is kept.
///
/// This is the mechanism that lets the frontend show `"••••••••"` without ever
/// reading or transmitting the real key — the UI sends `null` and the database
/// retains the original.
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

/// Insert or update a single text chunk and optional embedding for a book.
///
/// `chunk_index` is the 0-based position within the book's chunk sequence.
/// `ON CONFLICT` updates the text and embedding, allowing re-embedding after
/// a model change without deleting all chunks first.
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

/// Fetch all chunks for a book, ordered by chunk index.
///
/// Returns `(chunk_index, chunk_text, embedding_bytes)` tuples. The embedding
/// `Vec<u8>` is the raw little-endian f32 array if present, or `None` if this
/// chunk has not been embedded yet.
///
/// The AI chat handler in commands.rs takes the first 5 chunks by relevance
/// score; this function returns all chunks so the scorer can rank them.
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

/// Delete all stored chunks for a book.
///
/// Called before re-chunking after a full re-ingest, or when a book is deleted
/// as part of [`crate::commands::bulk_delete_books`].
pub async fn delete_book_chunks(pool: &SqlitePool, book_id: &str) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM book_chunks WHERE book_id = ?")
        .bind(book_id).execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}
