# Phase 2 — Full xcalibre-server Sync

> HOW TO USE: For every "Write `path`" line → call write_file with that path and content.
> For every "Then run:" block → call your shell tool for each command.
> DO NOT print code as output. Write it to disk using your tools.
> Prerequisite: Phase 1 complete and all tests green.
> Status: ✅ done · ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| P2-T01 | Config file (config.toml + env overrides) | ✅ |
| P2-T02 | Token store/retrieve CLI commands | ✅ |
| P2-T03 | xcalibre-server read API — get_book + ApiClient::new | ✅ |
| P2-T04 | xcalibre-server read API — list_books endpoint | ✅ |
| P2-T05 | Sync pull: fetch remote library into local_books | ✅ |
| P2-T06 | Push queue: background retry worker | ✅ |
| P2-T07 | Push queue: exponential back-off scheduling | ✅ |
| P2-T08 | Sync status back-propagation to DB | ✅ |
| P2-T09 | CLI: xcalibre sync command | ✅ |
| P2-T10 | Integration tests (mock HTTP server) | ✅ |

---

## P2-T01

In `processing/Cargo.toml`, add these lines under `[dependencies]`:
```toml
dirs = "5"
toml = "0.8"
```

In `processing/src/lib.rs`, add `pub mod config;` at the end of the file.

Write `processing/src/config.rs` with this exact content:
```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub xs_url: String,
    pub db_path:       PathBuf,
    pub cover_dir:     PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("xcalibre");
        Self {
            xs_url: "https://api.xcalibre.app".to_string(),
            db_path:       base.join("jobs.db"),
            cover_dir:     base.join("covers"),
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("xcalibre")
            .join("config.toml");

        let mut cfg: Config = if config_path.exists() {
            let text = std::fs::read_to_string(&config_path)?;
            toml::from_str::<Config>(&text)?
        } else {
            let default = Config::default();
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&config_path, toml::to_string(&default)?)?;
            default
        };

        if let Ok(url) = std::env::var("XCALIBRE_SERVER_URL") {
            cfg.xs_url = url;
        }
        if let Ok(p) = std::env::var("XCALIBRE_DB_PATH") {
            cfg.db_path = PathBuf::from(p);
        }
        Ok(cfg)
    }
}
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/config.rs processing/src/lib.rs processing/Cargo.toml
git commit -m "P2-T01: add Config struct with toml + env var loading"
```

---

## P2-T02

In `processing/Cargo.toml`, add `keyring = "2"` under `[dependencies]`.

Write `processing/src/main.rs` with this exact content:
```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "xcalibre", version, about = "xCalibre ebook processor")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest an ebook file into the local job database
    Ingest {
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
    /// Manage xcalibre-server authentication token
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
}

#[derive(Subcommand)]
enum AuthAction {
    /// Store a service token in the OS keychain
    SetToken {
        #[arg(value_name = "TOKEN")]
        token: String,
    },
    /// Show whether a token is stored (never prints the value)
    Status,
    /// Remove the stored token from the OS keychain
    RemoveToken,
}

const SERVICE: &str = "xcalibre";
const ACCOUNT: &str = "xs_token";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Ingest { path } => {
            println!("Ingesting: {}", path.display());
        }
        Commands::Auth { action } => match action {
            AuthAction::SetToken { token } => {
                let entry = keyring::Entry::new(SERVICE, ACCOUNT)
                    .map_err(|e| anyhow::anyhow!("keychain error: {}", e))?;
                entry
                    .set_password(&token)
                    .map_err(|e| anyhow::anyhow!("failed to store token: {}", e))?;
                println!("Token stored.");
            }
            AuthAction::Status => {
                let entry = keyring::Entry::new(SERVICE, ACCOUNT)
                    .map_err(|e| anyhow::anyhow!("keychain error: {}", e))?;
                match entry.get_password() {
                    Ok(_)  => println!("Token: stored"),
                    Err(_) => println!("Token: not set"),
                }
            }
            AuthAction::RemoveToken => {
                let entry = keyring::Entry::new(SERVICE, ACCOUNT)
                    .map_err(|e| anyhow::anyhow!("keychain error: {}", e))?;
                entry
                    .delete_credential()
                    .map_err(|e| anyhow::anyhow!("failed to remove token: {}", e))?;
                println!("Token removed.");
            }
        },
    }
    Ok(())
}
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/main.rs processing/Cargo.toml
git commit -m "P2-T02: add auth set-token/status/remove-token CLI subcommands"
```

