use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::notes_queries::list_notes;

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_save_ai_response_as_note() {
    let pool = setup().await;
    sqlx::query(
        "INSERT INTO local_books (id, title, authors, format) VALUES ('b1', 'Dune', '[]', 'EPUB')"
    ).execute(&pool).await.unwrap();

    xcalibre_processing::db::ai_note::save_ai_response_as_note(
        &pool,
        "b1",
        "AI Summary",
        "<p>Paul Atreides travels to Arrakis.</p>",
        "Paul Atreides travels to Arrakis.",
    ).await.expect("save");

    let notes = list_notes(&pool, "b1").await.unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "AI Summary");
    assert!(notes[0].body_html.contains("Arrakis"));
}
