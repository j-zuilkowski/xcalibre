# RMP-08b — AI Integration: Ollama MVP + RAG Pipeline (Green: Implementation)

> Prerequisite: rmp08a complete.
> TDD role: GREEN — implement until all tests pass.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R08b-T01 | `chunk.rs` — chunk_text() with overlap | ⬜ |
| R08b-T02 | `ollama/mod.rs` — OllamaBackend (chat + embed) | ⬜ |
| R08b-T03 | `db/ai_queries.rs` — AI config + chunk CRUD | ⬜ |
| R08b-T04 | RAG ingest step in pipeline | ⬜ |
| R08b-T05 | Tauri commands: ai_chat, get_ai_context_chunks, get_ai_config | ⬜ |
| R08b-T06 | AIChatPanel.tsx component | ⬜ |
| R08b-T07 | BookDiscussDialog.tsx component | ⬜ |
| R08b-T08 | Wire Ctrl+Alt+A shortcut | ⬜ |
| R08b-T09 | Milestone check + visual inspection | ⬜ |

---

## R08b-T01

Write `xcalibre-ai/src/chunk.rs`:
```rust
/// Configuration for the chunking algorithm.
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Maximum tokens per chunk (approximate: 1 token ≈ 4 chars).
    pub max_tokens:     usize,
    /// Number of tokens to overlap between consecutive chunks.
    pub overlap_tokens: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self { Self { max_tokens: 512, overlap_tokens: 64 } }
}

/// Split text into overlapping chunks, respecting sentence boundaries where possible.
pub fn chunk_text(text: &str, cfg: &ChunkConfig) -> Vec<String> {
    if text.trim().is_empty() { return vec![]; }

    let max_chars     = cfg.max_tokens * 4;
    let overlap_chars = cfg.overlap_tokens * 4;

    // Split into sentences first
    let sentences = split_sentences(text);
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();

    for sentence in &sentences {
        if current.len() + sentence.len() > max_chars && !current.is_empty() {
            chunks.push(current.trim().to_string());
            // Start next chunk with overlap from the end of the previous chunk
            let tail = last_chars(&current, overlap_chars);
            current = tail;
        }
        current.push_str(sentence);
        current.push(' ');
    }
    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }
    chunks.into_iter().filter(|c| !c.is_empty()).collect()
}

fn split_sentences(text: &str) -> Vec<String> {
    // Simple sentence splitter: split at ". ", "! ", "? "
    let mut sentences = Vec::new();
    let mut buf = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        buf.push(chars[i]);
        if matches!(chars[i], '.' | '!' | '?') {
            if i + 1 < chars.len() && chars[i + 1] == ' ' {
                sentences.push(buf.clone());
                buf.clear();
                i += 2; // skip the space
                continue;
            }
        }
        i += 1;
    }
    if !buf.trim().is_empty() { sentences.push(buf); }
    sentences
}

fn last_chars(s: &str, n: usize) -> String {
    if s.len() <= n { return s.to_string(); }
    let start = s.len() - n;
    // Find a word boundary
    let boundary = s[start..].find(' ').map(|p| start + p + 1).unwrap_or(start);
    s[boundary..].to_string()
}
```

Then run:
```bash
cargo test -p xcalibre-ai -- test_chunking
git add xcalibre-ai/src/chunk.rs
git commit -m "R08b-T01: chunk_text with sentence-aware overlap — chunking tests green"
```

---

## R08b-T02

Write `xcalibre-ai/src/ollama/mod.rs`:
```rust
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
```

```bash
cargo build -p xcalibre-ai
cargo clippy -p xcalibre-ai -- -D warnings
git add xcalibre-ai/src/ollama/mod.rs xcalibre-ai/src/lib.rs
git commit -m "R08b-T02: OllamaBackend — chat and embed implementation"
```

---

## R08b-T03

In `processing/src/db/mod.rs`, add `pub mod ai_queries;`.

