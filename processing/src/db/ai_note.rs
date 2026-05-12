use crate::db::notes_queries::{create_note, NewNote};
use sqlx::sqlite::SqlitePool;

pub async fn save_ai_response_as_note(
    pool: &SqlitePool,
    book_id: &str,
    title: &str,
    body_html: &str,
    body_text: &str,
) -> Result<(), sqlx::Error> {
    let new = NewNote {
        book_id:   book_id.to_string(),
        title:     title.to_string(),
        body_html: body_html.to_string(),
        body_text: body_text.to_string(),
    };
    create_note(pool, &new).await?;
    Ok(())
}
