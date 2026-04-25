use sqlx::sqlite::SqlitePoolOptions;
use std::io::Read;
use xcalibre_processing::db::{collection_queries, extended_queries, format_queries, fts_queries, queries};
use xcalibre_processing::{catalog, integrity, repair};
use xcalibre_processing::pipeline::conversion;
use xcalibre_processing::utils::sort::{author_sort, title_sort};

async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

async fn insert_book(pool: &sqlx::SqlitePool, id: &str, title: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO jobs (
            id, file_path, file_sha256, format, status, retry_count,
            next_retry_at, xs_book_id, push_step, error_message,
            created_at, updated_at
        ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(id)
    .bind(format!("/tmp/{id}.epub"))
    .bind(format!("sha-{id}"))
    .bind("EPUB")
    .bind("PENDING")
    .bind(0_i64)
    .bind::<Option<String>>(None)
    .bind::<Option<String>>(None)
    .bind(0_i64)
    .bind::<Option<String>>(None)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO local_books (
            id, title, authors_json, format, local_path, cover_path,
            reading_position, progress_percent, last_opened_at, created_at, updated_at
        ) VALUES (?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(id)
    .bind(title)
    .bind("[\"Author A\"]")
    .bind("EPUB")
    .bind::<Option<String>>(None)
    .bind::<Option<String>>(None)
    .bind::<Option<String>>(None)
    .bind(0_f64)
    .bind::<Option<String>>(None)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO books_fts (book_id, title, authors, description, full_text)
         VALUES (?,?,?,?,?)",
    )
    .bind(id)
    .bind(title)
    .bind("[\"Author A\"]")
    .bind("")
    .bind("")
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn test_fts_search_returns_results() {
    let pool = setup_db().await;
    insert_book(&pool, "b1", "Rust Programming Language").await;

    let results = queries::search_books(&pool, "Rust").await.unwrap();
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_collection_add_and_list() {
    let pool = setup_db().await;
    insert_book(&pool, "b1", "Test Book").await;

    let col_id = queries::create_collection(&pool, "Favourites")
        .await
        .unwrap();
    queries::add_book_to_collection(&pool, &col_id, "b1")
        .await
        .unwrap();

    let books = queries::get_collection_books(&pool, &col_id).await.unwrap();
    assert_eq!(books.len(), 1);
}

#[tokio::test]
async fn test_bulk_delete_removes_rows() {
    let pool = setup_db().await;
    for id in ["d1", "d2", "d3"] {
        insert_book(&pool, id, "Book").await;
    }

    for id in ["d1", "d2"] {
        sqlx::query("DELETE FROM local_books WHERE id=?")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM jobs WHERE id=?")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM local_books")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 1);
}

#[tokio::test]
async fn test_calibre_import_skips_duplicates() {
    let pool = setup_db().await;
    let tmpdir = tempfile::tempdir().unwrap();
    let src = std::path::PathBuf::from("tests/fixtures/fixture_epub.epub");
    let dst = tmpdir.path().join("fixture_epub.epub");
    std::fs::copy(&src, &dst).unwrap();

    let r1 = xcalibre_processing::import::calibre::import_calibre_library(&pool, tmpdir.path())
        .await
        .unwrap();
    let r2 = xcalibre_processing::import::calibre::import_calibre_library(&pool, tmpdir.path())
        .await
        .unwrap();

    assert_eq!(r1.queued, 1);
    assert_eq!(r2.skipped, 1);
}

#[tokio::test]
async fn test_sort_helpers() {
    assert_eq!(title_sort("The Great Gatsby"), "Great Gatsby, The");
    assert_eq!(author_sort("J.R.R. Tolkien"), "Tolkien, J.R.R.");
}

#[tokio::test]
async fn test_extended_metadata_tags_and_identifiers() {
    let pool = setup_db().await;
    queries::upsert_local_book(
        &pool,
        "b-tags",
        "The Sample Book",
        &["Alice Example".to_string()],
        "EPUB",
        Some("/tmp/sample.epub"),
        None,
    )
    .await
    .unwrap();

    extended_queries::upsert_tag(&pool, "b-tags", "fiction").await.unwrap();
    extended_queries::upsert_tag(&pool, "b-tags", "classic").await.unwrap();
    extended_queries::upsert_identifier(&pool, "b-tags", "isbn", "9780000000000")
        .await
        .unwrap();
    extended_queries::update_extended_metadata(
        &pool,
        "b-tags",
        Some("Sample Book, The"),
        Some("Example, Alice"),
        Some("2024-01-01"),
        Some("Description"),
        Some("Publisher"),
        Some("Series"),
        Some(2.0),
        Some(4),
    )
    .await
    .unwrap();

    let tags = extended_queries::get_tags(&pool, "b-tags").await.unwrap();
    assert_eq!(tags, vec!["classic".to_string(), "fiction".to_string()]);

    let identifiers = extended_queries::get_identifiers(&pool, "b-tags")
        .await
        .unwrap();
    assert_eq!(identifiers, vec![("isbn".to_string(), "9780000000000".to_string())]);
}

#[tokio::test]
async fn test_format_queries_crud() {
    let pool = setup_db().await;
    queries::upsert_local_book(
        &pool,
        "b-format",
        "Format Book",
        &["Formatter".to_string()],
        "EPUB",
        Some("/tmp/format.epub"),
        None,
    )
    .await
    .unwrap();

    let format_id = format_queries::insert_book_format(
        &pool,
        "b-format",
        "EPUB",
        "/tmp/format.epub",
        "sha-format",
        1234,
    )
    .await
    .unwrap();
    assert!(!format_id.is_empty());

    let formats = format_queries::get_formats_for_book(&pool, "b-format")
        .await
        .unwrap();
    assert_eq!(formats.len(), 1);
    assert_eq!(formats[0].format, "EPUB");
}

#[tokio::test]
async fn test_collection_queries_crud() {
    let pool = setup_db().await;
    queries::upsert_local_book(
        &pool,
        "b-col",
        "Collection Book",
        &["Collector".to_string()],
        "EPUB",
        Some("/tmp/collection.epub"),
        None,
    )
    .await
    .unwrap();

    let col_id = collection_queries::create_collection(&pool, "Sci-Fi")
        .await
        .unwrap();
    collection_queries::add_book_to_collection(&pool, "b-col", &col_id)
        .await
        .unwrap();

    let cols = collection_queries::list_collections(&pool).await.unwrap();
    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "Sci-Fi");

    let books = collection_queries::get_books_in_collection(&pool, &col_id)
        .await
        .unwrap();
    assert_eq!(books, vec!["b-col".to_string()]);
}

#[tokio::test]
async fn test_fts_queries_search() {
    let pool = setup_db().await;
    queries::upsert_local_book(
        &pool,
        "b-fts",
        "Fixture Search Book",
        &["Search Author".to_string()],
        "EPUB",
        Some("/tmp/fts.epub"),
        None,
    )
    .await
    .unwrap();

    fts_queries::upsert_fts(
        &pool,
        "b-fts",
        "Fixture Search Book",
        "Search Author",
        "",
        "This book contains fixture text about search.",
    )
    .await
    .unwrap();

    let results = fts_queries::search(&pool, "fixture").await.unwrap();
    assert_eq!(results, vec!["b-fts".to_string()]);
}

#[tokio::test]
async fn test_catalog_and_integrity_exports() {
    let pool = setup_db().await;
    let existing_path = tempfile::NamedTempFile::new().unwrap();
    let existing_path_str = existing_path.path().to_string_lossy().to_string();

    queries::upsert_local_book(
        &pool,
        "b-ok",
        "Catalog Book",
        &["Catalog Author".to_string()],
        "EPUB",
        Some(&existing_path_str),
        None,
    )
    .await
    .unwrap();
    queries::upsert_local_book(
        &pool,
        "b-missing",
        "Missing Book",
        &["Catalog Author".to_string()],
        "EPUB",
        Some("/tmp/does-not-exist.epub"),
        None,
    )
    .await
    .unwrap();

    let csv = catalog::export_csv(&pool).await.unwrap();
    assert!(csv.contains("Catalog Book"));
    assert!(csv.contains("Missing Book"));

    let html = catalog::export_html(&pool).await.unwrap();
    assert!(html.contains("xCalibre Library"));
    assert!(html.contains("Catalog Book"));

    let issues = integrity::check_integrity(&pool).await.unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].book_id, "b-missing");
}