Write `processing/src/db/ai_queries.rs`:
```rust
use crate::error::ProcessingError;
use sqlx::SqlitePool;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct AiConfig {
    pub provider:           String,
    pub model:              String,
    pub embed_model:        String,
    pub base_url:           String,
    pub api_key:            Option<String>,
    pub reasoning_strategy: String,
    pub include_fields:     String,
}

pub async fn get_ai_config(pool: &SqlitePool, library_id: Option<&str>)
    -> Result<Option<AiConfig>, ProcessingError>
{
    sqlx::query_as::<_, AiConfig>(
        "SELECT provider, model, embed_model, base_url, api_key,
                reasoning_strategy, include_fields
         FROM ai_config WHERE library_id IS ? LIMIT 1",
    )
    .bind(library_id)
    .fetch_optional(pool).await.map_err(ProcessingError::DbError)
}

pub async fn upsert_ai_config(pool: &SqlitePool, library_id: Option<&str>, cfg: &AiConfig)
    -> Result<(), ProcessingError>
{
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO ai_config
         (library_id, provider, model, embed_model, base_url, api_key,
          reasoning_strategy, include_fields, updated_at)
         VALUES (?,?,?,?,?,?,?,?,?)
         ON CONFLICT(id) DO UPDATE SET
          provider=excluded.provider, model=excluded.model,
          embed_model=excluded.embed_model, base_url=excluded.base_url,
          api_key=excluded.api_key, reasoning_strategy=excluded.reasoning_strategy,
          include_fields=excluded.include_fields, updated_at=excluded.updated_at",
    )
    .bind(library_id).bind(&cfg.provider).bind(&cfg.model).bind(&cfg.embed_model)
    .bind(&cfg.base_url).bind(&cfg.api_key).bind(&cfg.reasoning_strategy)
    .bind(&cfg.include_fields).bind(&now)
    .execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn insert_book_chunk(
    pool: &SqlitePool, book_id: &str, chunk_index: i64,
    chunk_text: &str, embedding: Option<&[u8]>, embed_model: Option<&str>,
) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO book_chunks (book_id, chunk_index, chunk_text, embedding, embed_model, created_at)
         VALUES (?,?,?,?,?,?)
         ON CONFLICT(book_id, chunk_index) DO UPDATE SET
          chunk_text=excluded.chunk_text, embedding=excluded.embedding,
          embed_model=excluded.embed_model",
    )
    .bind(book_id).bind(chunk_index).bind(chunk_text).bind(embedding).bind(embed_model).bind(&now)
    .execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn get_book_chunks(pool: &SqlitePool, book_id: &str)
    -> Result<Vec<(i64, String, Option<Vec<u8>>)>, ProcessingError>
{
    let rows: Vec<(i64, String, Option<Vec<u8>>)> = sqlx::query_as(
        "SELECT chunk_index, chunk_text, embedding FROM book_chunks
         WHERE book_id = ? ORDER BY chunk_index",
    )
    .bind(book_id)
    .fetch_all(pool).await.map_err(ProcessingError::DbError)?;
    Ok(rows)
}

pub async fn delete_book_chunks(pool: &SqlitePool, book_id: &str) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM book_chunks WHERE book_id = ?")
        .bind(book_id).execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}
```

Then run:
```bash
cargo test --workspace -- test_ai_config
git add processing/src/db/ai_queries.rs processing/src/db/mod.rs
git commit -m "R08b-T03: AI config and book_chunks queries — AI config tests green"
```

---

## R08b-T04

Add the RAG chunking step to the ingest pipeline.
In `processing/src/pipeline/mod.rs`, add `pub mod rag;`.

