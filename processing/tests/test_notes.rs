use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::notes_queries::{
    create_note, list_notes, get_note, update_note, delete_note,
    search_notes, NewNote, NoteRow,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_create_and_get_note() {
    let pool = setup().await;
    let new = NewNote {
        book_id:   "book1".into(),
        title:     "Chapter 1 Thoughts".into(),
        body_html: "<p>Great opening chapter.</p>".into(),
        body_text: "Great opening chapter.".into(),
    };
    create_note(&pool, &new).await.expect("create");
    let notes = list_notes(&pool, "book1").await.expect("list");
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "Chapter 1 Thoughts");
    assert_eq!(notes[0].body_html, "<p>Great opening chapter.</p>");
}

#[tokio::test]
async fn test_update_note() {
    let pool = setup().await;
    let new = NewNote {
        book_id:   "book1".into(),
        title:     "Original".into(),
        body_html: "<p>Old body.</p>".into(),
        body_text: "Old body.".into(),
    };
    create_note(&pool, &new).await.unwrap();
    let notes = list_notes(&pool, "book1").await.unwrap();
    let id = &notes[0].id.clone();

    update_note(&pool, id, "Updated", "<p>New body.</p>", "New body.")
        .await.expect("update");

    let note = get_note(&pool, id).await.unwrap().unwrap();
    assert_eq!(note.title, "Updated");
    assert_eq!(note.body_html, "<p>New body.</p>");
}

#[tokio::test]
async fn test_delete_note() {
    let pool = setup().await;
    let new = NewNote {
        book_id:   "book1".into(),
        title:     "To Delete".into(),
        body_html: "".into(),
        body_text: "".into(),
    };
    create_note(&pool, &new).await.unwrap();
    let notes = list_notes(&pool, "book1").await.unwrap();
    let id = notes[0].id.clone();

    delete_note(&pool, &id).await.expect("delete");
    let after = list_notes(&pool, "book1").await.unwrap();
    assert!(after.is_empty());
}

#[tokio::test]
async fn test_notes_scoped_to_book() {
    let pool = setup().await;
    for (book, title) in [("b1", "Note A"), ("b1", "Note B"), ("b2", "Note C")] {
        let new = NewNote {
            book_id:   book.into(),
            title:     title.into(),
            body_html: "".into(),
            body_text: "".into(),
        };
        create_note(&pool, &new).await.unwrap();
    }
    let b1 = list_notes(&pool, "b1").await.unwrap();
    let b2 = list_notes(&pool, "b2").await.unwrap();
    assert_eq!(b1.len(), 2);
    assert_eq!(b2.len(), 1);
}

#[tokio::test]
async fn test_search_notes_fts() {
    let pool = setup().await;
    for (title, text) in [("About Paul", "Paul Atreides leads"), ("About Sandworms", "Sandworms are massive")] {
        let new = NewNote {
            book_id:   "b1".into(),
            title:     title.into(),
            body_html: format!("<p>{text}</p>"),
            body_text: text.into(),
        };
        create_note(&pool, &new).await.unwrap();
    }
    let results = search_notes(&pool, "b1", "sandworms").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "About Sandworms");
}
