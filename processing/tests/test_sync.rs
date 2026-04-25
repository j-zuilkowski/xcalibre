use sqlx::sqlite::SqlitePoolOptions;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xcalibre_api::client::ApiClient;
use xcalibre_processing::pipeline::sync::sync_pull;

async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_sync_pull_paginates() {
    let server = MockServer::start().await;

    let books_page1 = serde_json::json!([
        { "id": "b1", "title": "Book One", "authors": ["Author A"], "format": "EPUB", "sha256": "aaa" },
        { "id": "b2", "title": "Book Two", "authors": ["Author B"], "format": "PDF",  "sha256": "bbb" }
    ]);

    Mock::given(method("GET"))
        .and(path("/books"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&books_page1))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/books"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let pool = setup_db().await;
    let client = ApiClient::new(&server.uri(), "test-token");
    sync_pull(&pool, &client).await.unwrap();

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM local_books")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 2);
}

#[tokio::test]
async fn test_push_retries_on_failure() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/books"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
        .mount(&server)
        .await;

    let pool = setup_db().await;
    let path = std::path::PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest =
        xcalibre_processing::pipeline::ingest::run_ingest(&pool, &path)
            .await
            .unwrap();

    let client = ApiClient::new(&server.uri(), "test-token");
    xcalibre_processing::pipeline::push::run_push(
        &pool,
        &ingest,
        serde_json::Value::Null,
        Some(&client),
    )
    .await
    .unwrap();

    let job: (String, i64) =
        sqlx::query_as("SELECT status, retry_count FROM jobs WHERE id = ?")
            .bind(&ingest.job_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(job.0, "RETRYING");
    assert_eq!(job.1, 1);
}