Write `processing/src/pipeline/rag.rs`:
```rust
//! RAG embedding pipeline step: chunk job_text and store chunks.
//! Embedding is performed asynchronously after text extraction.
//! When no embed service is reachable, chunks are stored without embeddings.

use crate::db::ai_queries::{delete_book_chunks, get_ai_config, insert_book_chunk};
use crate::error::ProcessingError;
use sqlx::SqlitePool;
use tracing::{info, warn};
use xcalibre_ai::chunk::{chunk_text, ChunkConfig};

pub async fn run_rag_chunk(
    pool: &SqlitePool,
    book_id: &str,
    full_text: &str,
) -> Result<(), ProcessingError> {
    if full_text.trim().is_empty() { return Ok(()); }

    let cfg = ChunkConfig::default();
    let chunks = chunk_text(full_text, &cfg);
    if chunks.is_empty() { return Ok(()); }

    delete_book_chunks(pool, book_id).await?;

    let ai_cfg = get_ai_config(pool, None).await.ok().flatten();
    let (embed_model, base_url) = ai_cfg
        .as_ref()
        .map(|c| (c.embed_model.as_str(), c.base_url.as_str()))
        .unwrap_or(("nomic-embed-text", "http://localhost:11434"));

    for (i, chunk) in chunks.iter().enumerate() {
        // Attempt embedding; fall back to None if Ollama is not running
        let embedding = try_embed(chunk, embed_model, base_url).await;
        let embed_blob = embedding.map(|v| {
            v.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<u8>>()
        });
        insert_book_chunk(pool, book_id, i as i64, chunk,
            embed_blob.as_deref(), Some(embed_model)).await?;
    }

    info!(book_id, chunks = chunks.len(), "RAG chunks stored");
    Ok(())
}

async fn try_embed(text: &str, model: &str, base_url: &str) -> Option<Vec<f32>> {
    use xcalibre_ai::ollama::OllamaBackend;
    use xcalibre_ai::provider::AiProvider;
    let backend = OllamaBackend::new(base_url, model, model);
    match backend.embed(text).await {
        Ok(v)  => Some(v),
        Err(e) => { warn!("embed failed (non-fatal): {}", e); None }
    }
}
```

Add `xcalibre-ai = { path = "../xcalibre-ai" }` to `processing/Cargo.toml`.

Call `run_rag_chunk` from the text pipeline stage (`pipeline/text.rs`) after
storing `full_text` in `job_text`:
```rust
// After the INSERT INTO job_text:
if let Err(e) = crate::pipeline::rag::run_rag_chunk(pool, &result.job_id, &extracted.full_text).await {
    tracing::warn!("RAG chunk failed (non-fatal): {}", e);
}
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/pipeline/rag.rs processing/src/pipeline/mod.rs \
        processing/src/pipeline/text.rs processing/Cargo.toml
git commit -m "R08b-T04: RAG chunking in ingest pipeline — chunks stored post-text-extraction"
```

---

## R08b-T05

In `src-tauri/src/commands.rs`, append:
```rust
use xcalibre_processing::db::ai_queries::{get_ai_config, upsert_ai_config, get_book_chunks, AiConfig};
use xcalibre_ai::ollama::OllamaBackend;
use xcalibre_ai::provider::AiProvider;
use xcalibre_ai::{ChatMessage, MessageRole};

#[tauri::command]
pub async fn get_ai_config_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
) -> Result<Option<AiConfig>, String> {
    get_ai_config(pool.inner().as_ref(), None).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_ai_config_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    provider: String, model: String, embed_model: String,
    base_url: String, reasoning_strategy: String,
) -> Result<(), String> {
    let cfg = AiConfig { provider, model, embed_model, base_url, api_key: None,
        reasoning_strategy, include_fields: "[\"title\",\"authors\",\"tags\"]".into() };
    upsert_ai_config(pool.inner().as_ref(), None, &cfg).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_ai_context_chunks(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    book_id: String,
    query: String,
) -> Result<Vec<String>, String> {
    let chunks = get_book_chunks(pool.inner().as_ref(), &book_id)
        .await.map_err(|e| e.to_string())?;
    // Simple cosine similarity retrieval; return top 5 chunks by text relevance
    // (embedding-based ANN search can replace this in a follow-on)
    let query_lower = query.to_lowercase();
    let mut scored: Vec<(usize, &str)> = chunks.iter()
        .enumerate()
        .map(|(i, (_, text, _))| {
            let score = text.to_lowercase().split_whitespace()
                .filter(|w| query_lower.contains(*w)).count();
            (score, text.as_str())
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(scored.into_iter().take(5).map(|(_, t)| t.to_string()).collect())
}

#[tauri::command]
pub async fn ai_chat(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    book_id: String,
    query: String,
    history: Vec<serde_json::Value>,
) -> Result<xcalibre_ai::ChatResponse, String> {
    let cfg = get_ai_config(pool.inner().as_ref(), None)
        .await.map_err(|e| e.to_string())?
        .ok_or("No AI provider configured")?;
    let backend = OllamaBackend::new(&cfg.base_url, &cfg.model, &cfg.embed_model);
    let chunks = get_book_chunks(pool.inner().as_ref(), &book_id)
        .await.map_err(|e| e.to_string())?;
    let context: String = chunks.iter().take(5)
        .map(|(_, t, _)| t.as_str()).collect::<Vec<_>>().join("\n\n");
    let system = format!("You are a helpful assistant discussing a book. \
        Relevant passages:\n\n{}\n\nAnswer the user's question based on these passages \
        and your general knowledge.", context);
    let mut messages = vec![ChatMessage { role: MessageRole::System, content: system }];
    for msg in &history {
        if let (Some(role), Some(content)) = (msg["role"].as_str(), msg["content"].as_str()) {
            let r = if role == "user" { MessageRole::User } else { MessageRole::Assistant };
            messages.push(ChatMessage { role: r, content: content.to_string() });
        }
    }
    messages.push(ChatMessage { role: MessageRole::User, content: query });
    backend.chat(&messages, None).await.map_err(|e| e.to_string())
}
```

