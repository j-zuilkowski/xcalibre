use tauri::Manager;
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentifierInput {
    pub id_type: String,
    pub value:   String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookEditRequest {
    pub title:        String,
    pub authors:      Vec<String>,
    pub pubdate:      String,
    pub description:  String,
    pub publisher:    String,
    pub series_name:  String,
    pub series_index: Option<f64>,
    pub rating:       Option<i64>,
    pub tags:         Vec<String>,
    pub identifiers:  Vec<IdentifierInput>,
}

type BookRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    f64,
    Option<String>,
    Option<String>,
);

fn book_row_to_value(
    (
        id,
        title,
        authors_json,
        format,
        cover_path,
        local_path,
        progress_percent,
        last_opened_at,
        reading_cfi,
    ): BookRow,
) -> serde_json::Value {
    let authors: Vec<String> = serde_json::from_str(&authors_json).unwrap_or_default();
    serde_json::json!({
        "id": id,
        "title": title,
        "authors": authors,
        "format": format,
        "cover_path": cover_path,
        "local_path": local_path,
        "progress_percent": progress_percent,
        "last_opened_at": last_opened_at,
        "reading_cfi": reading_cfi,
    })
}

async fn fetch_books(pool: &SqlitePool) -> Result<Vec<serde_json::Value>, String> {
    let rows = sqlx::query_as::<_, BookRow>(
        "SELECT id, title, authors_json, format, cover_path, local_path, progress_percent,
                last_opened_at, reading_cfi
         FROM local_books
         ORDER BY last_opened_at IS NULL, last_opened_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(
            |(id, title, authors_json, format, cover_path, local_path, progress_percent, last_opened_at, reading_cfi)| {
                book_row_to_value((
                    id,
                    title,
                    authors_json,
                    format,
                    cover_path,
                    local_path,
                    progress_percent,
                    last_opened_at,
                    reading_cfi,
                ))
            },
        )
        .collect())
}

