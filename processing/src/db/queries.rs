use chrono::Utc;
use uuid::Uuid;

type JobRow = (
    String,
    String,
    String,
    String,
    String,
    i64,
    Option<String>,
    Option<String>,
    i64,
    Option<String>,
    String,
    String,
);

type BookmarkRow = (String, String, String, Option<String>, String);
type CollectionRow = (String, String, String);

#[derive(Debug, Clone)]
pub struct NewJob {
    pub id:                String,
    pub file_path:         String,
    pub file_sha256:       String,
    pub format:            String,
    pub status:            String,
    pub retry_count:       i64,
    pub next_retry_at:     Option<String>,
    pub xs_book_id: Option<String>,
    pub push_step:         i64,
    pub error_message:     Option<String>,
}

impl NewJob {
    pub fn new(file_path: &str, format: &str) -> Self {
        Self {
            id:                Uuid::new_v4().to_string(),
            file_path:         file_path.to_string(),
            file_sha256:       String::new(),
            format:            format.to_string(),
            status:            "PENDING".to_string(),
            retry_count:       0,
            next_retry_at:     None,
            xs_book_id: None,
            push_step:         0,
            error_message:     None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id:                String,
    pub file_path:         String,
    pub file_sha256:       String,
    pub format:            String,
    pub status:            String,
    pub retry_count:       i64,
    pub next_retry_at:     Option<String>,
    pub xs_book_id: Option<String>,
    pub push_step:         i64,
    pub error_message:     Option<String>,
    pub created_at:        String,
    pub updated_at:        String,
}

use crate::error::ProcessingError;
use sqlx::SqlitePool;

pub async fn create_job(pool: &SqlitePool, job: &NewJob) -> Result<String, ProcessingError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO jobs
         (id, file_path, file_sha256, format, status, retry_count,
          next_retry_at, xs_book_id, push_step, error_message,
          created_at, updated_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(&job.id)
    .bind(&job.file_path)
    .bind(&job.file_sha256)
    .bind(&job.format)
    .bind(&job.status)
    .bind(job.retry_count)
    .bind(&job.next_retry_at)
    .bind(&job.xs_book_id)
    .bind(job.push_step)
    .bind(&job.error_message)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(job.id.clone())
}

pub async fn get_job(pool: &SqlitePool, id: &str) -> Result<Option<Job>, ProcessingError> {
    let row = sqlx::query_as::<_, JobRow>(
        "SELECT id, file_path, file_sha256, format, status, retry_count,
                next_retry_at, xs_book_id, push_step, error_message,
                created_at, updated_at
         FROM jobs WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(ProcessingError::DbError)?
    .map(|(id, file_path, file_sha256, format, status, retry_count,
           next_retry_at, xs_book_id, push_step, error_message,
           created_at, updated_at)| Job {
        id, file_path, file_sha256, format, status, retry_count,
        next_retry_at, xs_book_id, push_step, error_message,
        created_at, updated_at,
    });
    Ok(row)
}

pub async fn update_job_status(
    pool: &SqlitePool,
    id: &str,
    status: &str,
) -> Result<(), ProcessingError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE jobs SET status = ?, updated_at = ? WHERE id = ?")
        .bind(status)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn list_jobs_by_status(
    pool: &SqlitePool,
    status: &str,
) -> Result<Vec<Job>, ProcessingError> {
    let rows = sqlx::query_as::<_, JobRow>(
        "SELECT id, file_path, file_sha256, format, status, retry_count,
                next_retry_at, xs_book_id, push_step, error_message,
                created_at, updated_at
         FROM jobs WHERE status = ? ORDER BY created_at ASC",
    )
    .bind(status)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?
    .into_iter()
    .map(|(id, file_path, file_sha256, format, status, retry_count,
           next_retry_at, xs_book_id, push_step, error_message,
           created_at, updated_at)| Job {
        id, file_path, file_sha256, format, status, retry_count,
        next_retry_at, xs_book_id, push_step, error_message,
        created_at, updated_at,
    })
    .collect();
    Ok(rows)
}

pub async fn update_reading_position(
    pool: &SqlitePool,
    book_id: &str,
    position: &str,
) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE local_books SET reading_cfi=?, last_opened_at=?, updated_at=? WHERE id=?",
    )
    .bind(position)
    .bind(&now)
    .bind(&now)
    .bind(book_id)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn update_progress_percent(
    pool: &SqlitePool,
    book_id: &str,
    percent: f64,
) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE local_books SET progress_percent=?, updated_at=? WHERE id=?",
    )
    .bind(percent)
    .bind(&now)
    .bind(book_id)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn upsert_local_book(
    pool: &SqlitePool,
    book_id: &str,
    title: &str,
    authors: &[String],
    format: &str,
    local_path: Option<&str>,
    cover_path: Option<&str>,
) -> Result<(), ProcessingError> {
    let now = Utc::now().to_rfc3339();
    let authors_json = serde_json::to_string(authors)
        .map_err(|e| ProcessingError::DbError(sqlx::Error::Protocol(e.to_string())))?;

    sqlx::query(
        "INSERT INTO local_books
         (id, title, authors_json, format, local_path, cover_path, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             title = excluded.title,
             authors_json = excluded.authors_json,
             format = excluded.format,
             local_path = COALESCE(excluded.local_path, local_books.local_path),
             cover_path = COALESCE(excluded.cover_path, local_books.cover_path),
             updated_at = excluded.updated_at",
    )
    .bind(book_id)
    .bind(title)
    .bind(&authors_json)
    .bind(format)
    .bind(local_path)
    .bind(cover_path)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub id:         String,
    pub book_id:    String,
    pub cfi:        String,
    pub label:      Option<String>,
    pub created_at: String,
}

pub async fn add_bookmark(
    pool: &SqlitePool,
    book_id: &str,
    cfi: &str,
    label: Option<&str>,
) -> Result<String, ProcessingError> {
    let id  = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO bookmarks (id, book_id, cfi, label, created_at) VALUES (?,?,?,?,?)",
    )
    .bind(&id)
    .bind(book_id)
    .bind(cfi)
    .bind(label)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(id)
}

pub async fn list_bookmarks(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Vec<Bookmark>, ProcessingError> {
    let rows: Vec<BookmarkRow> = sqlx::query_as(
        "SELECT id, book_id, cfi, label, created_at FROM bookmarks WHERE book_id=? ORDER BY created_at ASC",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(rows
        .into_iter()
        .map(|(id, book_id, cfi, label, created_at)| Bookmark {
            id,
            book_id,
            cfi,
            label,
            created_at,
        })
        .collect())
}

pub async fn delete_bookmark(pool: &SqlitePool, bookmark_id: &str) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM bookmarks WHERE id=?")
        .bind(bookmark_id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn search_books(pool: &SqlitePool, query: &str) -> Result<Vec<Job>, ProcessingError> {
    let rows: Vec<JobRow> = sqlx::query_as(
            "SELECT j.id, j.file_path, j.file_sha256, j.format, j.status,
                    j.retry_count, j.next_retry_at, j.xs_book_id,
                    j.push_step, j.error_message, j.created_at, j.updated_at
             FROM books_fts
             JOIN jobs j ON j.id = books_fts.book_id
             WHERE books_fts MATCH ?
             ORDER BY rank
             LIMIT 100",
        )
        .bind(query)
        .fetch_all(pool)
        .await
        .map_err(ProcessingError::DbError)?;

    Ok(rows
        .into_iter()
        .map(|(id, file_path, file_sha256, format, status, retry_count, next_retry_at, xs_book_id, push_step, error_message, created_at, updated_at)| Job {
            id,
            file_path,
            file_sha256,
            format,
            status,
            retry_count,
            next_retry_at,
            xs_book_id,
            push_step,
            error_message,
            created_at,
            updated_at,
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

pub async fn create_collection(
    pool: &SqlitePool,
    name: &str,
) -> Result<String, ProcessingError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO collections (id, name, created_at) VALUES (?,?,?)")
        .bind(&id)
        .bind(name)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(id)
}

pub async fn list_collections(pool: &SqlitePool) -> Result<Vec<Collection>, ProcessingError> {
    let rows: Vec<CollectionRow> = sqlx::query_as(
        "SELECT id, name, created_at FROM collections ORDER BY name ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(rows
        .into_iter()
        .map(|(id, name, created_at)| Collection {
            id,
            name,
            created_at,
        })
        .collect())
}

pub async fn add_book_to_collection(
    pool: &SqlitePool,
    collection_id: &str,
    book_id: &str,
) -> Result<(), ProcessingError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR IGNORE INTO collection_books (collection_id, book_id, added_at) VALUES (?,?,?)",
    )
    .bind(collection_id)
    .bind(book_id)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn remove_book_from_collection(
    pool: &SqlitePool,
    collection_id: &str,
    book_id: &str,
) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM collection_books WHERE collection_id=? AND book_id=?")
        .bind(collection_id)
        .bind(book_id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn get_collection_books(
    pool: &SqlitePool,
    collection_id: &str,
) -> Result<Vec<Job>, ProcessingError> {
    let rows: Vec<JobRow> = sqlx::query_as(
        "SELECT j.id, j.file_path, j.file_sha256, j.format, j.status,
                j.retry_count, j.next_retry_at, j.xs_book_id,
                j.push_step, j.error_message, j.created_at, j.updated_at
         FROM jobs j
         JOIN collection_books cb ON cb.book_id = j.id
         WHERE cb.collection_id = ?
         ORDER BY j.created_at ASC",
    )
    .bind(collection_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(rows
        .into_iter()
        .map(|(id, file_path, file_sha256, format, status, retry_count,
               next_retry_at, xs_book_id, push_step, error_message,
               created_at, updated_at)| Job {
            id,
            file_path,
            file_sha256,
            format,
            status,
            retry_count,
            next_retry_at,
            xs_book_id,
            push_step,
            error_message,
            created_at,
            updated_at,
        })
        .collect())
}
