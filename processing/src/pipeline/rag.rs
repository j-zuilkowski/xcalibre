//! RAG embedding pipeline step: chunk job_text and store chunks.
//! Embedding is performed asynchronously after text extraction.
//! When no embed service is reachable, chunks are stored without embeddings.

use crate::db::ai_queries::{delete_book_chunks, get_ai_config, insert_book_chunk};
use crate::error::ProcessingError;
use sqlx::SqlitePool;
use tracing::{info, warn};
use xcalibre_ai::chunk::{chunk_text, ChunkConfig};

pub async fn run_rag_chunk(
    pool: &SqlitePool,
    book_id: &str,
    full_text: &str,
) -> Result<(), ProcessingError> {
    if full_text.trim().is_empty() { return Ok(()); }

    let cfg = ChunkConfig::default();
    let chunks = chunk_text(full_text, &cfg);
    if chunks.is_empty() { return Ok(()); }

    delete_book_chunks(pool, book_id).await?;

    let ai_cfg = get_ai_config(pool, None).await.ok().flatten();
    let (embed_model, base_url) = ai_cfg
        .as_ref()
        .map(|c| (c.embed_model.as_str(), c.base_url.as_str()))
        .unwrap_or(("nomic-embed-text", "http://localhost:11434"));

    for (i, chunk) in chunks.iter().enumerate() {
        // Attempt embedding; fall back to None if Ollama is not running
        let embedding = try_embed(chunk, embed_model, base_url).await;
        let embed_blob = embedding.map(|v| {
            v.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<u8>>()
        });
        insert_book_chunk(pool, book_id, i as i64, chunk,
            embed_blob.as_deref(), Some(embed_model)).await?;
    }

    info!(book_id, chunks = chunks.len(), "RAG chunks stored");
    Ok(())
}

async fn try_embed(text: &str, model: &str, base_url: &str) -> Option<Vec<f32>> {
    use xcalibre_ai::ollama::OllamaBackend;
    use xcalibre_ai::provider::AiProvider;
    let backend = OllamaBackend::new(base_url, model, model);
    match backend.embed(text).await {
        Ok(v)  => Some(v),
        Err(e) => { warn!("embed failed (non-fatal): {}", e); None }
    }
}
