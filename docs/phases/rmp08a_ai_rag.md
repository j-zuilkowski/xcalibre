# RMP-08a — AI Integration: Ollama MVP + RAG Pipeline (Red: Failing Tests)

> Prerequisite: rmp01b complete (per-library config needed for AI config storage).
> Decision: S7-D — RAG pipeline: chunk job_text, embed via sqlite-vec, retrieve top-K.
> TDD role: RED — define the AI provider trait, RAG pipeline, and chat UI contract.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R08a-T01 | Migration 0014 — ai_config table | ⬜ |
| R08a-T02 | Migration 0015 — book_chunks table (RAG) | ⬜ |
| R08a-T03 | xcalibre-ai crate scaffold | ⬜ |
| R08a-T04 | Failing tests: chunking algorithm | ⬜ |
| R08a-T05 | Failing tests: AI config DB queries | ⬜ |
| R08a-T06 | Failing tests: AIChatPanel component | ⬜ |
| R08a-T07 | Failing tests: BookDiscussDialog component | ⬜ |

---

## R08a-T01

Write `processing/src/db/migrations/0014_ai_config.sql`:
```sql
-- Per-library AI configuration.
CREATE TABLE IF NOT EXISTS ai_config (
    id            TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    library_id    TEXT REFERENCES libraries(id) ON DELETE CASCADE,
    provider      TEXT NOT NULL DEFAULT 'ollama',
    model         TEXT NOT NULL DEFAULT 'llama3',
    embed_model   TEXT NOT NULL DEFAULT 'nomic-embed-text',
    base_url      TEXT NOT NULL DEFAULT 'http://localhost:11434',
    api_key       TEXT,
    reasoning_strategy TEXT NOT NULL DEFAULT 'auto'
                  CHECK (reasoning_strategy IN ('auto','none','low','medium','high')),
    include_fields TEXT NOT NULL DEFAULT '["title","authors","tags","series","description"]',
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
```

```bash
cargo build --workspace
git add processing/src/db/migrations/0014_ai_config.sql
git commit -m "R08a-T01: migration 0014 — ai_config table"
```

---

## R08a-T02

Write `processing/src/db/migrations/0015_book_chunks.sql`:
```sql
-- RAG pipeline: chunked embeddings for AI book discussion (S7-D).
-- sqlite-vec stores float32 vectors as BLOBs.
CREATE TABLE IF NOT EXISTS book_chunks (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id     TEXT    NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    chunk_text  TEXT    NOT NULL,
    embedding   BLOB,             -- float32[N] stored by sqlite-vec
    embed_model TEXT,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS book_chunks_book_id ON book_chunks(book_id);
CREATE UNIQUE INDEX IF NOT EXISTS book_chunks_book_chunk ON book_chunks(book_id, chunk_index);
```

```bash
cargo build --workspace
git add processing/src/db/migrations/0015_book_chunks.sql
git commit -m "R08a-T02: migration 0015 — book_chunks table for RAG"
```

---

## R08a-T03

Add `"xcalibre-ai"` to workspace members in root `Cargo.toml`.
Create `xcalibre-ai/src/`.

Write `xcalibre-ai/Cargo.toml`:
```toml
[package]
name = "xcalibre-ai"
version = "0.1.0"
edition = "2021"

[dependencies]
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
tokio       = { version = "1", features = ["full"] }
reqwest     = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream"] }
thiserror   = "1"
futures-util = "0.3"
tracing     = "0.1"

[dev-dependencies]
tokio = { version = "1", features = ["full", "test-util"] }
```

Write `xcalibre-ai/src/lib.rs`:
```rust
pub mod provider;
pub mod ollama;
pub mod chunk;
pub mod error;

pub use error::AiError;
pub use provider::{AiProvider, ChatMessage, ChatResponse, MessageRole};
pub use chunk::{chunk_text, ChunkConfig};
```

Write `xcalibre-ai/src/error.rs`:
```rust
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("not connected: {0}")]
    NotConnected(String),
}
```

Write `xcalibre-ai/src/provider.rs` with stubs:
```rust
use crate::AiError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageRole { System, User, Assistant }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role:    MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content:   String,
    pub model:     String,
    pub done:      bool,
    pub reasoning: Option<String>,
}

/// Trait implemented by each AI provider backend.
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    async fn chat(
        &self,
        messages: &[ChatMessage],
        stream_tx: Option<tokio::sync::mpsc::Sender<String>>,
    ) -> Result<ChatResponse, AiError>;

    async fn embed(&self, text: &str) -> Result<Vec<f32>, AiError>;

    async fn list_models(&self) -> Result<Vec<String>, AiError>;

    fn name(&self) -> &str;
}
```

