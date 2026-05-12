# RMP-14b — AI Additional Providers (Green: Implementation)

> Prerequisite: rmp14a complete, rmp08b complete.
> TDD role: GREEN — implement OpenAI-compatible and Gemini provider backends + settings UI.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R14b-T01 | `openai.rs` — OpenAI provider (OpenAI-compatible API) | ⬜ |
| R14b-T02 | `gemini.rs` — Google Gemini provider | ⬜ |
| R14b-T03 | `lmstudio.rs` + `openrouter.rs` — thin wrappers | ⬜ |
| R14b-T04 | Provider factory + `ai_list_models` Tauri command | ⬜ |
| R14b-T05 | `AIProviderSettingsPanel.tsx` component | ⬜ |
| R14b-T06 | Milestone check + visual inspection | ⬜ |

---

## R14b-T01

Add to `xcalibre-ai/src/lib.rs`:
```rust
pub mod openai;
pub mod gemini;
pub mod lmstudio;
pub mod openrouter;
```

Write `xcalibre-ai/src/openai.rs`:
```rust
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
struct OaiChatReq<'a> {
    model:    &'a str,
    messages: Vec<OaiMsg<'a>>,
    stream:   bool,
}
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
        &self,
        messages: &[ChatMessage],
        _stream_tx: Option<tokio::sync::mpsc::Sender<String>>,
    ) -> Result<ChatResponse, AiError> {
        let msgs: Vec<OaiMsg<'_>> = messages.iter()
            .map(|m| OaiMsg { role: role_str(&m.role), content: &m.content })
            .collect();
        let body = OaiChatReq { model: &self.model, messages: msgs, stream: false };
        let resp = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send().await?;
        if !resp.status().is_success() {
            return Err(AiError::Provider(format!("HTTP {}", resp.status())));
        }
        let data: OaiChatResp = resp.json().await?;
        let content = data.choices.into_iter().next()
            .map(|c| c.message.content)
            .unwrap_or_default();
        Ok(ChatResponse { content, model: self.model.clone(), done: true, reasoning: None })
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, AiError> {
        let body = OaiEmbedReq { model: &self.embed_model, input: text };
        let resp = self.client
            .post(format!("{}/embeddings", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send().await?;
        if !resp.status().is_success() {
            return Err(AiError::Provider(format!("HTTP {}", resp.status())));
        }
        let data: OaiEmbedResp = resp.json().await?;
        Ok(data.data.into_iter().next().map(|d| d.embedding).unwrap_or_default())
    }

    async fn list_models(&self) -> Result<Vec<String>, AiError> {
        let resp = self.client
            .get(format!("{}/models", self.base_url))
            .bearer_auth(&self.api_key)
            .send().await?;
        if !resp.status().is_success() {
            return Err(AiError::Provider(format!("HTTP {}", resp.status())));
        }
        let data: OaiModelsResp = resp.json().await?;
        Ok(data.data.into_iter().map(|m| m.id).collect())
    }

    fn name(&self) -> &str { "openai" }
}
```

Then run:
```bash
cargo test -p xcalibre-ai -- test_openai
git add xcalibre-ai/src/openai.rs xcalibre-ai/src/lib.rs
git commit -m "R14b-T01: OpenAI provider implementation — tests green"
```

---

## R14b-T02

