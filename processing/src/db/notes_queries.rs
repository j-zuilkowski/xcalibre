use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NoteRow {
    pub id:         String,
    pub book_id:    String,
    pub title:      String,
    pub body_html:  String,
    pub body_text:  String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewNote {
    pub book_id:   String,
    pub title:     String,
    pub body_html: String,
    pub body_text: String,
}

pub async fn create_note(
    pool: &SqlitePool,
    new: &NewNote,
) -> Result<NoteRow, sqlx::Error> {
    sqlx::query_as::<_, NoteRow>(
        "INSERT INTO notes (book_id, title, body_html, body_text)
         VALUES (?, ?, ?, ?)
         RETURNING *"
    )
    .bind(&new.book_id)
    .bind(&new.title)
    .bind(&new.body_html)
    .bind(&new.body_text)
    .fetch_one(pool)
    .await
}

pub async fn list_notes(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Vec<NoteRow>, sqlx::Error> {
    sqlx::query_as::<_, NoteRow>(
        "SELECT * FROM notes WHERE book_id=? ORDER BY updated_at DESC"
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
}

pub async fn get_note(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<NoteRow>, sqlx::Error> {
    sqlx::query_as::<_, NoteRow>("SELECT * FROM notes WHERE id=?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn update_note(
    pool: &SqlitePool,
    id: &str,
    title: &str,
    body_html: &str,
    body_text: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE notes SET title=?, body_html=?, body_text=?, updated_at=datetime('now')
         WHERE id=?"
    )
    .bind(title)
    .bind(body_html)
    .bind(body_text)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_note(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM notes WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn search_notes(
    pool: &SqlitePool,
    book_id: &str,
    query: &str,
) -> Result<Vec<NoteRow>, sqlx::Error> {
    sqlx::query_as::<_, NoteRow>(
        "SELECT n.* FROM notes n
         JOIN notes_fts f ON n.rowid = f.rowid
         WHERE n.book_id=? AND notes_fts MATCH ?
         ORDER BY n.updated_at DESC"
    )
    .bind(book_id)
    .bind(query)
    .fetch_all(pool)
    .await
}
