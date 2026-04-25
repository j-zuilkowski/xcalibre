use crate::db::{annotation_queries, extended_queries, fts_queries};
use crate::error::ProcessingError;
use crate::utils::sort::{author_sort, title_sort};
use sqlx::SqlitePool;
use tracing::{info, warn};
use xcalibre_api::{client::ApiClient, read};

pub async fn sync_pull(pool: &SqlitePool, client: &ApiClient) -> Result<(), ProcessingError> {
    let mut page = 1u32;
    loop {
        let books = match read::list_books(client, page).await {
            Ok(b)  => b,
            Err(e) => { warn!("sync_pull page {} failed: {}", page, e); break; }
        };
        if books.is_empty() { break; }
        for book in &books {
            upsert_book(pool, book).await?;
        }
        info!("sync_pull page {} — {} books", page, books.len());
        page += 1;
    }
    Ok(())
}

async fn upsert_book(pool: &SqlitePool, book: &read::RemoteBook) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    let authors_json = serde_json::to_string(&book.authors)
        .map_err(|e| ProcessingError::DbError(sqlx::Error::Protocol(e.to_string())))?;
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, created_at, updated_at)
         VALUES (?,?,?,?,?,?)
         ON CONFLICT(id) DO UPDATE SET title=excluded.title,
                                        authors_json=excluded.authors_json,
                                        updated_at=excluded.updated_at",
    )
    .bind(&book.id)
    .bind(&book.title)
    .bind(&authors_json)
    .bind(&book.format)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    let title_sort_value = title_sort(&book.title);
    let author_sort_value = book
        .authors
        .first()
        .map(|author| author_sort(author))
        .unwrap_or_default();
    extended_queries::update_extended_metadata(
        pool,
        &book.id,
        Some(&title_sort_value),
        Some(&author_sort_value),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await?;
    fts_queries::refresh_book_index(pool, &book.id).await?;
    Ok(())
}

pub async fn sync_status_backprop(
    pool: &SqlitePool,
    client: &ApiClient,
) -> Result<(), ProcessingError> {
    let jobs: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, xs_book_id FROM jobs
         WHERE status = 'COMPLETED' AND xs_book_id IS NOT NULL",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    for (job_id, book_id) in jobs {
        match read::get_book(client, &book_id).await {
            Ok(None) => {
                warn!(job_id = %job_id, book_id = %book_id, "remote book deleted — re-queuing");
                sqlx::query(
                    "UPDATE jobs SET status='READY_TO_PUSH', updated_at=datetime() WHERE id=?",
                )
                .bind(&job_id)
                .execute(pool)
                .await
                .map_err(ProcessingError::DbError)?;
            }
            Ok(Some(_)) => {}
            Err(e) => warn!(job_id = %job_id, error = %e, "get_book check failed — skipping"),
        }
    }
    Ok(())
}

/// Push unsynced annotations to xcalibre-server.
/// Marks each annotation as synced on success.
/// Does nothing if no API client is configured.
pub async fn sync_annotations(
    pool: &SqlitePool,
    client: Option<&ApiClient>,
) -> Result<usize, ProcessingError> {
    let client = match client {
        Some(c) => c,
        None => {
            info!("no API client — skipping annotation sync");
            return Ok(0);
        }
    };

    let pending = annotation_queries::get_unsynced_annotations(pool).await?;
    let mut synced_count = 0;

    for ann in &pending {
        let payload = serde_json::json!({
            "id": ann.id,
            "book_id": ann.book_id,
            "type": ann.annotation_type,
            "cfi": ann.cfi,
            "selected_text": ann.selected_text,
            "note": ann.note,
            "color": ann.color,
            "created_at": ann.created_at,
        });

        let resp = client
            .http()
            .post(format!("{}/annotations", client.base_url()))
            .bearer_auth(client.token())
            .json(&payload)
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                annotation_queries::mark_annotation_synced(pool, &ann.id).await?;
                synced_count += 1;
                info!(annotation_id = %ann.id, "annotation synced");
            }
            Ok(r) => {
                warn!(annotation_id = %ann.id, status = %r.status(), "annotation sync failed");
            }
            Err(e) => {
                warn!(annotation_id = %ann.id, error = %e, "annotation sync error");
            }
        }
    }

    Ok(synced_count)
}