Write `xcalibre-ai/src/gemini.rs`:
```rust
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
            base_url:    base_url.trim_end_matches('/').to_string(),
            api_key:     api_key.to_string(),
            model:       model.to_string(),
            embed_model: embed_model.to_string(),
            client:      Client::new(),
        }
    }
    pub fn model(&self) -> &str { &self.model }
}

#[derive(Serialize)]
struct GemPart<'a>    { text: &'a str }
#[derive(Serialize)]
struct GemContent<'a> { role: &'a str, parts: Vec<GemPart<'a>> }
#[derive(Serialize)]
struct GemReq<'a>     { contents: Vec<GemContent<'a>> }
#[derive(Deserialize)]
struct GemResp        { candidates: Vec<GemCandidate> }
#[derive(Deserialize)]
struct GemCandidate   { content: GemRespContent }
#[derive(Deserialize)]
struct GemRespContent { parts: Vec<GemRespPart> }
#[derive(Deserialize)]
struct GemRespPart    { text: String }
#[derive(Deserialize)]
struct GemModelsResp  { models: Vec<GemModel> }
#[derive(Deserialize)]
struct GemModel       { name: String }

fn gem_role(role: &MessageRole) -> &'static str {
    match role {
        MessageRole::User      => "user",
        MessageRole::Assistant => "model",
        MessageRole::System    => "user",
    }
}

#[async_trait]
impl AiProvider for GeminiBackend {
    async fn chat(
        &self,
        messages: &[ChatMessage],
        _stream_tx: Option<tokio::sync::mpsc::Sender<String>>,
    ) -> Result<ChatResponse, AiError> {
        let contents: Vec<GemContent<'_>> = messages.iter()
            .map(|m| GemContent {
                role: gem_role(&m.role),
                parts: vec![GemPart { text: &m.content }],
            })
            .collect();
        let url = format!("{}/models/{}:generateContent?key={}",
                          self.base_url, self.model, self.api_key);
        let resp = self.client.post(&url).json(&GemReq { contents }).send().await?;
        if !resp.status().is_success() {
            return Err(AiError::Provider(format!("HTTP {}", resp.status())));
        }
        let data: GemResp = resp.json().await?;
        let text = data.candidates.into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .unwrap_or_default();
        Ok(ChatResponse { content: text, model: self.model.clone(), done: true, reasoning: None })
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, AiError> {
        #[derive(Serialize)]
        struct Req<'a> { content: GemContent<'a> }
        #[derive(Deserialize)]
        struct Resp { embedding: EmbedVals }
        #[derive(Deserialize)]
        struct EmbedVals { values: Vec<f32> }

        let url = format!("{}/models/{}:embedContent?key={}",
                          self.base_url, self.embed_model, self.api_key);
        let body = Req {
            content: GemContent {
                role: "user",
                parts: vec![GemPart { text }],
            }
        };
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
```

Then run:
```bash
cargo test -p xcalibre-ai -- test_gemini
git add xcalibre-ai/src/gemini.rs
git commit -m "R14b-T02: Google Gemini provider implementation — tests green"
```

---

## R14b-T03

Write `xcalibre-ai/src/lmstudio.rs`:
```rust
// LM Studio exposes an OpenAI-compatible REST API on localhost:1234.
// This is a thin wrapper around OpenAiBackend with provider name "lmstudio".
use crate::openai::OpenAiBackend;
use crate::{AiError, AiProvider, ChatMessage, ChatResponse};
use async_trait::async_trait;

pub struct LmStudioBackend(OpenAiBackend);

impl LmStudioBackend {
    pub fn new(base_url: &str, model: &str, embed_model: &str) -> Self {
        // No API key for LM Studio
        Self(OpenAiBackend::new(base_url, "", model, embed_model))
    }
    pub fn model(&self) -> &str { self.0.model() }
}

#[async_trait]
impl AiProvider for LmStudioBackend {
    async fn chat(&self, msgs: &[ChatMessage], tx: Option<tokio::sync::mpsc::Sender<String>>) -> Result<ChatResponse, AiError> {
        self.0.chat(msgs, tx).await
    }
    async fn embed(&self, text: &str) -> Result<Vec<f32>, AiError> { self.0.embed(text).await }
    async fn list_models(&self) -> Result<Vec<String>, AiError>    { self.0.list_models().await }
    fn name(&self) -> &str { "lmstudio" }
}
```

