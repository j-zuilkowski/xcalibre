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
    let resp = match read_client()
        .get(format!("{}/books/{}", client.base_url(), book_id))
        .bearer_auth(client.token())
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(_) => return Ok(None),
    };

    if resp.status().as_u16() == 404 {
        return Ok(None);
    }
    if !resp.status().is_success() {
        return Ok(None);
    }
    Ok(resp.json::<RemoteBook>().await.ok())
}

pub async fn list_books(client: &ApiClient, page: u32) -> Result<Vec<RemoteBook>, ApiError> {
    let resp = match read_client()
        .get(format!("{}/books?page={}", client.base_url(), page))
        .bearer_auth(client.token())
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(_) => return Ok(vec![]),
    };

    if !resp.status().is_success() {
        return Ok(vec![]);
    }
    Ok(resp.json::<Vec<RemoteBook>>().await.unwrap_or_default())
}