---

## P2-T03

In `api/src/client.rs`, add the following method inside the `impl ApiClient` block, directly after the `from_keyring` function:
```rust
    pub fn new(base_url: &str, token: &str) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("failed to build http client");
        Self { base_url: base_url.to_string(), token: token.to_string(), http }
    }
```

In `api/src/lib.rs`, add `pub mod read;` at the end of the file.

Write `api/src/read.rs` with this exact content:
```rust
use crate::client::{ApiClient, ApiError};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct RemoteBook {
    pub id:      String,
    pub title:   String,
    pub authors: Vec<String>,
    pub format:  String,
    pub sha256:  String,
}

fn read_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("failed to build read client")
}

pub async fn get_book(client: &ApiClient, book_id: &str) -> Result<Option<RemoteBook>, ApiError> {
    let resp = read_client()
        .get(format!("{}/books/{}", client.base_url(), book_id))
        .bearer_auth(client.token())
        .send()
        .await?;

    if resp.status().as_u16() == 404 {
        return Ok(None);
    }
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(ApiError::Api { status, body });
    }
    Ok(Some(resp.json::<RemoteBook>().await?))
}
```

Then run:
```bash
cargo build --workspace
git add api/src/read.rs api/src/lib.rs api/src/client.rs
git commit -m "P2-T03: add get_book read endpoint (5s timeout) and ApiClient::new"
```

---

## P2-T04

Add the following to the end of `api/src/read.rs`. Do not remove anything already in the file.

```rust
pub async fn list_books(client: &ApiClient, page: u32) -> Result<Vec<RemoteBook>, ApiError> {
    let resp = read_client()
        .get(format!("{}/books?page={}", client.base_url(), page))
        .bearer_auth(client.token())
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(ApiError::Api { status, body });
    }
    Ok(resp.json::<Vec<RemoteBook>>().await?)
}
```

Then run:
```bash
cargo build --workspace
git add api/src/read.rs
git commit -m "P2-T04: add list_books paginated endpoint"
```

---

## P2-T05

In `processing/src/pipeline/mod.rs`, add `pub mod sync;` at the end of the file.

Write `processing/src/pipeline/sync.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use sqlx::SqlitePool;
use tracing::{info, warn};
use xcalibre_api::{client::ApiClient, read};

pub async fn sync_pull(pool: &SqlitePool, client: &ApiClient) -> Result<(), ProcessingError> {
    let mut page = 1u32;
    loop {
        let books = match read::list_books(client, page).await {
            Ok(b)  => b,
            Err(e) => { warn!("sync_pull page {} failed: {}", page, e); break; }
        };
        if books.is_empty() { break; }
        for book in &books {
            upsert_book(pool, book).await?;
        }
        info!("sync_pull page {} — {} books", page, books.len());
        page += 1;
    }
    Ok(())
}

async fn upsert_book(pool: &SqlitePool, book: &read::RemoteBook) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    let authors_json = serde_json::to_string(&book.authors)
        .map_err(|e| ProcessingError::DbError(sqlx::Error::Protocol(e.to_string())))?;
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, created_at, updated_at)
         VALUES (?,?,?,?,?,?)
         ON CONFLICT(id) DO UPDATE SET title=excluded.title,
                                        authors_json=excluded.authors_json,
                                        updated_at=excluded.updated_at",
    )
    .bind(&book.id)
    .bind(&book.title)
    .bind(&authors_json)
    .bind(&book.format)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn sync_status_backprop(
    pool: &SqlitePool,
    client: &ApiClient,
) -> Result<(), ProcessingError> {
    let jobs: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, xs_book_id FROM jobs
         WHERE status = 'COMPLETED' AND xs_book_id IS NOT NULL",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    for (job_id, book_id) in jobs {
        match read::get_book(client, &book_id).await {
            Ok(None) => {
                warn!(job_id = %job_id, book_id = %book_id, "remote book deleted — re-queuing");
                sqlx::query(
                    "UPDATE jobs SET status='READY_TO_PUSH', updated_at=datetime() WHERE id=?",
                )
                .bind(&job_id)
                .execute(pool)
                .await
                .map_err(ProcessingError::DbError)?;
            }
            Ok(Some(_)) => {}
            Err(e) => warn!(job_id = %job_id, error = %e, "get_book check failed — skipping"),
        }
    }
    Ok(())
}
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/pipeline/sync.rs processing/src/pipeline/mod.rs
git commit -m "P2-T05: add sync_pull with paginated upsert into local_books"
```

