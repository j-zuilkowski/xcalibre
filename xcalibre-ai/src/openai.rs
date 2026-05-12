use crate::{AiError, AiProvider, ChatMessage, ChatResponse, MessageRole};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAiBackend {
    base_url:    String,
    api_key:     String,
    model:       String,
    embed_model: String,
    client:      Client,
}

impl OpenAiBackend {
    pub fn new(base_url: &str, api_key: &str, model: &str, embed_model: &str) -> Self {
        Self {
            base_url:    base_url.trim_end_matches('/').to_string(),
            api_key:     api_key.to_string(),
            model:       model.to_string(),
            embed_model: embed_model.to_string(),
            client:      Client::new(),
        }
    }
    pub fn model(&self)       -> &str { &self.model }
    pub fn embed_model(&self) -> &str { &self.embed_model }
}

#[derive(Serialize)]
struct OaiChatReq<'a> { model: &'a str, messages: Vec<OaiMsg<'a>>, stream: bool }
#[derive(Serialize)]
struct OaiMsg<'a> { role: &'a str, content: &'a str }
#[derive(Deserialize)]
struct OaiChatResp { choices: Vec<OaiChoice> }
#[derive(Deserialize)]
struct OaiChoice { message: OaiRespMsg }
#[derive(Deserialize)]
struct OaiRespMsg { content: String }
#[derive(Serialize)]
struct OaiEmbedReq<'a> { model: &'a str, input: &'a str }
#[derive(Deserialize)]
struct OaiEmbedResp { data: Vec<OaiEmbedData> }
#[derive(Deserialize)]
struct OaiEmbedData { embedding: Vec<f32> }
#[derive(Deserialize)]
struct OaiModelsResp { data: Vec<OaiModelData> }
#[derive(Deserialize)]
struct OaiModelData { id: String }

fn role_str(role: &MessageRole) -> &'static str {
    match role {
        MessageRole::System    => "system",
        MessageRole::User      => "user",
        MessageRole::Assistant => "assistant",
    }
}

#[async_trait]
impl AiProvider for OpenAiBackend {
    async fn chat(
        &self, messages: &[ChatMessage],
        _stream_tx: Option<tokio::sync::mpsc::Sender<String>>,
    ) -> Result<ChatResponse, AiError> {
        let msgs: Vec<OaiMsg<'_>> = messages.iter()
            .map(|m| OaiMsg { role: role_str(&m.role), content: &m.content }).collect();
        let body = OaiChatReq { model: &self.model, messages: msgs, stream: false };
        let resp = self.client.post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key).json(&body).send().await?;
        if !resp.status().is_success() {
            return Err(AiError::Provider(format!("HTTP {}", resp.status())));
        }
        let data: OaiChatResp = resp.json().await?;
        let content = data.choices.into_iter().next()
            .map(|c| c.message.content).unwrap_or_default();
        Ok(ChatResponse { content, model: self.model.clone(), done: true, reasoning: None })
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, AiError> {
        let body = OaiEmbedReq { model: &self.embed_model, input: text };
        let resp = self.client.post(format!("{}/embeddings", self.base_url))
            .bearer_auth(&self.api_key).json(&body).send().await?;
        if !resp.status().is_success() {
            return Err(AiError::Provider(format!("HTTP {}", resp.status())));
        }
        let data: OaiEmbedResp = resp.json().await?;
        Ok(data.data.into_iter().next().map(|d| d.embedding).unwrap_or_default())
    }

    async fn list_models(&self) -> Result<Vec<String>, AiError> {
        let resp = self.client.get(format!("{}/models", self.base_url))
            .bearer_auth(&self.api_key).send().await?;
        if !resp.status().is_success() {
            return Err(AiError::Provider(format!("HTTP {}", resp.status())));
        }
        let data: OaiModelsResp = resp.json().await?;
        Ok(data.data.into_iter().map(|m| m.id).collect())
    }

    fn name(&self) -> &str { "openai" }
}
