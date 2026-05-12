use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::similar_queries::{
    find_similar_books, SimilarBook, SimilarityFactors,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();

    let now = chrono::Utc::now().to_rfc3339();
    // Insert test books
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, series_name, created_at, updated_at)
         VALUES ('b1','Dune','[\"Frank Herbert\"]','EPUB','Dune Chronicles',?1,?1),
                ('b2','Dune Messiah','[\"Frank Herbert\"]','EPUB','Dune Chronicles',?1,?1),
                ('b3','Foundation','[\"Isaac Asimov\"]','EPUB','Foundation',?1,?1),
                ('b4','Pride and Prejudice','[\"Jane Austen\"]','EPUB',null,?1,?1),
                ('b5','Neuromancer','[\"William Gibson\"]','EPUB',null,?1,?1)"
    )
    .bind(&now)
    .execute(&pool).await.unwrap();

    // Insert tags
    sqlx::query("INSERT OR IGNORE INTO tags (name) VALUES ('sci-fi'),('classic'),('romance'),('cyberpunk')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT OR IGNORE INTO book_tags (book_id, tag_id) VALUES ('b1', 1),('b1', 2),('b2', 1),('b2', 2),('b3', 1),('b3', 2),('b4', 3),('b4', 2),('b5', 1),('b5', 4)")
        .execute(&pool).await.unwrap();

    pool
}

#[tokio::test]
async fn test_same_author_scores_high() {
    let pool = setup().await;
    let results = find_similar_books(
        &pool, "b1",
        &SimilarityFactors { same_author: true, same_series: true, shared_tags: true, same_language: true },
        10,
    ).await.expect("find_similar");
    assert!(!results.is_empty());
    assert_eq!(results[0].id, "b2", "Dune Messiah should be most similar to Dune");
}

#[tokio::test]
async fn test_shared_tags_included() {
    let pool = setup().await;
    let results = find_similar_books(
        &pool, "b1",
        &SimilarityFactors { same_author: false, same_series: false, shared_tags: true, same_language: true },
        10,
    ).await.expect("find_similar");
    let ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"b3") || ids.contains(&"b5"), "sci-fi books should appear: {:?}", ids);
}

#[tokio::test]
async fn test_excludes_source_book() {
    let pool = setup().await;
    let results = find_similar_books(
        &pool, "b1",
        &SimilarityFactors { same_author: true, same_series: true, shared_tags: true, same_language: true },
        10,
    ).await.expect("find_similar");
    assert!(!results.iter().any(|r| r.id == "b1"), "source book must not appear in results");
}

#[tokio::test]
async fn test_respects_limit() {
    let pool = setup().await;
    let results = find_similar_books(
        &pool, "b1",
        &SimilarityFactors { same_author: true, same_series: true, shared_tags: true, same_language: true },
        2,
    ).await.expect("find_similar");
    assert!(results.len() <= 2, "must respect limit of 2");
}

#[tokio::test]
async fn test_no_results_for_isolated_book() {
    let pool = setup().await;
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, created_at, updated_at)
         VALUES ('b99','Lonely Book','[\"Unknown Author\"]','EPUB',?1,?1)"
    )
    .bind(&now)
    .execute(&pool).await.unwrap();

    let results = find_similar_books(
        &pool, "b99",
        &SimilarityFactors { same_author: true, same_series: true, shared_tags: true, same_language: true },
        10,
    ).await.expect("find_similar");
    assert!(results.is_empty(), "isolated book should have no similar books");
}