---

### ✅ Milestone check — after P2-T05
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```
Fix any issues before continuing.

---

## P2-T06

In `processing/src/pipeline/mod.rs`, add `pub mod retry;` at the end of the file.

Write `processing/src/pipeline/retry.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn};
use xcalibre_api::client::ApiClient;

fn next_retry_delay_minutes(retry_count: i64) -> i64 {
    2_i64.pow(retry_count.min(6) as u32).min(60)
}

pub async fn start_retry_worker(pool: Arc<SqlitePool>, client: Option<Arc<ApiClient>>) {
    let mut ticker = interval(Duration::from_secs(60));
    loop {
        ticker.tick().await;
        if let Err(e) = run_retries(&pool, client.as_deref()).await {
            warn!("retry worker error: {}", e);
        }
    }
}

pub async fn run_retries(
    pool: &SqlitePool,
    client: Option<&ApiClient>,
) -> Result<(), ProcessingError> {
    let Some(client) = client else {
        return Ok(());
    };

    let jobs: Vec<(String, String, String, i64)> = sqlx::query_as(
        "SELECT id, format, file_sha256, retry_count FROM jobs
         WHERE status = 'RETRYING'
           AND (next_retry_at IS NULL OR next_retry_at <= datetime('now'))",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    for (id, format, sha256, retry_count) in jobs {
        let meta: serde_json::Value = sqlx::query_as::<_, (String,)>(
            "SELECT metadata_json FROM job_metadata WHERE job_id = ?",
        )
        .bind(&id)
        .fetch_optional(pool)
        .await
        .map_err(ProcessingError::DbError)?
        .and_then(|(s,)| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Null);

        let payload = xcalibre_api::push::PushPayload {
            job_id:   id.clone(),
            format,
            sha256,
            metadata: meta,
        };

        match xcalibre_api::push::push_book(client, &payload).await {
            Ok(resp) => {
                sqlx::query(
                    "UPDATE jobs SET status='COMPLETED', xs_book_id=?, updated_at=datetime() WHERE id=?",
                )
                .bind(&resp.book_id)
                .bind(&id)
                .execute(pool)
                .await
                .map_err(ProcessingError::DbError)?;
                info!(job_id = %id, book_id = %resp.book_id, "retry push complete");
            }
            Err(e) => {
                warn!(job_id = %id, error = %e, "retry push failed");
                let new_count = retry_count + 1;
                if new_count >= 5 {
                    sqlx::query(
                        "UPDATE jobs SET status='FAILED', updated_at=datetime() WHERE id=?",
                    )
                    .bind(&id)
                    .execute(pool)
                    .await
                    .map_err(ProcessingError::DbError)?;
                } else {
                    let delay = next_retry_delay_minutes(retry_count);
                    let next = chrono::Utc::now() + chrono::Duration::minutes(delay);
                    sqlx::query(
                        "UPDATE jobs SET retry_count=?, next_retry_at=?, updated_at=datetime() WHERE id=?",
                    )
                    .bind(new_count)
                    .bind(next.to_rfc3339())
                    .bind(&id)
                    .execute(pool)
                    .await
                    .map_err(ProcessingError::DbError)?;
                }
            }
        }
    }
    Ok(())
}
```

Then run:
```bash
cargo build --workspace
git add processing/src/pipeline/retry.rs processing/src/pipeline/mod.rs
git commit -m "P2-T06: add background retry worker (60s poll interval)"
```

---

## P2-T07

The `next_retry_delay_minutes` function with exponential back-off was already included in P2-T06. No additional code changes needed.

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/pipeline/retry.rs
git commit -m "P2-T07: exponential back-off confirmed (2^n min, cap 60 min)"
```