Write `xcalibre-ai/src/openrouter.rs`:
```rust
// OpenRouter exposes an OpenAI-compatible API at openrouter.ai/api/v1.
use crate::openai::OpenAiBackend;
use crate::{AiError, AiProvider, ChatMessage, ChatResponse};
use async_trait::async_trait;

pub struct OpenRouterBackend(OpenAiBackend);

impl OpenRouterBackend {
    pub fn new(base_url: &str, api_key: &str, model: &str, embed_model: &str) -> Self {
        Self(OpenAiBackend::new(base_url, api_key, model, embed_model))
    }
    pub fn model(&self) -> &str { self.0.model() }
}

#[async_trait]
impl AiProvider for OpenRouterBackend {
    async fn chat(&self, msgs: &[ChatMessage], tx: Option<tokio::sync::mpsc::Sender<String>>) -> Result<ChatResponse, AiError> {
        self.0.chat(msgs, tx).await
    }
    async fn embed(&self, text: &str) -> Result<Vec<f32>, AiError> { self.0.embed(text).await }
    async fn list_models(&self) -> Result<Vec<String>, AiError>    { self.0.list_models().await }
    fn name(&self) -> &str { "openrouter" }
}
```

Then run:
```bash
cargo test -p xcalibre-ai -- test_lmstudio test_openrouter
git add xcalibre-ai/src/lmstudio.rs xcalibre-ai/src/openrouter.rs
git commit -m "R14b-T03: LM Studio + OpenRouter thin wrapper providers — tests green"
```

---

## R14b-T04

Write `xcalibre-ai/src/factory.rs`:
```rust
use crate::{AiProvider, AiError};
use crate::ollama::OllamaBackend;
use crate::openai::OpenAiBackend;
use crate::gemini::GeminiBackend;
use crate::lmstudio::LmStudioBackend;
use crate::openrouter::OpenRouterBackend;

pub struct ProviderConfig {
    pub provider:    String,
    pub model:       String,
    pub embed_model: String,
    pub base_url:    String,
    pub api_key:     Option<String>,
}

pub fn make_provider(cfg: &ProviderConfig) -> Result<Box<dyn AiProvider>, AiError> {
    let key = cfg.api_key.as_deref().unwrap_or("");
    match cfg.provider.as_str() {
        "ollama"     => Ok(Box::new(OllamaBackend::new(&cfg.base_url, &cfg.model, &cfg.embed_model))),
        "openai"     => Ok(Box::new(OpenAiBackend::new(&cfg.base_url, key, &cfg.model, &cfg.embed_model))),
        "gemini"     => Ok(Box::new(GeminiBackend::new(&cfg.base_url, key, &cfg.model, &cfg.embed_model))),
        "lmstudio"   => Ok(Box::new(LmStudioBackend::new(&cfg.base_url, &cfg.model, &cfg.embed_model))),
        "openrouter" => Ok(Box::new(OpenRouterBackend::new(&cfg.base_url, key, &cfg.model, &cfg.embed_model))),
        other        => Err(AiError::Provider(format!("unknown provider: {other}"))),
    }
}
```

Add to `xcalibre-ai/src/lib.rs`:
```rust
pub mod factory;
pub use factory::{make_provider, ProviderConfig};
```

In `src-tauri/src/commands/ai.rs`, add:
```rust
use xcalibre_ai::factory::{make_provider, ProviderConfig};

#[tauri::command]
pub async fn ai_list_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let cfg = xcalibre_processing::db::ai_queries::get_ai_config(&state.pool, None)
        .await.map_err(|e| e.to_string())?;
    let cfg = match cfg {
        Some(c) => c,
        None    => return Ok(vec![]),
    };
    let pcfg = ProviderConfig {
        provider:    cfg.provider,
        model:       cfg.model,
        embed_model: cfg.embed_model,
        base_url:    cfg.base_url,
        api_key:     cfg.api_key,
    };
    let provider = make_provider(&pcfg).map_err(|e| e.to_string())?;
    provider.list_models().await.map_err(|e| e.to_string())
}
```

Register `ai_list_models` in `src-tauri/src/main.rs`.

```bash
cargo build --workspace
git add xcalibre-ai/src/factory.rs xcalibre-ai/src/lib.rs \
        src-tauri/src/commands/ai.rs src-tauri/src/main.rs
git commit -m "R14b-T04: provider factory + ai_list_models Tauri command"
```

---

## R14b-T05

