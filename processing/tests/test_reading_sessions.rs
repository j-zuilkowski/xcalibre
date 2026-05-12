use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::reading_sessions::{
    start_reading_session, end_reading_session, list_reading_sessions,
    total_reading_time_seconds, ReadingSession,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, created_at, updated_at)
         VALUES ('b1', 'Dune', '[]', 'EPUB', '2024-01-01', '2024-01-01')"
    ).execute(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_start_and_end_session() {
    let pool = setup().await;
    let session_id = start_reading_session(&pool, "b1", 0.0).await.expect("start");
    end_reading_session(&pool, &session_id, 60, 0.0, 5.0).await.expect("end");

    let sessions = list_reading_sessions(&pool, "b1").await.expect("list");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].duration_s, 60);
    assert_eq!(sessions[0].progress_end, 5.0);
}

#[tokio::test]
async fn test_total_reading_time() {
    let pool = setup().await;
    for _ in 0..3 {
        let id = start_reading_session(&pool, "b1", 0.0).await.unwrap();
        end_reading_session(&pool, &id, 120, 0.0, 1.0).await.unwrap();
    }
    let total = total_reading_time_seconds(&pool, "b1").await.expect("total");
    assert_eq!(total, 360, "3 sessions of 120s = 360s total");
}

#[tokio::test]
async fn test_sessions_scoped_to_book() {
    let pool = setup().await;
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, created_at, updated_at) VALUES ('b2', 'Foundation', '[]', 'EPUB', '2024-01-01', '2024-01-01')"
    ).execute(&pool).await.unwrap();

    let id1 = start_reading_session(&pool, "b1", 0.0).await.unwrap();
    end_reading_session(&pool, &id1, 60, 0.0, 1.0).await.unwrap();
    let id2 = start_reading_session(&pool, "b2", 0.0).await.unwrap();
    end_reading_session(&pool, &id2, 90, 0.0, 1.0).await.unwrap();

    let b1_sessions = list_reading_sessions(&pool, "b1").await.unwrap();
    let b2_sessions = list_reading_sessions(&pool, "b2").await.unwrap();
    assert_eq!(b1_sessions.len(), 1);
    assert_eq!(b2_sessions.len(), 1);
}

// Needed to suppress unused import warning when module doesn't exist yet
#[allow(dead_code)]
fn _assert_reading_session_type(_: ReadingSession) {}