---

## P2-T08

The `sync_status_backprop` function was already included in P2-T05. No additional code changes needed.

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/pipeline/sync.rs
git commit -m "P2-T08: re-queue completed jobs whose remote book was deleted"
```

---

## P2-T09

Write `processing/src/main.rs` with this exact content:
```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use xcalibre_processing::config::Config;

#[derive(Parser)]
#[command(name = "xcalibre", version, about = "xCalibre ebook processor")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest an ebook file into the local job database
    Ingest {
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
    /// Manage xcalibre-server authentication token
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
    /// Sync library with xcalibre-server
    Sync,
}

#[derive(Subcommand)]
enum AuthAction {
    /// Store a service token in the OS keychain
    SetToken {
        #[arg(value_name = "TOKEN")]
        token: String,
    },
    /// Show whether a token is stored (never prints the value)
    Status,
    /// Remove the stored token from the OS keychain
    RemoveToken,
}

const SERVICE: &str = "xcalibre";
const ACCOUNT: &str = "xs_token";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Ingest { path } => {
            println!("Ingesting: {}", path.display());
        }
        Commands::Auth { action } => match action {
            AuthAction::SetToken { token } => {
                let entry = keyring::Entry::new(SERVICE, ACCOUNT)
                    .map_err(|e| anyhow::anyhow!("keychain error: {}", e))?;
                entry
                    .set_password(&token)
                    .map_err(|e| anyhow::anyhow!("failed to store token: {}", e))?;
                println!("Token stored.");
            }
            AuthAction::Status => {
                let entry = keyring::Entry::new(SERVICE, ACCOUNT)
                    .map_err(|e| anyhow::anyhow!("keychain error: {}", e))?;
                match entry.get_password() {
                    Ok(_)  => println!("Token: stored"),
                    Err(_) => println!("Token: not set"),
                }
            }
            AuthAction::RemoveToken => {
                let entry = keyring::Entry::new(SERVICE, ACCOUNT)
                    .map_err(|e| anyhow::anyhow!("keychain error: {}", e))?;
                entry
                    .delete_credential()
                    .map_err(|e| anyhow::anyhow!("failed to remove token: {}", e))?;
                println!("Token removed.");
            }
        },
        Commands::Sync => {
            let config = match Config::load() {
                Ok(c)  => c,
                Err(e) => {
                    eprintln!("Config error: {}", e);
                    std::process::exit(1);
                }
            };
            let client = match xcalibre_api::client::ApiClient::from_keyring(&config.xs_url)
            {
                Ok(c) => c,
                Err(_) => {
                    println!(
                        "No xcalibre-server token. Run `xcalibre auth set-token <TOKEN>` first."
                    );
                    return Ok(());
                }
            };
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .connect(&format!("sqlite:{}", config.db_path.display()))
                .await?;
            sqlx::migrate!("src/db/migrations").run(&pool).await?;
            xcalibre_processing::pipeline::sync::sync_pull(&pool, &client).await?;
            xcalibre_processing::pipeline::sync::sync_status_backprop(&pool, &client).await?;
            xcalibre_processing::pipeline::retry::run_retries(&pool, Some(&client)).await?;
            println!("Sync complete.");
        }
    }
    Ok(())
}
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/main.rs
git commit -m "P2-T09: add xcalibre sync CLI subcommand"
```

---

## P2-T10

In `processing/Cargo.toml`, add the following under `[dev-dependencies]`:
```toml
wiremock = "0.6"
```

Write `processing/tests/test_sync.rs` with this exact content:
```rust
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
        .and(path("/books"))
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
```

Then run:
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo audit
git add processing/tests/test_sync.rs processing/Cargo.toml
git commit -m "P2-T10: add sync + retry integration tests with wiremock"
```

### ✅ Milestone check — after P2-T10
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```
Fix any issues before continuing.
