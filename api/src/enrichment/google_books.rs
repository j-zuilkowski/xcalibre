use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GBBook {
    pub title:       Option<String>,
    pub authors:     Vec<String>,
    pub publisher:   Option<String>,
    pub published:   Option<String>,
    pub description: Option<String>,
    pub thumbnail:   Option<String>,
}

#[derive(Debug, Deserialize)]
struct GBResponse {
    #[serde(default)]
    items: Vec<GBItem>,
}

#[derive(Debug, Deserialize)]
struct GBItem {
    #[serde(rename = "volumeInfo")]
    volume_info: GBVolumeInfo,
}

#[derive(Debug, Deserialize)]
struct GBVolumeInfo {
    title:                Option<String>,
    #[serde(default)]
    authors:              Vec<String>,
    publisher:            Option<String>,
    #[serde(rename = "publishedDate")]
    published_date:       Option<String>,
    description:          Option<String>,
    #[serde(rename = "imageLinks")]
    image_links:          Option<GBImageLinks>,
}

#[derive(Debug, Deserialize)]
struct GBImageLinks {
    thumbnail: Option<String>,
}

pub async fn lookup_by_isbn(isbn: &str) -> GBBook {
    match fetch(isbn).await {
        Ok(book) => book,
        Err(e) => {
            tracing::warn!("Google Books lookup failed for ISBN {}: {}", isbn, e);
            GBBook::default()
        }
    }
}

async fn fetch(isbn: &str) -> Result<GBBook, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let url = format!(
        "https://www.googleapis.com/books/v1/volumes?q=isbn:{}",
        isbn
    );

    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Ok(GBBook::default());
    }

    let gb: GBResponse = resp.json().await?;
    let item = match gb.items.into_iter().next() {
        Some(i) => i,
        None    => return Ok(GBBook::default()),
    };
    let vi = item.volume_info;

    Ok(GBBook {
        title:       vi.title,
        authors:     vi.authors,
        publisher:   vi.publisher,
        published:   vi.published_date,
        description: vi.description,
        thumbnail:   vi.image_links.and_then(|il| il.thumbnail),
    })
}