#[tokio::test]
async fn test_metadata_edit_refreshes_search_and_extended_fields() {
    let pool = setup_db().await;
    let path = tempfile::NamedTempFile::new().unwrap();
    let path_str = path.path().to_string_lossy().to_string();

    queries::upsert_local_book(
        &pool,
        "b-edit",
        "Old Title",
        &["Old Author".to_string()],
        "EPUB",
        Some(path_str.as_str()),
        None,
    )
    .await
    .unwrap();

    extended_queries::replace_book_details(
        &pool,
        "b-edit",
        "New Title",
        &["New Author".to_string()],
        "Title, New",
        "Author, New",
        Some("2024-03-12"),
        Some("Searchable description"),
        Some("Publisher"),
        Some("Series"),
        Some(1.5),
        4,
    )
    .await
    .unwrap();
    extended_queries::replace_tags(&pool, "b-edit", &["fiction".to_string(), "featured".to_string()])
        .await
        .unwrap();
    extended_queries::replace_identifiers(
        &pool,
        "b-edit",
        &[extended_queries::BookIdentifier {
            id_type: "isbn".to_string(),
            value: "9780000000000".to_string(),
        }],
    )
    .await
    .unwrap();
    fts_queries::refresh_book_index(&pool, "b-edit").await.unwrap();

    let details = extended_queries::get_book_details(&pool, "b-edit")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(details.title, "New Title");
    assert_eq!(details.tags, vec!["featured".to_string(), "fiction".to_string()]);
    assert_eq!(details.identifiers[0].id_type, "isbn");

    let results = fts_queries::search(&pool, "Searchable").await.unwrap();
    assert_eq!(results, vec!["b-edit".to_string()]);
}

