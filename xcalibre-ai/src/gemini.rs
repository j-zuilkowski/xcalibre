use crate::{AiError, AiProvider, ChatMessage, ChatResponse, MessageRole};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct GeminiBackend {
    base_url:    String,
    api_key:     String,
    model:       String,
    embed_model: String,
    client:      Client,
}

impl GeminiBackend {
    pub fn new(base_url: &str, api_key: &str, model: &str, embed_model: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            embed_model: embed_model.to_string(),
            client: Client::new(),
        }
    }
    pub fn model(&self) -> &str { &self.model }
}

#[derive(Serialize)]
struct GemPart<'a> { text: &'a str }
#[derive(Serialize)]
struct GemContent<'a> { role: &'a str, parts: Vec<GemPart<'a>> }
#[derive(Serialize)]
struct GemReq<'a> { contents: Vec<GemContent<'a>> }
#[derive(Deserialize)]
struct GemResp { candidates: Vec<GemCandidate> }
#[derive(Deserialize)]
struct GemCandidate { content: GemRespContent }
#[derive(Deserialize)]
struct GemRespContent { parts: Vec<GemRespPart> }
#[derive(Deserialize)]
struct GemRespPart { text: String }
#[derive(Deserialize)]
struct GemModelsResp { models: Vec<GemModel> }
#[derive(Deserialize)]
struct GemModel { name: String }

fn gem_role(role: &MessageRole) -> &'static str {
    match role {
        MessageRole::User => "user",
        MessageRole::Assistant => "model",
        MessageRole::System => "user",
    }
}

#[async_trait]
impl AiProvider for GeminiBackend {
    async fn chat(
        &self, messages: &[ChatMessage],
        _stream_tx: Option<tokio::sync::mpsc::Sender<String>>,
    ) -> Result<ChatResponse, AiError> {
        let contents: Vec<GemContent<'_>> = messages.iter().map(|m| GemContent {
            role: gem_role(&m.role),
            parts: vec![GemPart { text: &m.content }],
        }).collect();
        let url = format!("{}/models/{}:generateContent?key={}", self.base_url, self.model, self.api_key);
        let resp = self.client.post(&url).json(&GemReq { contents }).send().await?;
        if !resp.status().is_success() {
            return Err(AiError::Provider(format!("HTTP {}", resp.status())));
        }
        let data: GemResp = resp.json().await?;
        let text = data.candidates.into_iter().next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text).unwrap_or_default();
        Ok(ChatResponse { content: text, model: self.model.clone(), done: true, reasoning: None })
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, AiError> {
        #[derive(Serialize)]
        struct Req<'a> { content: GemContent<'a> }
        #[derive(Deserialize)]
        struct Resp { embedding: EmbedVals }
        #[derive(Deserialize)]
        struct EmbedVals { values: Vec<f32> }
        let url = format!("{}/models/{}:embedContent?key={}", self.base_url, self.embed_model, self.api_key);
        let body = Req { content: GemContent { role: "user", parts: vec![GemPart { text }] } };
        let resp = self.client.post(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            return Err(AiError::Provider(format!("HTTP {}", resp.status())));
        }
        let data: Resp = resp.json().await?;
        Ok(data.embedding.values)
    }

    async fn list_models(&self) -> Result<Vec<String>, AiError> {
        let url = format!("{}/models?key={}", self.base_url, self.api_key);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(AiError::Provider(format!("HTTP {}", resp.status())));
        }
        let data: GemModelsResp = resp.json().await?;
        Ok(data.models.into_iter()
            .map(|m| m.name.trim_start_matches("models/").to_string())
            .collect())
    }

    fn name(&self) -> &str { "gemini" }
}
