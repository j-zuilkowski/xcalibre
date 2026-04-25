use crate::client::{ApiClient, ApiError};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug)]
pub struct PushPayload {
    pub file_path: PathBuf,
    pub file_name: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct PushResponse {
    pub id: String,
}

pub async fn push_book(
    client: &ApiClient,
    payload: &PushPayload,
) -> Result<PushResponse, ApiError> {
    let bytes = tokio::fs::read(&payload.file_path).await?;
    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(bytes).file_name(payload.file_name.clone()),
        )
        .part(
            "metadata",
            reqwest::multipart::Part::text(payload.metadata.to_string()),
        );

    let resp = client
        .http()
        .post(format!("{}/api/v1/books", client.base_url()))
        .bearer_auth(client.token())
        .multipart(form)
        .send()
        .await?;

    let status = resp.status().as_u16();
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(ApiError::Api { status, body });
    }
    Ok(resp.json::<PushResponse>().await?)
}
