use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OLBook {
    pub title:        Option<String>,
    pub authors:      Vec<String>,
    pub publishers:   Vec<String>,
    pub publish_date: Option<String>,
    pub description:  Option<String>,
    pub subjects:     Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OLEntry {
    title:        Option<String>,
    #[serde(default)]
    authors:      Vec<OLAuthor>,
    #[serde(default)]
    publishers:   Vec<OLPublisher>,
    publish_date: Option<String>,
    notes:        Option<serde_json::Value>,
    #[serde(default)]
    subjects:     Vec<OLSubject>,
}

#[derive(Debug, Deserialize)]
struct OLAuthor    { name: String }
#[derive(Debug, Deserialize)]
struct OLPublisher { name: String }
#[derive(Debug, Deserialize)]
struct OLSubject   { name: String }

pub async fn lookup_by_isbn(isbn: &str) -> OLBook {
    match fetch(isbn).await {
        Ok(book) => book,
        Err(e) => {
            tracing::warn!("Open Library lookup failed for ISBN {}: {}", isbn, e);
            OLBook::default()
        }
    }
}

async fn fetch(isbn: &str) -> Result<OLBook, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let url = format!(
        "https://openlibrary.org/api/books?bibkeys=ISBN:{}&format=json&jscmd=data",
        isbn
    );

    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Ok(OLBook::default());
    }

    let map: std::collections::HashMap<String, OLEntry> = resp.json().await?;
    let key = format!("ISBN:{}", isbn);
    let entry = match map.get(&key) {
        Some(e) => e,
        None    => return Ok(OLBook::default()),
    };

    let description = entry.notes.as_ref().and_then(|n| match n {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Object(o) => o.get("value").and_then(|v| v.as_str()).map(String::from),
        _ => None,
    });

    Ok(OLBook {
        title:        entry.title.clone(),
        authors:      entry.authors.iter().map(|a| a.name.clone()).collect(),
        publishers:   entry.publishers.iter().map(|p| p.name.clone()).collect(),
        publish_date: entry.publish_date.clone(),
        description,
        subjects:     entry.subjects.iter().map(|s| s.name.clone()).collect(),
    })
}