Write `ui/src/components/AIProviderSettingsPanel.tsx`:
```tsx
import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"

const PROVIDERS = [
  { value: "ollama",     label: "Ollama",         needsKey: false, defaultUrl: "http://localhost:11434" },
  { value: "openai",     label: "OpenAI",          needsKey: true,  defaultUrl: "https://api.openai.com/v1" },
  { value: "gemini",     label: "Google Gemini",   needsKey: true,  defaultUrl: "https://generativelanguage.googleapis.com/v1beta" },
  { value: "lmstudio",   label: "LM Studio",       needsKey: false, defaultUrl: "http://localhost:1234" },
  { value: "openrouter", label: "OpenRouter",      needsKey: true,  defaultUrl: "https://openrouter.ai/api/v1" },
]

interface Props { onClose: () => void }

export function AIProviderSettingsPanel({ onClose }: Props) {
  const [provider,   setProvider]   = useState("ollama")
  const [baseUrl,    setBaseUrl]    = useState("http://localhost:11434")
  const [apiKey,     setApiKey]     = useState("")
  const [model,      setModel]      = useState("")
  const [embedModel, setEmbedModel] = useState("")
  const [models,     setModels]     = useState<string[]>([])
  const [testResult, setTestResult] = useState<string | null>(null)
  const [saving,     setSaving]     = useState(false)
  const [loading,    setLoading]    = useState(true)

  useEffect(() => {
    invoke<any>("get_ai_config_cmd").then(cfg => {
      if (cfg) {
        setProvider(cfg.provider); setBaseUrl(cfg.base_url)
        setApiKey(cfg.api_key ?? ""); setModel(cfg.model)
        setEmbedModel(cfg.embed_model)
      }
    }).catch(console.error).finally(() => setLoading(false))
  }, [])

  async function handleTestConnection() {
    setTestResult(null)
    try {
      const list = await invoke<string[]>("ai_list_models")
      setModels(list)
      setTestResult(`Connected. ${list.length} model(s) available.`)
    } catch (e) {
      setTestResult(`Connection failed: ${e}`)
    }
  }

  async function handleSave() {
    setSaving(true)
    try {
      await invoke("save_ai_config_cmd", {
        provider, baseUrl, apiKey: apiKey || null, model,
        embedModel, reasoningStrategy: "auto",
        includeFields: '["title","authors","tags","series","description"]',
      })
      onClose()
    } finally { setSaving(false) }
  }

  function handleProviderChange(p: string) {
    setProvider(p)
    const meta = PROVIDERS.find(x => x.value === p)
    if (meta) setBaseUrl(meta.defaultUrl)
    setModels([])
    setTestResult(null)
  }

  const providerMeta = PROVIDERS.find(p => p.value === provider)

  return (
    <div role="dialog" aria-modal="true" style={{
      position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)",
      display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000,
    }}>
      <div style={{
        background: "var(--bg-surface, #1e1e2e)", borderRadius: "12px",
        padding: "2rem", minWidth: "420px", color: "var(--text-primary, #cdd6f4)",
      }}>
        <h2 style={{ margin: "0 0 1.5rem" }}>AI Provider Settings</h2>
        {loading ? (
          <p>Loading…</p>
        ) : (
          <>
            <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.9rem" }}>Provider</label>
            <select
              data-testid="ai-provider-select"
              value={provider}
              onChange={e => handleProviderChange(e.target.value)}
              style={{ width: "100%", padding: "0.5rem", background: "var(--bg-overlay, #313244)",
                       border: "1px solid var(--border, #45475a)", borderRadius: "6px",
                       color: "inherit", marginBottom: "1rem" }}
            >
              {PROVIDERS.map(p => <option key={p.value} value={p.value}>{p.label}</option>)}
            </select>

            <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.9rem" }}>Base URL</label>
            <input
              data-testid="ai-base-url-input"
              value={baseUrl}
              onChange={e => setBaseUrl(e.target.value)}
              style={{ width: "100%", padding: "0.5rem", background: "var(--bg-overlay, #313244)",
                       border: "1px solid var(--border, #45475a)", borderRadius: "6px",
                       color: "inherit", marginBottom: "1rem", boxSizing: "border-box" }}
            />

            {providerMeta?.needsKey && (
              <>
                <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.9rem" }}>API Key</label>
                <input
                  data-testid="ai-api-key-input"
                  type="password"
                  value={apiKey}
                  onChange={e => setApiKey(e.target.value)}
                  style={{ width: "100%", padding: "0.5rem", background: "var(--bg-overlay, #313244)",
                           border: "1px solid var(--border, #45475a)", borderRadius: "6px",
                           color: "inherit", marginBottom: "1rem", boxSizing: "border-box" }}
                />
              </>
            )}

            <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.9rem" }}>Model</label>
            <input
              data-testid="ai-model-input"
              value={model}
              onChange={e => setModel(e.target.value)}
              list="ai-models-list"
              style={{ width: "100%", padding: "0.5rem", background: "var(--bg-overlay, #313244)",
                       border: "1px solid var(--border, #45475a)", borderRadius: "6px",
                       color: "inherit", marginBottom: "1rem", boxSizing: "border-box" }}
            />
            <datalist id="ai-models-list">
              {models.map(m => <option key={m} value={m} />)}
            </datalist>

            {testResult && (
              <p style={{
                fontSize: "0.85rem", padding: "0.5rem",
                background: testResult.includes("Connected")
                  ? "var(--green-dim, #1e3a2a)" : "var(--red-dim, #3a1e1e)",
                borderRadius: "6px", marginBottom: "1rem",
              }}>
                {testResult}
              </p>
            )}

            <div style={{ display: "flex", gap: "0.75rem", justifyContent: "flex-end" }}>
              <button
                data-testid="ai-test-connection-btn"
                onClick={handleTestConnection}
                style={{ padding: "0.5rem 1rem", background: "var(--bg-overlay, #313244)",
                         border: "1px solid var(--border, #45475a)", borderRadius: "6px",
                         cursor: "pointer", color: "inherit" }}
              >
                Test Connection
              </button>
              <button
                onClick={onClose}
                style={{ padding: "0.5rem 1rem", background: "var(--bg-overlay, #313244)",
                         border: "none", borderRadius: "6px", cursor: "pointer", color: "inherit" }}
              >
                Cancel
              </button>
              <button
                data-testid="ai-settings-save-btn"
                onClick={handleSave}
                disabled={saving}
                style={{ padding: "0.5rem 1rem", background: "var(--blue, #89b4fa)",
                         border: "none", borderRadius: "6px", cursor: "pointer",
                         color: "#1e1e2e", fontWeight: 600, opacity: saving ? 0.6 : 1 }}
              >
                {saving ? "Saving…" : "Save"}
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  )
}
```

