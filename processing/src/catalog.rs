use crate::error::ProcessingError;
use sqlx::SqlitePool;

/// Export library as a CSV string.
pub async fn export_csv(pool: &SqlitePool) -> Result<String, ProcessingError> {
    let rows = sqlx::query_as::<_, (
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<f64>,
        Option<String>,
        Option<String>,
    )>(
        "SELECT id, title, authors_json, format, publisher,
                series_name, series_index, pubdate, description
         FROM local_books ORDER BY COALESCE(title_sort, title), title",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    let mut out = String::from("id,title,authors,format,publisher,series,series_index,pubdate,description\n");
    for (id, title, authors_json, format, publisher, series_name, series_index, pubdate, description) in rows {
        let authors: Vec<String> = serde_json::from_str(&authors_json).unwrap_or_default();
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&id),
            csv_escape(&title),
            csv_escape(&authors.join("; ")),
            csv_escape(&format),
            csv_escape(publisher.as_deref().unwrap_or("")),
            csv_escape(series_name.as_deref().unwrap_or("")),
            series_index.map(|f| f.to_string()).unwrap_or_default(),
            csv_escape(pubdate.as_deref().unwrap_or("")),
            csv_escape(description.as_deref().unwrap_or("")),
        ));
    }
    Ok(out)
}

/// Export library as an HTML page.
pub async fn export_html(pool: &SqlitePool) -> Result<String, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>)>(
        "SELECT id, title, authors_json, format, series_name
         FROM local_books ORDER BY COALESCE(title_sort, title), title",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    let mut rows_html = String::new();
    for (id, title, authors_json, format, series_name) in &rows {
        let authors: Vec<String> = serde_json::from_str(authors_json).unwrap_or_default();
        rows_html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            html_escape(title),
            html_escape(&authors.join(", ")),
            html_escape(format),
            html_escape(series_name.as_deref().unwrap_or("")),
        ));
        let _ = id;
    }

    Ok(format!(
        r#"<!doctype html><html lang="en"><head><meta charset="UTF-8">
<title>xCalibre Library</title>
<style>body{{font-family:sans-serif;padding:2rem}}table{{border-collapse:collapse;width:100%}}
th,td{{border:1px solid #ccc;padding:.4rem .8rem;text-align:left}}th{{background:#f0f0f0}}</style>
</head><body>
<h1>xCalibre Library ({} books)</h1>
<table><thead><tr><th>Title</th><th>Authors</th><th>Format</th><th>Series</th></tr></thead>
<tbody>{}</tbody></table></body></html>"#,
        rows.len(),
        rows_html
    ))
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