async fn fetch_book_by_id(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<serde_json::Value>, String> {
    let row = sqlx::query_as::<_, BookRow>(
        "SELECT id, title, authors_json, format, cover_path, local_path, progress_percent,
                last_opened_at, reading_cfi
         FROM local_books
         WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(
        |row| {
            book_row_to_value((
                row.0,
                row.1,
                row.2,
                row.3,
                row.4,
                row.5,
                row.6,
                row.7,
                row.8,
            ))
        },
    ))
}

#[tauri::command]
pub async fn get_book_details(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
) -> Result<serde_json::Value, String> {
    let details = xcalibre_processing::db::extended_queries::get_book_details(
        pool.inner().as_ref(),
        &book_id,
    )
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "book not found".to_string())?;

    serde_json::to_value(details).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_book_details(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_ids: Vec<String>,
    details: BookEditRequest,
) -> Result<usize, String> {
    if book_ids.is_empty() {
        return Ok(0);
    }

    let title = details.title.trim();
    if title.is_empty() {
        return Err("title cannot be empty".to_string());
    }

    let authors: Vec<String> = details
        .authors
        .into_iter()
        .map(|author| author.trim().to_string())
        .filter(|author| !author.is_empty())
        .collect();
    let title_sort_value = xcalibre_processing::utils::sort::title_sort(title);
    let author_sort_value = authors
        .first()
        .map(|author| xcalibre_processing::utils::sort::author_sort(author))
        .unwrap_or_default();
    let rating = details.rating.unwrap_or(0).clamp(0, 5);
    let pubdate = trim_to_option(details.pubdate);
    let description = trim_to_option(details.description);
    let publisher = trim_to_option(details.publisher);
    let series_name = trim_to_option(details.series_name);
    let identifiers: Vec<xcalibre_processing::db::extended_queries::BookIdentifier> = details
        .identifiers
        .into_iter()
        .map(|identifier| xcalibre_processing::db::extended_queries::BookIdentifier {
            id_type: identifier.id_type.trim().to_string(),
            value: identifier.value.trim().to_string(),
        })
        .filter(|identifier| !identifier.id_type.is_empty() && !identifier.value.is_empty())
        .collect();
    let tag_values: Vec<String> = details
        .tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect();

    for book_id in &book_ids {
        xcalibre_processing::db::extended_queries::replace_book_details(
            pool.inner().as_ref(),
            book_id,
            title,
            &authors,
            &title_sort_value,
            &author_sort_value,
            pubdate.as_deref(),
            description.as_deref(),
            publisher.as_deref(),
            series_name.as_deref(),
            details.series_index,
            rating,
        )
        .await
        .map_err(|e| e.to_string())?;
        xcalibre_processing::db::extended_queries::replace_tags(
            pool.inner().as_ref(),
            book_id,
            &tag_values,
        )
        .await
        .map_err(|e| e.to_string())?;
        xcalibre_processing::db::extended_queries::replace_identifiers(
            pool.inner().as_ref(),
            book_id,
            &identifiers,
        )
        .await
        .map_err(|e| e.to_string())?;
        xcalibre_processing::db::fts_queries::refresh_book_index(
            pool.inner().as_ref(),
            book_id,
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(book_ids.len())
}

#[tauri::command]
pub async fn repair_books(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_ids: Vec<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let results = xcalibre_processing::repair::repair_books(pool.inner().as_ref(), &book_ids)
        .await
        .map_err(|e| e.to_string())?;
    results
        .into_iter()
        .map(|result| serde_json::to_value(result).map_err(|e| e.to_string()))
        .collect()
}

#[tauri::command]
pub async fn convert_book_to_epub(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
    tweak: bool,
) -> Result<serde_json::Value, String> {
    let result = xcalibre_processing::pipeline::conversion::convert_book_to_epub(
        pool.inner().as_ref(),
        &book_id,
        tweak,
    )
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_value(result).map_err(|e| e.to_string())
}

fn trim_to_option(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[tauri::command]
pub async fn list_books(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<Vec<serde_json::Value>, String> {
    fetch_books(pool.inner().as_ref()).await
}

#[tauri::command]
pub async fn get_library(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<Vec<serde_json::Value>, String> {
    fetch_books(pool.inner().as_ref()).await
}

#[tauri::command]
pub async fn filter_library(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    format: Option<String>,
    status: Option<String>,
    author: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let rows = sqlx::query_as::<_, BookRow>(
        "SELECT lb.id, lb.title, lb.authors_json, lb.format, lb.cover_path,
                lb.local_path, lb.progress_percent, lb.last_opened_at, lb.reading_cfi
         FROM local_books lb
         JOIN jobs j ON j.id = lb.id
         WHERE (? IS NULL OR lb.format = ?)
           AND (? IS NULL OR j.status = ?)
           AND (? IS NULL OR lb.authors_json LIKE '%' || ? || '%')
         ORDER BY lb.last_opened_at IS NULL, lb.last_opened_at DESC",
    )
    .bind(&format)
    .bind(&format)
    .bind(&status)
    .bind(&status)
    .bind(&author)
    .bind(&author)
    .fetch_all(pool.inner().as_ref())
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(
            |row| {
                book_row_to_value((
                    row.0,
                    row.1,
                    row.2,
                    row.3,
                    row.4,
                    row.5,
                    row.6,
                    row.7,
                    row.8,
                ))
            },
        )
        .collect())
}

#[tauri::command]
pub async fn search_library(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    query: String,
) -> Result<Vec<serde_json::Value>, String> {
    let jobs =
        xcalibre_processing::db::queries::search_books(pool.inner().as_ref(), &query)
            .await
            .map_err(|e| e.to_string())?;

    let mut books = Vec::with_capacity(jobs.len());
    for job in jobs {
        if let Some(book) = fetch_book_by_id(pool.inner().as_ref(), &job.id).await? {
            books.push(book);
        }
    }
    Ok(books)
}

#[tauri::command]
pub async fn ingest_file(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    path: String,
) -> Result<String, String> {
    let path = std::path::Path::new(&path);
    let result = xcalibre_processing::pipeline::local::import_local_book(
        pool.inner().as_ref(),
        path,
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(result.job_id)
}

#[tauri::command]
pub async fn get_spine(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    job_id: String,
) -> Result<Vec<String>, String> {
    xcalibre_processing::commands::get_spine(pool.inner().as_ref(), job_id).await
}

#[tauri::command]
pub async fn get_epub_chapter_html(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
    href: String,
) -> Result<String, String> {
    eprintln!("get_epub_chapter_html:start book_id={} href={}", book_id, href);
    let file_path: Option<String> = sqlx::query_as::<_, (String,)>("SELECT file_path FROM jobs WHERE id = ?")
        .bind(&book_id)
        .fetch_optional(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?
        .map(|(path,)| path);

    let file_path = file_path.ok_or_else(|| "job not found".to_string())?;
    let rendered = crate::epub_protocol::render_html(&file_path, &book_id, &href)
        .map_err(|e| e.to_string())?;
    eprintln!(
        "get_epub_chapter_html:done book_id={} href={} bytes={}",
        book_id,
        href,
        rendered.len()
    );
    Ok(rendered)
}

#[tauri::command]
pub async fn update_position(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
    position: String,
) -> Result<(), String> {
    xcalibre_processing::db::queries::update_reading_position(
        pool.inner().as_ref(),
        &book_id,
        &position,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_progress(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
    percent: f64,
) -> Result<(), String> {
    xcalibre_processing::db::queries::update_progress_percent(
        pool.inner().as_ref(),
        &book_id,
        percent,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_bookmark(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
    cfi: String,
    label: Option<String>,
) -> Result<String, String> {
    xcalibre_processing::db::queries::add_bookmark(
        pool.inner().as_ref(),
        &book_id,
        &cfi,
        label.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_bookmarks(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let bookmarks = xcalibre_processing::db::queries::list_bookmarks(
        pool.inner().as_ref(),
        &book_id,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(bookmarks
        .into_iter()
        .map(|b| serde_json::json!({
            "id": b.id,
            "book_id": b.book_id,
            "cfi": b.cfi,
            "label": b.label,
            "created_at": b.created_at,
        }))
        .collect())
}

#[tauri::command]
pub async fn delete_bookmark(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    bookmark_id: String,
) -> Result<(), String> {
    xcalibre_processing::db::queries::delete_bookmark(
        pool.inner().as_ref(),
        &bookmark_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_collection_cmd(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    name: String,
) -> Result<String, String> {
    xcalibre_processing::db::queries::create_collection(pool.inner().as_ref(), &name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_collections_cmd(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<Vec<serde_json::Value>, String> {
    let collections = xcalibre_processing::db::queries::list_collections(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;

    Ok(collections
        .into_iter()
        .map(|collection| {
            serde_json::json!({
                "id": collection.id,
                "name": collection.name,
                "created_at": collection.created_at,
            })
        })
        .collect())
}

#[tauri::command]
pub async fn add_book_to_collection_cmd(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    collection_id: String,
    book_id: String,
) -> Result<(), String> {
    xcalibre_processing::db::queries::add_book_to_collection(
        pool.inner().as_ref(),
        &collection_id,
        &book_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_book_from_collection_cmd(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    collection_id: String,
    book_id: String,
) -> Result<(), String> {
    xcalibre_processing::db::queries::remove_book_from_collection(
        pool.inner().as_ref(),
        &collection_id,
        &book_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_collection_books_cmd(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    collection_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let jobs = xcalibre_processing::db::queries::get_collection_books(
        pool.inner().as_ref(),
        &collection_id,
    )
    .await
    .map_err(|e| e.to_string())?;

    let mut books = Vec::with_capacity(jobs.len());
    for job in jobs {
        if let Some(book) = fetch_book_by_id(pool.inner().as_ref(), &job.id).await? {
            books.push(book);
        }
    }

    Ok(books)
}

#[tauri::command]
pub async fn bulk_reingest(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_ids: Vec<String>,
) -> Result<(), String> {
    let _ = bulk_reingest_books(pool, book_ids).await?;
    Ok(())
}

#[tauri::command]
pub async fn bulk_delete(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_ids: Vec<String>,
) -> Result<(), String> {
    let _ = bulk_delete_books(pool, book_ids).await?;
    Ok(())
}

#[tauri::command]
pub async fn export_metadata(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_ids: Vec<String>,
) -> Result<String, String> {
    let mut results = Vec::new();
    for id in &book_ids {
        let meta: Option<(String,)> = sqlx::query_as(
            "SELECT metadata_json FROM job_metadata WHERE job_id=?",
        )
        .bind(id)
        .fetch_optional(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;

        if let Some((json,)) = meta {
            results.push(serde_json::from_str::<serde_json::Value>(&json).unwrap_or_default());
        }
    }

    serde_json::to_string_pretty(&results).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_file(path: String, contents: String) -> Result<(), String> {
    if let Some(parent) = std::path::Path::new(&path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    std::fs::write(path, contents).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_calibre(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    library_path: String,
) -> Result<serde_json::Value, String> {
    let result = xcalibre_processing::import::calibre::import_calibre_library(
        pool.inner().as_ref(),
        std::path::Path::new(&library_path),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "found": result.found,
        "imported": result.imported,
        "queued": result.queued,
        "skipped": result.skipped,
        "errors": result.errors,
    }))
}

#[tauri::command]
pub async fn save_config(autolib_url: Option<String>) -> Result<(), String> {
    let mut config = xcalibre_processing::config::Config::load()
        .map_err(|e| e.to_string())?;
    if let Some(url) = autolib_url {
        config.xs_url = url;
    }
    let dirs_config = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("xcalibre")
        .join("config.toml");
    if let Some(parent) = dirs_config.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let toml_str = toml::to_string(&config).map_err(|e| e.to_string())?;
    std::fs::write(&dirs_config, toml_str).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_xs_url() -> Result<Option<String>, String> {
    let config = xcalibre_processing::config::Config::load().map_err(|e| e.to_string())?;
    Ok(Some(config.xs_url))
}

#[tauri::command]
pub async fn has_token() -> Result<bool, String> {
    let entry = keyring::Entry::new("xcalibre", "xs_token")
        .map_err(|e| e.to_string())?;
    Ok(entry.get_password().is_ok())
}

#[tauri::command]
pub async fn search_books(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    query: String,
) -> Result<Vec<serde_json::Value>, String> {
    let book_ids = xcalibre_processing::db::fts_queries::search(
        pool.inner().as_ref(),
        &query,
    )
    .await
    .map_err(|e| e.to_string())?;

    if book_ids.is_empty() {
        return Ok(vec![]);
    }

    let placeholders = book_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT id, title, authors_json, format, cover_path, local_path, progress_percent, last_opened_at, reading_cfi
         FROM local_books WHERE id IN ({})
         ORDER BY COALESCE(title_sort, title), title",
        placeholders
    );
    let mut q = sqlx::query_as::<_, BookRow>(&sql);
    for id in &book_ids {
        q = q.bind(id);
    }
    let rows = q
        .fetch_all(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(book_row_to_value).collect())
}

#[tauri::command]
pub async fn filter_books(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    format: Option<String>,
    status: Option<String>,
    author: Option<String>,
    tag: Option<String>,
    series: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let mut conditions = vec!["1=1".to_string()];
    let mut binds: Vec<String> = vec![];
    let mut joins = vec!["JOIN jobs j ON j.id = lb.id".to_string()];

    if let Some(ref f) = format {
        conditions.push("lb.format = ?".to_string());
        binds.push(f.clone());
    }
    if let Some(ref s) = status {
        conditions.push("j.status = ?".to_string());
        binds.push(s.clone());
    }
    if let Some(ref a) = author {
        conditions.push("lb.authors_json LIKE ?".to_string());
        binds.push(format!("%{}%", a));
    }
    if let Some(ref s) = series {
        conditions.push("lb.series_name = ?".to_string());
        binds.push(s.clone());
    }

    if let Some(ref t) = tag {
        joins.push("JOIN book_tags bt ON bt.book_id = lb.id".to_string());
        joins.push("JOIN tags t ON t.id = bt.tag_id".to_string());
        conditions.push("t.name = ?".to_string());
        binds.push(t.clone());
    }

    let sql = format!(
        "SELECT DISTINCT lb.id, lb.title, lb.authors_json, lb.format, lb.cover_path,
                lb.local_path, lb.progress_percent, lb.last_opened_at, lb.reading_cfi
         FROM local_books lb {}
         WHERE {} ORDER BY COALESCE(lb.title_sort, lb.title), lb.title",
        joins.join(" "),
        conditions.join(" AND ")
    );

    let mut q = sqlx::query_as::<_, BookRow>(&sql);
    for b in &binds {
        q = q.bind(b.as_str());
    }
    let rows = q
        .fetch_all(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(book_row_to_value).collect())
}

#[tauri::command]
pub async fn list_tags(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query_as::<_, (String,)>("SELECT name FROM tags ORDER BY name")
        .fetch_all(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|(n,)| n).collect())
}

#[tauri::command]
pub async fn list_series(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT series_name FROM local_books
         WHERE series_name IS NOT NULL AND series_name <> ''
         ORDER BY series_name",
    )
    .fetch_all(pool.inner().as_ref())
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|(n,)| n).collect())
}

#[tauri::command]
pub async fn list_authors(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT authors_json FROM local_books ORDER BY COALESCE(author_sort, title_sort, title)",
    )
    .fetch_all(pool.inner().as_ref())
    .await
    .map_err(|e| e.to_string())?;
    let mut authors: Vec<String> = rows
        .into_iter()
        .flat_map(|(json,)| serde_json::from_str::<Vec<String>>(&json).unwrap_or_default())
        .collect();
    authors.sort();
    authors.dedup();
    Ok(authors)
}

#[tauri::command]
pub async fn create_collection(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    name: String,
) -> Result<String, String> {
    xcalibre_processing::db::collection_queries::create_collection(pool.inner().as_ref(), &name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_collections(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<Vec<serde_json::Value>, String> {
    let cols = xcalibre_processing::db::collection_queries::list_collections(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;
    Ok(cols.into_iter().map(|c| serde_json::json!({ "id": c.id, "name": c.name })).collect())
}

#[tauri::command]
pub async fn delete_collection(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    id: String,
) -> Result<(), String> {
    xcalibre_processing::db::collection_queries::delete_collection(pool.inner().as_ref(), &id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_book_to_collection(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
    collection_id: String,
) -> Result<(), String> {
    xcalibre_processing::db::collection_queries::add_book_to_collection(
        pool.inner().as_ref(),
        &book_id,
        &collection_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_book_from_collection(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
    collection_id: String,
) -> Result<(), String> {
    xcalibre_processing::db::collection_queries::remove_book_from_collection(
        pool.inner().as_ref(),
        &book_id,
        &collection_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_books_in_collection(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    collection_id: String,
) -> Result<Vec<String>, String> {
    xcalibre_processing::db::collection_queries::get_books_in_collection(
        pool.inner().as_ref(),
        &collection_id,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bulk_delete_books(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_ids: Vec<String>,
) -> Result<u64, String> {
    let mut count = 0u64;
    for id in &book_ids {
        let cover: Option<(Option<String>,)> = sqlx::query_as(
            "SELECT cover_path FROM local_books WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;

        if let Some((Some(path),)) = cover {
            if !path.is_empty() {
                if let Err(err) = std::fs::remove_file(&path) {
                    log::warn!("could not delete cover {}: {}", path, err);
                }
            }
        }

        let _ = sqlx::query("DELETE FROM books_fts WHERE book_id = ?")
            .bind(id)
            .execute(pool.inner().as_ref())
            .await;
        let _ = sqlx::query("DELETE FROM job_text WHERE job_id = ?")
            .bind(id)
            .execute(pool.inner().as_ref())
            .await;
        let _ = sqlx::query("DELETE FROM book_formats WHERE book_id = ?")
            .bind(id)
            .execute(pool.inner().as_ref())
            .await;
        let _ = sqlx::query("DELETE FROM book_tags WHERE book_id = ?")
            .bind(id)
            .execute(pool.inner().as_ref())
            .await;
        let _ = sqlx::query("DELETE FROM identifiers WHERE book_id = ?")
            .bind(id)
            .execute(pool.inner().as_ref())
            .await;
        let _ = sqlx::query("DELETE FROM collection_books WHERE book_id = ?")
            .bind(id)
            .execute(pool.inner().as_ref())
            .await;

        let result = sqlx::query("DELETE FROM local_books WHERE id = ?")
            .bind(id)
            .execute(pool.inner().as_ref())
            .await
            .map_err(|e| e.to_string())?;
        count += result.rows_affected();

        let _ = sqlx::query("DELETE FROM push_queue WHERE job_id = ?")
            .bind(id)
            .execute(pool.inner().as_ref())
            .await;
        let _ = sqlx::query("DELETE FROM jobs WHERE id = ?")
            .bind(id)
            .execute(pool.inner().as_ref())
            .await;
    }
    Ok(count)
}

#[tauri::command]
pub async fn bulk_reingest_books(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_ids: Vec<String>,
) -> Result<Vec<String>, String> {
    for id in &book_ids {
        sqlx::query("UPDATE jobs SET status='PENDING', updated_at=datetime() WHERE id=?")
            .bind(id)
            .execute(pool.inner().as_ref())
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(book_ids)
}

#[tauri::command]
pub async fn bulk_export_metadata(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_ids: Vec<String>,
) -> Result<String, String> {
    if book_ids.is_empty() {
        return Ok("id,title,authors,format,publisher,series,series_index,pubdate,description\n".to_string());
    }

    let placeholders = book_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT id, title, authors_json, format, publisher, series_name, series_index, pubdate, description
         FROM local_books WHERE id IN ({}) ORDER BY COALESCE(title_sort, title), title",
        placeholders
    );
    let mut q = sqlx::query_as::<_, (
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<f64>,
        Option<String>,
        Option<String>,
    )>(&sql);
    for id in &book_ids {
        q = q.bind(id);
    }
    let rows = q
        .fetch_all(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;

    let mut csv = "id,title,authors,format,publisher,series,series_index,pubdate,description\n".to_string();
    for (id, title, authors_json, format, publisher, series_name, series_index, pubdate, description) in rows {
        let authors: Vec<String> = serde_json::from_str(&authors_json).unwrap_or_default();
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            id,
            title.replace(',', ";"),
            authors.join(";").replace(',', ";"),
            format,
            publisher.unwrap_or_default(),
            series_name.unwrap_or_default(),
            series_index.map(|f| f.to_string()).unwrap_or_default(),
            pubdate.unwrap_or_default(),
            description.unwrap_or_default(),
        ));
    }
    Ok(csv)
}

#[tauri::command]
pub async fn check_library_integrity(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<Vec<serde_json::Value>, String> {
    let issues = xcalibre_processing::integrity::check_integrity(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())?;
    Ok(issues
        .into_iter()
        .map(|i| serde_json::json!({
            "book_id": i.book_id,
            "file_path": i.file_path,
            "issue": i.issue,
        }))
        .collect())
}

#[tauri::command]
pub async fn export_library_csv(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<String, String> {
    xcalibre_processing::catalog::export_csv(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_library_html(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<String, String> {
    xcalibre_processing::catalog::export_html(pool.inner().as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_annotation(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
    annotation_type: String,
    cfi: String,
    selected_text: Option<String>,
    note: Option<String>,
    color: String,
) -> Result<String, String> {
    xcalibre_processing::db::annotation_queries::create_annotation(
        pool.inner().as_ref(),
        &book_id,
        &annotation_type,
        &cfi,
        selected_text.as_deref(),
        note.as_deref(),
        &color,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_annotations(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let annotations = xcalibre_processing::db::annotation_queries::get_annotations(
        pool.inner().as_ref(),
        &book_id,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(annotations
        .into_iter()
        .map(|a| serde_json::json!({
            "id": a.id,
            "book_id": a.book_id,
            "type": a.annotation_type,
            "cfi": a.cfi,
            "selected_text": a.selected_text,
            "note": a.note,
            "color": a.color,
            "synced": a.synced,
            "created_at": a.created_at,
        }))
        .collect())
}

#[tauri::command]
pub async fn delete_annotation(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    id: String,
) -> Result<(), String> {
    xcalibre_processing::db::annotation_queries::delete_annotation(pool.inner().as_ref(), &id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_annotation_note(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    id: String,
    note: String,
) -> Result<(), String> {
    xcalibre_processing::db::annotation_queries::update_annotation_note(
        pool.inner().as_ref(),
        &id,
        &note,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_in_os(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_comic_pages(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
) -> Result<Vec<String>, String> {
    let row = sqlx::query_as::<_, (Option<String>,)>(
        "SELECT COALESCE(local_path, (SELECT file_path FROM jobs WHERE id = local_books.id))
         FROM local_books WHERE id = ?",
    )
    .bind(&book_id)
    .fetch_optional(pool.inner().as_ref())
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "book not found".to_string())?;

    let file_path = row
        .0
        .ok_or_else(|| "book has no local file path".to_string())?;
    let path = std::path::Path::new(&file_path);

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext != "cbz" {
        return Err("Only CBZ is supported for in-app viewing; use open_in_os for CBR".to_string());
    }

    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let archive = zip::ZipArchive::new(std::io::BufReader::new(file)).map_err(|e| e.to_string())?;

    let temp_dir = std::env::temp_dir().join(format!("xcalibre_comic_{}", book_id));
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let mut pages: Vec<String> = vec![];
    let mut archive = archive;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        let lower = name.to_lowercase();
        if lower.ends_with(".jpg") || lower.ends_with(".jpeg")
            || lower.ends_with(".png") || lower.ends_with(".webp")
        {
            let out_path = temp_dir.join(
                std::path::Path::new(&name)
                    .file_name()
                    .unwrap_or(std::ffi::OsStr::new(&name)),
            );
            let mut buf = vec![];
            std::io::Read::read_to_end(&mut entry, &mut buf).map_err(|e| e.to_string())?;
            std::fs::write(&out_path, &buf).map_err(|e| e.to_string())?;
            pages.push(out_path.to_string_lossy().into_owned());
        }
    }
    pages.sort();
    Ok(pages)
}

#[tauri::command]
pub async fn sync_annotations(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<usize, String> {
    let client = xcalibre_api::client::ApiClient::from_keyring("https://api.xcalibre.app").ok();
    xcalibre_processing::pipeline::sync::sync_annotations(
        pool.inner().as_ref(),
        client.as_ref(),
    )
    .await
    .map_err(|e| e.to_string())
}

use xcalibre_processing::db::library_queries::{
    create_library, delete_library, get_active_library,
    list_libraries, set_active_library, NewLibrary, LibraryRow,
};

#[tauri::command]
pub async fn list_libraries_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
) -> Result<Vec<LibraryRow>, String> {
    list_libraries(pool.inner().as_ref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_library_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    name: String,
    db_path: String,
    cover_dir: String,
    layout: String,
    xs_url: Option<String>,
) -> Result<String, String> {
    let lib = NewLibrary { name, db_path, cover_dir, layout, xs_url };
    create_library(pool.inner().as_ref(), &lib).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_active_library_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    id: String,
) -> Result<(), String> {
    set_active_library(pool.inner().as_ref(), &id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_active_library_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
) -> Result<Option<LibraryRow>, String> {
    get_active_library(pool.inner().as_ref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_library_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    id: String,
) -> Result<(), String> {
    delete_library(pool.inner().as_ref(), &id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_app_data_dir(
    app: tauri::AppHandle,
) -> Result<String, String> {
    app.path().app_data_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_library_advanced(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    query: String,
) -> Result<Vec<String>, String> {
    xcalibre_processing::search::execute_query(pool.inner().as_ref(), &query)
        .await
        .map_err(|e| e.to_string())
}