Add `xcalibre-ai = { path = "../xcalibre-ai" }` to `src-tauri/Cargo.toml`.
Register all commands in `generate_handler!`. Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add src-tauri/src/commands.rs src-tauri/src/main.rs src-tauri/Cargo.toml
git commit -m "R08b-T05: AI Tauri commands (ai_chat, get_ai_config, get_ai_context_chunks)"
```

---

## R08b-T06

Write `ui/src/components/AIChatPanel.tsx`:
```tsx
import { useState, useRef, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Book {
  id: string; title: string; authors: string[]
  format: string; cover_path: string | null
  progress_percent: number; last_opened_at: string | null
}

interface Message { role: "user" | "assistant"; content: string }

const QUICK_ACTIONS = [
  { id: "summarize", label: "Summarize", prompt: "Give me a concise summary of this book." },
  { id: "chapters",  label: "Chapters",  prompt: "What are the main chapters or sections?" },
  { id: "read_next", label: "Read Next", prompt: "Based on this book, what should I read next?" },
  { id: "universe",  label: "Universe",  prompt: "Tell me about the world and universe of this book." },
  { id: "series",    label: "Series",    prompt: "Is this part of a series? What's the reading order?" },
]

interface Props { book: Book; onClose: () => void }

export function AIChatPanel({ book, onClose }: Props) {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState("")
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => { bottomRef.current?.scrollIntoView({ behavior: "smooth" }) }, [messages])

  const send = async (query: string) => {
    if (!query.trim() || loading) return
    const userMsg: Message = { role: "user", content: query }
    const nextMessages = [...messages, userMsg]
    setMessages(nextMessages)
    setInput("")
    setLoading(true)
    setError(null)
    try {
      const history = nextMessages.map(m => ({ role: m.role, content: m.content }))
      const resp = await invoke<{ content: string }>("ai_chat", {
        bookId: book.id, query, history,
      })
      setMessages(prev => [...prev, { role: "assistant", content: resp.content }])
    } catch (e) { setError(String(e)) }
    finally { setLoading(false) }
  }

  return (
    <div data-testid="ai-chat-panel" className="flex flex-col h-full bg-white dark:bg-gray-900">
      <div className="flex items-center justify-between px-4 py-3 border-b dark:border-gray-700">
        <h3 className="font-semibold text-sm dark:text-white">AI · {book.title}</h3>
        <button data-testid="ai-panel-close" onClick={onClose} className="text-gray-400 hover:text-gray-600 text-lg">×</button>
      </div>

      {messages.length === 0 && (
        <div className="p-4 flex flex-wrap gap-2">
          {QUICK_ACTIONS.map(a => (
            <button key={a.id} data-testid={`quick-action-${a.id}`} onClick={() => send(a.prompt)}
              className="text-xs px-3 py-1.5 bg-blue-50 dark:bg-blue-900/30 text-blue-600
                         dark:text-blue-400 rounded-full border border-blue-200 dark:border-blue-800
                         hover:bg-blue-100 transition-colors">
              {a.label}
            </button>
          ))}
        </div>
      )}

      <div className="flex-1 overflow-y-auto px-4 py-2 space-y-4">
        {messages.map((m, i) => (
          <div key={i} className={`flex ${m.role === "user" ? "justify-end" : "justify-start"}`}>
            <div className={`max-w-[85%] rounded-xl px-3 py-2 text-sm
              ${m.role === "user"
                ? "bg-blue-600 text-white"
                : "bg-gray-100 dark:bg-gray-800 dark:text-gray-200"}`}>
              {m.content}
            </div>
          </div>
        ))}
        {loading && (
          <div className="flex justify-start">
            <div className="bg-gray-100 dark:bg-gray-800 rounded-xl px-3 py-2 text-sm text-gray-500">
              Thinking…
            </div>
          </div>
        )}
        {error && <p className="text-red-500 text-xs">{error}</p>}
        <div ref={bottomRef}/>
      </div>

      <div className="px-4 py-3 border-t dark:border-gray-700 flex gap-2">
        <input
          value={input} onChange={e => setInput(e.target.value)}
          onKeyDown={e => e.key === "Enter" && send(input)}
          placeholder="Ask about this book…"
          className="flex-1 text-sm border rounded-lg px-3 py-2 dark:bg-gray-800 dark:text-white
                     dark:border-gray-600 outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button data-testid="send-message-btn" onClick={() => send(input)} disabled={loading}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium
                     hover:bg-blue-700 disabled:opacity-50">
          Send
        </button>
      </div>
    </div>
  )
}
```

```bash
cd ui && npm test -- --reporter=verbose 2>&1 | tail -20 && cd ..
git add ui/src/components/AIChatPanel.tsx
git commit -m "R08b-T06: AIChatPanel component — AI chat panel tests green"
```

---

## R08b-T07

Write `ui/src/components/BookDiscussDialog.tsx`:
```tsx
import { AIChatPanel } from "./AIChatPanel"