#[tokio::test]
async fn test_repair_rebuilds_missing_paths_and_cover() {
    let pool = setup_db().await;
    let source = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(source.path(), b"dummy").unwrap();

    queries::upsert_local_book(
        &pool,
        "b-repair",
        "Repair Book",
        &["Repair Author".to_string()],
        "EPUB",
        Some("/tmp/does-not-exist.epub"),
        None,
    )
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO book_formats (id, book_id, format, file_path, file_sha256, file_size)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind("fmt-repair")
    .bind("b-repair")
    .bind("EPUB")
    .bind(source.path().to_string_lossy().to_string())
    .bind("sha-repair")
    .bind(5_i64)
    .execute(&pool)
    .await
    .unwrap();

    let repaired = repair::repair_books(&pool, &["b-repair".to_string()])
        .await
        .unwrap();
    assert_eq!(repaired.len(), 1);

    let row: (Option<String>, Option<String>) = sqlx::query_as(
        "SELECT local_path, cover_path FROM local_books WHERE id = ?",
    )
    .bind("b-repair")
    .fetch_one(&pool)
    .await
    .unwrap();

    let expected_path = source.path().to_string_lossy().to_string();
    assert_eq!(row.0.as_deref(), Some(expected_path.as_str()));
    assert!(row.1.is_some());
    assert!(std::path::Path::new(row.1.as_ref().unwrap()).exists());
}

#[tokio::test]
async fn test_conversion_creates_epub_and_job_row() {
    let pool = setup_db().await;
    let source_path = tempfile::NamedTempFile::new().unwrap();
    let source_path_str = source_path.path().to_string_lossy().to_string();
    std::fs::copy("tests/fixtures/fixture_epub.epub", source_path.path()).unwrap();

    queries::upsert_local_book(
        &pool,
        "b-convert",
        "Convert Book",
        &["Convert Author".to_string()],
        "EPUB",
        Some(source_path_str.as_str()),
        None,
    )
    .await
    .unwrap();

    let outcome = conversion::convert_book_to_epub(&pool, "b-convert", true)
        .await
        .unwrap();
    assert_eq!(outcome.book_id, "b-convert");
    assert!(std::path::Path::new(&outcome.output_path).exists());

    let job: (String, String) = sqlx::query_as(
        "SELECT status, mode FROM conversion_jobs WHERE id = ?",
    )
    .bind(&outcome.job_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(job.0, "COMPLETED");
    assert_eq!(job.1, "TWEAK");

    let formats = format_queries::get_formats_for_book(&pool, "b-convert")
        .await
        .unwrap();
    assert!(formats.iter().any(|format| format.format == "EPUB"));

    let mut archive = zip::ZipArchive::new(std::fs::File::open(&outcome.output_path).unwrap())
        .unwrap();
    let mut opf = String::new();
    archive
        .by_name("OEBPS/content.opf")
        .unwrap()
        .read_to_string(&mut opf)
        .unwrap();
    assert!(opf.contains("Convert Book"));
}
