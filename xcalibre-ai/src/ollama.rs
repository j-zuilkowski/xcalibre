use crate::{AiError, ChatMessage, ChatResponse, MessageRole};
use crate::provider::AiProvider;
use serde::{Deserialize, Serialize};

pub struct OllamaBackend {
    pub base_url:    String,
    pub model:       String,
    pub embed_model: String,
    client:          reqwest::Client,
}

impl OllamaBackend {
    pub fn new(base_url: &str, model: &str, embed_model: &str) -> Self {
        Self {
            base_url:    base_url.trim_end_matches('/').to_string(),
            model:       model.to_string(),
            embed_model: embed_model.to_string(),
            client:      reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build().unwrap(),
        }
    }

    pub async fn health_check(&self) -> bool {
        self.client.get(format!("{}/api/tags", self.base_url))
            .send().await.map(|r| r.status().is_success()).unwrap_or(false)
    }
}

#[derive(Serialize)]
struct OllamaChatRequest {
    model:    String,
    messages: Vec<OllamaMsg>,
    stream:   bool,
}

#[derive(Serialize, Deserialize)]
struct OllamaMsg {
    role:    String,
    content: String,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: OllamaMsg,
    model:   String,
    done:    bool,
}

#[derive(Serialize)]
struct OllamaEmbedRequest { model: String, prompt: String }

#[derive(Deserialize)]
struct OllamaEmbedResponse { embedding: Vec<f32> }

#[async_trait::async_trait]
impl AiProvider for OllamaBackend {
    async fn chat(
        &self,
        messages: &[ChatMessage],
        _stream_tx: Option<tokio::sync::mpsc::Sender<String>>,
    ) -> Result<ChatResponse, AiError> {
        let ollama_msgs: Vec<OllamaMsg> = messages.iter().map(|m| OllamaMsg {
            role: match m.role {
                MessageRole::System    => "system".into(),
                MessageRole::User      => "user".into(),
                MessageRole::Assistant => "assistant".into(),
            },
            content: m.content.clone(),
        }).collect();

        let req = OllamaChatRequest {
            model: self.model.clone(), messages: ollama_msgs, stream: false,
        };
        let resp = self.client
            .post(format!("{}/api/chat", self.base_url))
            .json(&req).send().await?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AiError::Provider(format!("Ollama error: {}", body)));
        }
        let body: OllamaChatResponse = resp.json().await?;
        Ok(ChatResponse {
            content:   body.message.content,
            model:     body.model,
            done:      body.done,
            reasoning: None,
        })
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, AiError> {
        let req = OllamaEmbedRequest {
            model: self.embed_model.clone(), prompt: text.to_string(),
        };
        let resp = self.client
            .post(format!("{}/api/embeddings", self.base_url))
            .json(&req).send().await?;
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AiError::Provider(format!("embed error: {}", body)));
        }
        let body: OllamaEmbedResponse = resp.json().await?;
        Ok(body.embedding)
    }

    async fn list_models(&self) -> Result<Vec<String>, AiError> {
        #[derive(Deserialize)]
        struct TagsResp { models: Vec<ModelInfo> }
        #[derive(Deserialize)]
        struct ModelInfo { name: String }
        let resp = self.client.get(format!("{}/api/tags", self.base_url))
            .send().await?;
        let body: TagsResp = resp.json().await?;
        Ok(body.models.into_iter().map(|m| m.name).collect())
    }

    fn name(&self) -> &str { "ollama" }
}