interface Book {
  id: string; title: string; authors: string[]
  format: string; cover_path: string | null
  progress_percent: number; last_opened_at: string | null
}
interface Props { book: Book; onClose: () => void }

export function BookDiscussDialog({ book, onClose }: Props) {
  return (
    <div data-testid="book-discuss-dialog"
         className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-900 rounded-xl shadow-2xl w-full max-w-lg h-[600px] flex flex-col overflow-hidden">
        <AIChatPanel book={book} onClose={onClose} />
      </div>
    </div>
  )
}
```

```bash
cd ui && npm test -- --reporter=verbose 2>&1 | tail -20 && cd ..
git add ui/src/components/BookDiscussDialog.tsx
git commit -m "R08b-T07: BookDiscussDialog component — all AI dialog tests green"
```

---

## R08b-T08

Wire `Ctrl+Alt+A` keyboard shortcut in `ui/src/App.tsx`:
```tsx
// Add useEffect to listen for Ctrl+Alt+A
useEffect(() => {
  const handleKey = (e: KeyboardEvent) => {
    if (e.ctrlKey && e.altKey && e.key === "a") {
      if (selectedBook) setShowDiscuss(true)
    }
  }
  window.addEventListener("keydown", handleKey)
  return () => window.removeEventListener("keydown", handleKey)
}, [selectedBook])
```

Also add a Discuss button to `BookDetail.tsx` that opens `BookDiscussDialog`.

```bash
cargo build --workspace
cd ui && npm run build && npm test && cd ..
git add ui/src/App.tsx ui/src/components/BookDetail.tsx
git commit -m "R08b-T08: Ctrl+Alt+A shortcut + Discuss button in BookDetail"
```

---

## R08b-T09 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm run build && npm test && cd ..
```

**Visual inspection (requires Ollama running locally):**
```bash
# Start Ollama if not running
ollama serve &
ollama pull llama3
ollama pull nomic-embed-text

cargo tauri dev 2>&1 &
sleep 8
```

Verify:
- [ ] Ingest an EPUB — check that book_chunks are created in the DB
- [ ] Open BookDiscussDialog (Ctrl+Alt+A or Discuss button)
- [ ] Five quick action buttons visible
- [ ] Clicking "Summarize" sends a request and shows a response
- [ ] Typing a custom question and pressing Enter sends a message
- [ ] Conversation history accumulates correctly
- [ ] Without Ollama running: shows an error message (not a crash)

```bash
git add -A
git commit -m "R08b-T09: RMP-08 AI + RAG — all tests green, Ollama chat verified"
```
