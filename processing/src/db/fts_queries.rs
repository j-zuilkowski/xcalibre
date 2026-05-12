//! Full-text search index operations for the `books_fts` FTS5 virtual table.
//!
//! ## Schema
//!
//! The FTS5 virtual table is defined in the migration files and has this shape:
//!
//! ```sql
//! CREATE VIRTUAL TABLE books_fts USING fts5(
//!     book_id UNINDEXED,   -- foreign key to local_books.id, not indexed
//!     title,
//!     authors,
//!     description,
//!     full_text            -- extracted plain text from the book body
//! );
//! ```
//!
//! `book_id` is `UNINDEXED` because we query *from* it (via `WHERE books_fts
//! MATCH ?`) and then look up the row by `book_id`, not the other way around.
//!
//! ## Search syntax
//!
//! The `query` parameter passed to [`search`] is forwarded directly to SQLite's
//! FTS5 MATCH operator, so users can use field-prefix syntax:
//!
//! - `title:dune` — search only the title column
//! - `authors:herbert` — search only the authors column
//! - `dune herbert` — AND across all columns (FTS5 default)
//! - `dune OR tolkien` — explicit OR
//! - `"lord of the rings"` — phrase search
//!
//! Invalid MATCH queries (e.g. an empty or malformed expression) would normally
//! cause sqlx to return a `Database` error. [`search`] silently swallows those
//! errors and returns an empty result set — see the safety note in that function.
//!
//! ## Update strategy
//!
//! FTS5 triggers do not exist in SQLite, so the index must be updated manually:
//!
//! - [`upsert_fts`] is called by the ingest pipeline after metadata and text
//!   extraction complete.
//! - [`refresh_book_index`] is called by `update_book_details` in commands.rs
//!   after the user edits metadata, so the title/author changes are immediately
//!   searchable.

use crate::error::ProcessingError;
use sqlx::SqlitePool;

/// Insert or replace a book's full-text search entry.
///
/// FTS5 does not support `ON CONFLICT` or `UPDATE` directly. The conventional
/// approach is delete-then-insert, which is what this function does. The delete
/// targets by `book_id` (an unindexed column), so it uses a subquery to find the
/// internal `rowid` that FTS5 requires for deletion.
///
/// Called after the ingest pipeline finishes text extraction and after the user
/// updates book metadata via `update_book_details`.
pub async fn upsert_fts(
    pool: &SqlitePool,
    book_id: &str,
    title: &str,
    authors: &str,
    description: &str,
    full_text: &str,
) -> Result<(), ProcessingError> {
    // Delete any existing FTS row for this book before re-inserting.
    // FTS5 requires deleting by rowid; the subquery resolves book_id → rowid.
    sqlx::query(
        "DELETE FROM books_fts WHERE rowid IN
         (SELECT rowid FROM books_fts WHERE book_id = ?)",
    )
    .bind(book_id)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    sqlx::query(
        "INSERT INTO books_fts (book_id, title, authors, description, full_text) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(book_id)
    .bind(title)
    .bind(authors)
    .bind(description)
    .bind(full_text)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(())
}

/// Search the FTS index and return matching `book_id` values.
///
/// Results are ordered by FTS5's built-in `rank` (BM25 relevance). The caller
/// (`search_books` in commands.rs) then fetches full book rows using the returned
/// IDs.
///
/// ## Error handling for malformed queries
///
/// SQLite FTS5 returns a `Database` error for syntactically invalid MATCH
/// expressions (e.g. `title:` with no term, or an unclosed quote). Rather than
/// propagating these as errors — which would surface as error toasts in the UI
/// for routine user mistakes like trailing spaces — malformed-query errors are
/// detected by matching the error message against `"fts5:"` or `"malformed MATCH"`
/// and converted to an empty result set. All other database errors still propagate.
pub async fn search(pool: &SqlitePool, query: &str) -> Result<Vec<String>, ProcessingError> {
    let result = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT book_id FROM books_fts WHERE books_fts MATCH ? ORDER BY rank",
    )
    .bind(query)
    .fetch_all(pool)
    .await;

    match result {
        Ok(rows) => Ok(rows.into_iter().map(|(id,)| id).collect()),
        // Swallow FTS syntax errors: a bad query returns no results rather than an error.
        Err(sqlx::Error::Database(e))
            if e.message().contains("fts5:") || e.message().contains("malformed MATCH") =>
        {
            Ok(vec![])
        }
        Err(e) => Err(ProcessingError::DbError(e)),
    }
}

/// Re-index a single book by re-reading its current metadata and extracted text.
///
/// Used after metadata edits so the FTS index stays in sync with `local_books`
/// without requiring a full library re-index. The join with `job_text` picks up
/// any previously-extracted full text.
///
/// Does nothing if `book_id` is not found in `local_books` (e.g. race condition
/// where the book was deleted just before the edit committed).
pub async fn refresh_book_index(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<(), ProcessingError> {
    let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>)>(
        "SELECT lb.title, lb.authors_json, lb.description, jt.full_text
         FROM local_books lb
         LEFT JOIN job_text jt ON jt.job_id = lb.id
         WHERE lb.id = ?",
    )
    .bind(book_id)
    .fetch_optional(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    let Some((title, authors_json, description, full_text)) = row else {
        return Ok(());
    };

    upsert_fts(
        pool,
        book_id,
        &title,
        &authors_json,
        description.as_deref().unwrap_or(""),
        full_text.as_deref().unwrap_or(""),
    )
    .await
}