Then run:
```bash
cd ui && npm test -- AIProviderSettingsPanel && cd ..
git add ui/src/components/AIProviderSettingsPanel.tsx
git commit -m "R14b-T05: AIProviderSettingsPanel — all UI tests green"
```

---

## R14b-T06 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm test && cd ..
```

**Visual inspection:**
```bash
pkill -x xcalibre 2>/dev/null || true
cargo tauri dev &>/tmp/xcalibre_tauri_dev.log &
# Poll until xcalibre process appears — first-run compilation can take 3-5 min
for i in $(seq 1 30); do
  sleep 10
  if pgrep -x xcalibre > /dev/null 2>&1; then
    echo "xcalibre running after $((i*10))s"
    sleep 3
    break
  fi
  echo "Waiting for xcalibre… $((i*10))s elapsed"
  [ "$i" -eq 30 ] && echo "ERROR: xcalibre did not launch within 5 minutes" && exit 1
done
```

1. Verify the UI:
2. Open Settings → AI Provider
3. Verify all 5 providers appear in the dropdown
4. Select Ollama → click "Test Connection" → verify model list appears if Ollama is running
5. Select OpenAI → verify API Key field appears
6. Enter OpenAI key + model → Save → open AI chat → verify response uses OpenAI
7. Select LM Studio → Test Connection → verify success if LM Studio is running
8. Verify that saved provider persists after app restart

```bash
pkill -x xcalibre 2>/dev/null || true
```

```bash
git add -A
git commit -m "R14b-T06: RMP-14 AI Additional Providers — all tests green, factory wired"
```