Add `async-trait = "0.1"` to `xcalibre-ai/Cargo.toml` dependencies.

Then run:
```bash
cargo build --workspace
git add xcalibre-ai/ Cargo.toml
git commit -m "R08a-T03: xcalibre-ai crate scaffold with AiProvider trait"
```

---

## R08a-T04

Write `xcalibre-ai/tests/test_chunking.rs`:
```rust
use xcalibre_ai::{chunk_text, ChunkConfig};

#[test]
fn test_chunk_empty_text() {
    let chunks = chunk_text("", &ChunkConfig::default());
    assert!(chunks.is_empty());
}

#[test]
fn test_chunk_short_text_is_single_chunk() {
    let text = "This is a short paragraph.";
    let chunks = chunk_text(text, &ChunkConfig::default());
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].trim(), text.trim());
}

#[test]
fn test_chunk_splits_at_token_boundary() {
    // Generate text long enough to require multiple chunks at default 512-token limit
    let long = "word ".repeat(600);
    let chunks = chunk_text(&long, &ChunkConfig::default());
    assert!(chunks.len() >= 2, "600 words must produce at least 2 chunks");
}

#[test]
fn test_chunk_overlap() {
    let cfg = ChunkConfig { max_tokens: 10, overlap_tokens: 3 };
    let text = "one two three four five six seven eight nine ten eleven twelve thirteen";
    let chunks = chunk_text(text, &cfg);
    assert!(chunks.len() >= 2);
    // The last word of chunk N should appear in chunk N+1 (overlap)
    let words_0: Vec<&str> = chunks[0].split_whitespace().collect();
    let words_1: Vec<&str> = chunks[1].split_whitespace().collect();
    let last_of_0 = words_0.last().unwrap();
    assert!(words_1.iter().any(|w| w == last_of_0),
            "overlap: '{}' should appear in chunk 1: {:?}", last_of_0, words_1);
}

#[test]
fn test_chunk_respects_sentence_boundaries() {
    let cfg = ChunkConfig { max_tokens: 20, overlap_tokens: 0 };
    let text = "First sentence here. Second sentence follows. Third sentence ends.";
    let chunks = chunk_text(text, &cfg);
    // Chunks should not split mid-sentence
    for chunk in &chunks {
        let trimmed = chunk.trim();
        if !trimmed.is_empty() {
            assert!(trimmed.ends_with('.') || trimmed.ends_with('?') || trimmed.ends_with('!')
                    || chunks.last() == Some(chunk),
                "chunk should end at sentence boundary: '{}'", trimmed);
        }
    }
}
```

```bash
cargo test -p xcalibre-ai 2>&1 | grep -E "^error" | head -10
```

Expected: `chunk_text` / `ChunkConfig` not yet implemented. RED confirmed.

```bash
git add xcalibre-ai/tests/test_chunking.rs
git commit -m "R08a-T04: failing tests for RAG chunking algorithm"
```

---

## R08a-T05

Write `processing/tests/test_ai_config.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::ai_queries::{
    get_ai_config, upsert_ai_config, AiConfig,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_upsert_and_get_ai_config() {
    let pool = setup().await;
    let cfg = AiConfig {
        provider:            "ollama".into(),
        model:               "llama3".into(),
        embed_model:         "nomic-embed-text".into(),
        base_url:            "http://localhost:11434".into(),
        api_key:             None,
        reasoning_strategy:  "auto".into(),
        include_fields:      "[\"title\"]".into(),
    };
    upsert_ai_config(&pool, None, &cfg).await.expect("upsert");
    let loaded = get_ai_config(&pool, None).await.expect("get").expect("should exist");
    assert_eq!(loaded.provider, "ollama");
    assert_eq!(loaded.embed_model, "nomic-embed-text");
}

#[tokio::test]
async fn test_ai_config_defaults_to_ollama() {
    let pool = setup().await;
    // No config inserted — should return None
    let loaded = get_ai_config(&pool, None).await.expect("get");
    assert!(loaded.is_none(), "no config yet should return None");
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

RED confirmed (`db::ai_queries` not found).

```bash
git add processing/tests/test_ai_config.rs
git commit -m "R08a-T05: failing tests for AI config DB queries"
```

---

## R08a-T06

Write `ui/src/components/AIChatPanel.test.tsx`:
```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { AIChatPanel } from "./AIChatPanel"
import { mockInvoke } from "../test/setup"

