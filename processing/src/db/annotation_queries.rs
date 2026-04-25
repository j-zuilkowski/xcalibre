use crate::error::ProcessingError;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Annotation {
    pub id: String,
    pub book_id: String,
    pub annotation_type: String,
    pub cfi: String,
    pub selected_text: Option<String>,
    pub note: Option<String>,
    pub color: String,
    pub synced: bool,
    pub created_at: String,
}

pub async fn create_annotation(
    pool: &SqlitePool,
    book_id: &str,
    annotation_type: &str,
    cfi: &str,
    selected_text: Option<&str>,
    note: Option<&str>,
    color: &str,
) -> Result<String, ProcessingError> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO annotations
         (id, book_id, type, cfi, selected_text, note, color, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(book_id)
    .bind(annotation_type)
    .bind(cfi)
    .bind(selected_text)
    .bind(note)
    .bind(color)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(id)
}

pub async fn get_annotations(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Vec<Annotation>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, Option<String>, String, i64, String)>(
        "SELECT id, book_id, type, cfi, selected_text, note, color, synced, created_at
         FROM annotations WHERE book_id = ? ORDER BY created_at ASC",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(rows
        .into_iter()
        .map(
            |(id, book_id, annotation_type, cfi, selected_text, note, color, synced, created_at)| {
                Annotation {
                    id,
                    book_id,
                    annotation_type,
                    cfi,
                    selected_text,
                    note,
                    color,
                    synced: synced != 0,
                    created_at,
                }
            },
        )
        .collect())
}

pub async fn delete_annotation(pool: &SqlitePool, id: &str) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM annotations WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn update_annotation_note(
    pool: &SqlitePool,
    id: &str,
    note: &str,
) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE annotations SET note = ?, updated_at = ? WHERE id = ?")
        .bind(note)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn get_unsynced_annotations(pool: &SqlitePool) -> Result<Vec<Annotation>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, Option<String>, String, i64, String)>(
        "SELECT id, book_id, type, cfi, selected_text, note, color, synced, created_at
         FROM annotations WHERE synced = 0 ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(rows
        .into_iter()
        .map(
            |(id, book_id, annotation_type, cfi, selected_text, note, color, synced, created_at)| {
                Annotation {
                    id,
                    book_id,
                    annotation_type,
                    cfi,
                    selected_text,
                    note,
                    color,
                    synced: synced != 0,
                    created_at,
                }
            },
        )
        .collect())
}

pub async fn mark_annotation_synced(pool: &SqlitePool, id: &str) -> Result<(), ProcessingError> {
    sqlx::query("UPDATE annotations SET synced = 1, updated_at = ? WHERE id = ?")
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}
