use xcalibre_processing::db::queries::{self, NewJob, Job};
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};

async fn setup_db() -> Pool<Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("src/db/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

#[tokio::test]
async fn test_create_and_fetch_job() {
    let pool = setup_db().await;

    let mut job = NewJob::new("/tmp/test.epub", "EPUB");
    job.file_sha256 = "dummy_sha256_hash_for_testing".to_string();

    let job_id: String = queries::create_job(&pool, &job)
        .await
        .expect("Failed to create job");

    let fetched_job: Job = queries::get_job(&pool, &job_id)
        .await
        .expect("Failed to fetch job")
        .expect("Job not found");

    assert_eq!(fetched_job.id, job_id);
    assert_eq!(fetched_job.file_path, job.file_path);
    assert_eq!(fetched_job.file_sha256, job.file_sha256);
    assert_eq!(fetched_job.format, job.format);
    assert_eq!(fetched_job.status, job.status);
    assert_eq!(fetched_job.retry_count, job.retry_count);
    assert_eq!(fetched_job.next_retry_at, job.next_retry_at);
    assert_eq!(fetched_job.xs_book_id, job.xs_book_id);
    assert_eq!(fetched_job.push_step, job.push_step);
    assert_eq!(fetched_job.error_message, job.error_message);
    assert!(!fetched_job.created_at.is_empty());
    assert!(!fetched_job.updated_at.is_empty());
}

#[tokio::test]
async fn test_update_job_status() {
    let pool = setup_db().await;

    let mut job = NewJob::new("/tmp/test.epub", "EPUB");
    job.file_sha256 = "dummy_sha256_hash_for_testing".to_string();

    let job_id: String = queries::create_job(&pool, &job)
        .await
        .expect("Failed to create job");

    queries::update_job_status(&pool, &job_id, "COMPLETED")
        .await
        .expect("Failed to update job status");

    let fetched_job: Job = queries::get_job(&pool, &job_id)
        .await
        .expect("Failed to fetch job")
        .expect("Job not found");

    assert_eq!(fetched_job.status, "COMPLETED");
}

#[tokio::test]
async fn test_list_jobs_by_status() {
    let pool = setup_db().await;

    let mut job1 = NewJob::new("/tmp/test1.epub", "EPUB");
    job1.file_sha256 = "sha1".to_string();
    let mut job2 = NewJob::new("/tmp/test2.epub", "EPUB");
    job2.file_sha256 = "sha2".to_string();
    let mut job3 = NewJob::new("/tmp/test3.epub", "EPUB");
    job3.file_sha256 = "sha3".to_string();
    job3.status = "COMPLETED".to_string();

    queries::create_job(&pool, &job1).await.expect("Failed to create job1");
    queries::create_job(&pool, &job2).await.expect("Failed to create job2");
    queries::create_job(&pool, &job3).await.expect("Failed to create job3");

    let pending_jobs: Vec<Job> = queries::list_jobs_by_status(&pool, "PENDING")
        .await
        .expect("Failed to list jobs by status");

    assert_eq!(pending_jobs.len(), 2);
    for job in pending_jobs {
        assert_eq!(job.status, "PENDING");
    }
}
