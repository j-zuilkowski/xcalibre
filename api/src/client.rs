use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("auth error: {0}")]
    Auth(String),
    #[error("API error {status}: {body}")]
    Api { status: u16, body: String },
}

const SERVICE: &str = "xcalibre";
const ACCOUNT: &str = "xs_token";

pub struct ApiClient {
    base_url: String,
    token:    String,
    http:     reqwest::Client,
}

impl ApiClient {
    pub fn from_keyring(base_url: &str) -> Result<Self, ApiError> {
        let entry = keyring::Entry::new(SERVICE, ACCOUNT)
            .map_err(|e| ApiError::Auth(e.to_string()))?;
        let token = entry.get_password()
            .map_err(|e| ApiError::Auth(e.to_string()))?;
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(ApiError::Http)?;
        Ok(Self { base_url: base_url.to_string(), token, http })
    }

    pub fn new(base_url: &str, token: &str) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("failed to build http client");
        Self { base_url: base_url.to_string(), token: token.to_string(), http }
    }

    pub fn http(&self) -> &reqwest::Client { &self.http }
    pub fn base_url(&self) -> &str         { &self.base_url }
    pub fn token(&self) -> &str             { &self.token }
}