const mockBook = {
  id: "b1", title: "Test Book", authors: ["Test Author"],
  format: "EPUB", cover_path: null, progress_percent: 0, last_opened_at: null,
}

beforeEach(() => {
  mockInvoke("get_ai_config", { provider: "ollama", model: "llama3", base_url: "http://localhost:11434" })
  mockInvoke("ai_chat", { content: "Here is a summary of the book.", model: "llama3", done: true, reasoning: null })
  mockInvoke("get_ai_context_chunks", ["relevant chunk one", "relevant chunk two"])
})

describe("AIChatPanel", () => {
  it("renders input and send button", () => {
    render(<AIChatPanel book={mockBook} onClose={vi.fn()} />)
    expect(screen.getByPlaceholderText(/Ask about this book/i)).toBeInTheDocument()
    expect(screen.getByTestId("send-message-btn")).toBeInTheDocument()
  })

  it("shows quick action buttons", () => {
    render(<AIChatPanel book={mockBook} onClose={vi.fn()} />)
    expect(screen.getByTestId("quick-action-summarize")).toBeInTheDocument()
  })

  it("sends a message and shows response", async () => {
    render(<AIChatPanel book={mockBook} onClose={vi.fn()} />)
    fireEvent.change(screen.getByPlaceholderText(/Ask about this book/i), {
      target: { value: "What is this book about?" }
    })
    fireEvent.click(screen.getByTestId("send-message-btn"))
    await waitFor(() =>
      expect(screen.getByText(/Here is a summary/i)).toBeInTheDocument()
    )
  })

  it("closes when × is clicked", () => {
    const onClose = vi.fn()
    render(<AIChatPanel book={mockBook} onClose={onClose} />)
    fireEvent.click(screen.getByTestId("ai-panel-close"))
    expect(onClose).toHaveBeenCalled()
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
```

RED confirmed. Commit:
```bash
git add ui/src/components/AIChatPanel.test.tsx
git commit -m "R08a-T06: failing tests for AIChatPanel"
```

---

## R08a-T07

Write `ui/src/components/BookDiscussDialog.test.tsx`:
```tsx
import { render, screen, waitFor, fireEvent } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { BookDiscussDialog } from "./BookDiscussDialog"
import { mockInvoke } from "../test/setup"

const mockBook = {
  id: "b1", title: "Dune", authors: ["Frank Herbert"],
  format: "EPUB", cover_path: null, progress_percent: 0, last_opened_at: null,
}

beforeEach(() => {
  mockInvoke("get_ai_config", { provider: "ollama", model: "llama3", base_url: "http://localhost:11434" })
  mockInvoke("ai_chat", { content: "Dune is a sci-fi epic.", model: "llama3", done: true, reasoning: null })
  mockInvoke("get_ai_context_chunks", [])
})

describe("BookDiscussDialog", () => {
  it("shows book title in header", async () => {
    render(<BookDiscussDialog book={mockBook} onClose={vi.fn()} />)
    expect(screen.getByText("Dune")).toBeInTheDocument()
  })

  it("shows all 5 default quick actions", async () => {
    render(<BookDiscussDialog book={mockBook} onClose={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByTestId("quick-action-summarize")).toBeInTheDocument()
      expect(screen.getByTestId("quick-action-chapters")).toBeInTheDocument()
      expect(screen.getByTestId("quick-action-read_next")).toBeInTheDocument()
      expect(screen.getByTestId("quick-action-universe")).toBeInTheDocument()
      expect(screen.getByTestId("quick-action-series")).toBeInTheDocument()
    })
  })

  it("clicking a quick action triggers ai_chat invoke", async () => {
    render(<BookDiscussDialog book={mockBook} onClose={vi.fn()} />)
    await waitFor(() => screen.getByTestId("quick-action-summarize"))
    fireEvent.click(screen.getByTestId("quick-action-summarize"))
    await waitFor(() => screen.getByText(/Dune is a sci-fi epic/))
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/BookDiscussDialog.test.tsx
git commit -m "R08a-T07: failing tests for BookDiscussDialog"
```

---

### ✅ RED Checkpoint

```bash
cargo test --workspace 2>&1 | grep -c "^error"
cd ui && npm test 2>&1 | grep -c "FAIL" && cd ..
```

Both should show failures. Proceed to **rmp08b**.
