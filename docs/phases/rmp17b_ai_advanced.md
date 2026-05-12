# RMP-17b — AI Advanced Features (Green: Implementation)

> Prerequisite: rmp17a complete, rmp08b complete, rmp11b complete (notes).
> TDD role: GREEN — implement citations, reasoning budget, save-as-note, UI components.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R17b-T01 | `xcalibre-ai/src/citations.rs` — citation extraction | ⬜ |
| R17b-T02 | `xcalibre-ai/src/reasoning.rs` — reasoning budget | ⬜ |
| R17b-T03 | `db/ai_note.rs` — save AI response as note | ⬜ |
| R17b-T04 | Wire citations + reasoning into ai_chat command | ⬜ |
| R17b-T05 | `CitedResponseView.tsx` + `ReasoningBudgetSelector.tsx` | ⬜ |
| R17b-T06 | Update AIChatPanel to use citations and reasoning budget | ⬜ |
| R17b-T07 | Milestone check + visual inspection | ⬜ |

---

## R17b-T01

Write `xcalibre-ai/src/citations.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub chunk_index:      usize,
    pub chunk_text:       String,
    pub relevance_score:  f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitedResponse {
    pub response_text: String,
    pub citations:     Vec<Citation>,
}

/// Extract citations by computing word-overlap between the response and each chunk.
/// Returns chunks whose overlap score exceeds a minimum threshold.
pub fn extract_citations(response: &str, chunks: &[&str]) -> CitedResponse {
    const MIN_SCORE: f32 = 0.05;

    let resp_words: std::collections::HashSet<String> = tokenize(response);

    let mut citations: Vec<Citation> = chunks
        .iter()
        .enumerate()
        .filter_map(|(idx, chunk)| {
            let chunk_words: std::collections::HashSet<String> = tokenize(chunk);
            if chunk_words.is_empty() || resp_words.is_empty() { return None; }

            let overlap = resp_words.intersection(&chunk_words).count();
            let score = overlap as f32 / resp_words.len().max(chunk_words.len()) as f32;

            if score >= MIN_SCORE {
                Some(Citation {
                    chunk_index:     idx,
                    chunk_text:      chunk.to_string(),
                    relevance_score: score.min(1.0),
                })
            } else {
                None
            }
        })
        .collect();

    // Sort by relevance descending
    citations.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

    CitedResponse {
        response_text: response.to_string(),
        citations,
    }
}

fn tokenize(text: &str) -> std::collections::HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3)
        .map(|w| w.to_string())
        .collect()
}
```

Add to `xcalibre-ai/src/lib.rs`:
```rust
pub mod citations;
pub use citations::{extract_citations, Citation, CitedResponse};
```

Then run:
```bash
cargo test -p xcalibre-ai -- test_citations
git add xcalibre-ai/src/citations.rs xcalibre-ai/src/lib.rs
git commit -m "R17b-T01: citation extraction — all citation tests green"
```

---

## R17b-T02

Write `xcalibre-ai/src/reasoning.rs`:
```rust
use crate::{ChatMessage, MessageRole};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningBudget { None, Auto, Low, Medium, High }

impl ReasoningBudget {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "none"   => Some(Self::None),
            "auto"   => Some(Self::Auto),
            "low"    => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high"   => Some(Self::High),
            _        => None,
        }
    }
}

/// Prepend a system message that instructs the model on reasoning depth.
/// Does not modify the messages if budget is None.
pub fn apply_reasoning_budget(
    mut messages: Vec<ChatMessage>,
    budget: ReasoningBudget,
) -> Vec<ChatMessage> {
    let instruction = match budget {
        ReasoningBudget::None   => return messages,
        ReasoningBudget::Auto   => "Think carefully before answering.",
        ReasoningBudget::Low    => "Briefly consider the question before responding.",
        ReasoningBudget::Medium => "Think step by step, then provide a concise answer.",
        ReasoningBudget::High   =>
            "Reason extensively step by step. \
             Explore multiple perspectives, identify key evidence, \
             then synthesize a well-structured answer.",
    };

    // Insert reasoning instruction as first system message (or prepend if none exists)
    let system_msg = ChatMessage { role: MessageRole::System, content: instruction.to_string() };
    messages.insert(0, system_msg);
    messages
}
```

Add to `xcalibre-ai/src/lib.rs`:
```rust
pub mod reasoning;
pub use reasoning::{ReasoningBudget, apply_reasoning_budget};
```

Then run:
```bash
cargo test -p xcalibre-ai -- test_reasoning
git add xcalibre-ai/src/reasoning.rs xcalibre-ai/src/lib.rs
git commit -m "R17b-T02: reasoning budget — all reasoning tests green"
```

---

## R17b-T03

Write `processing/src/db/ai_note.rs`:
```rust
use crate::db::notes_queries::{create_note, NewNote};
use sqlx::sqlite::SqlitePool;

pub async fn save_ai_response_as_note(
    pool: &SqlitePool,
    book_id: &str,
    title: &str,
    body_html: &str,
    body_text: &str,
) -> Result<(), sqlx::Error> {
    let new = NewNote {
        book_id:   book_id.to_string(),
        title:     title.to_string(),
        body_html: body_html.to_string(),
        body_text: body_text.to_string(),
    };
    create_note(pool, &new).await?;
    Ok(())
}
```

Add to `processing/src/db/mod.rs`:
```rust
pub mod ai_note;
```

Add Tauri command in `src-tauri/src/commands/ai.rs`:
```rust
#[tauri::command]
pub async fn save_ai_response_as_note_cmd(
    book_id:   String,
    title:     String,
    body_html: String,
    body_text: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    xcalibre_processing::db::ai_note::save_ai_response_as_note(
        &state.pool, &book_id, &title, &body_html, &body_text
    ).await.map_err(|e| e.to_string())
}
```

Register in `src-tauri/src/main.rs`:
```rust
commands::ai::save_ai_response_as_note_cmd,
```

Then run:
```bash
cargo test --workspace -- test_save_ai_note
git add processing/src/db/ai_note.rs processing/src/db/mod.rs \
        src-tauri/src/commands/ai.rs src-tauri/src/main.rs
git commit -m "R17b-T03: save_ai_response_as_note — test green"
```

---

## R17b-T04

Update `src-tauri/src/commands/ai.rs`, in `ai_chat` command:
- Import `xcalibre_ai::{extract_citations, apply_reasoning_budget, ReasoningBudget}` 
- After getting the response, call `extract_citations(&response.content, &chunk_texts)`
- Apply reasoning budget by calling `apply_reasoning_budget(messages, budget)` before sending
- Return a new struct `AiChatResult { response: ChatResponse, citations: Vec<Citation> }` instead of bare `ChatResponse`

```rust
use xcalibre_ai::{extract_citations, apply_reasoning_budget, ReasoningBudget, CitedResponse};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AiChatResult {
    pub content:   String,
    pub model:     String,
    pub done:      bool,
    pub reasoning: Option<String>,
    pub citations: Vec<xcalibre_ai::Citation>,
}

// In ai_chat command, after getting response:
let chunk_texts: Vec<&str> = chunks.iter().map(|c| c.as_str()).collect();
let cited = extract_citations(&response.content, &chunk_texts);

Ok(AiChatResult {
    content:   response.content,
    model:     response.model,
    done:      response.done,
    reasoning: response.reasoning,
    citations: cited.citations,
})
```

```bash
cargo build --workspace
git add src-tauri/src/commands/ai.rs
git commit -m "R17b-T04: wire citations + reasoning budget into ai_chat command"
```

---

## R17b-T05

Write `ui/src/components/CitedResponseView.tsx`:
```tsx
import { useState } from "react"

interface Citation {
  chunk_index:     number
  chunk_text:      string
  relevance_score: number
}

interface CitedResponse {
  response_text: string
  citations:     Citation[]
}

interface Props { cited: CitedResponse }

export function CitedResponseView({ cited }: Props) {
  const [showCitations, setShowCitations] = useState(false)
  const { response_text, citations } = cited

  return (
    <div>
      <p style={{ margin: "0 0 0.5rem", lineHeight: 1.6 }}>{response_text}</p>

      {citations.length > 0 && (
        <div>
          <button
            onClick={() => setShowCitations(s => !s)}
            style={{
              padding: "0.25rem 0.5rem", background: "transparent",
              border: "1px solid var(--border, #45475a)", borderRadius: "4px",
              cursor: "pointer", color: "var(--text-muted, #6c7086)",
              fontSize: "0.8rem", display: "flex", alignItems: "center", gap: "0.4rem",
            }}
          >
            <span data-testid="citation-count">{citations.length}</span>
            {citations.length === 1 ? "source" : "sources"}
            {showCitations ? " ▲" : " ▼"}
          </button>

          {showCitations && (
            <ul style={{
              listStyle: "none", padding: 0, margin: "0.5rem 0 0",
              borderLeft: "2px solid var(--border, #45475a)",
              paddingLeft: "0.75rem",
            }}>
              {citations.map((c, i) => (
                <li key={i} style={{
                  fontSize: "0.82rem", color: "var(--text-muted, #6c7086)",
                  marginBottom: "0.4rem", fontStyle: "italic",
                }}>
                  "{c.chunk_text.slice(0, 120)}{c.chunk_text.length > 120 ? "…" : ""}"
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  )
}
```

Write `ui/src/components/ReasoningBudgetSelector.tsx`:
```tsx
const OPTIONS = [
  { value: "none",   label: "None",   description: "No extra reasoning" },
  { value: "auto",   label: "Auto",   description: "Let the model decide" },
  { value: "low",    label: "Low",    description: "Brief consideration" },
  { value: "medium", label: "Medium", description: "Step-by-step thinking" },
  { value: "high",   label: "High",   description: "Deep reasoning" },
]

interface Props {
  value: string
  onChange: (value: string) => void
}

export function ReasoningBudgetSelector({ value, onChange }: Props) {
  return (
    <select
      data-testid="reasoning-budget-select"
      value={value}
      onChange={e => onChange(e.target.value)}
      style={{
        padding: "0.3rem 0.5rem",
        background: "var(--bg-overlay, #313244)",
        border: "1px solid var(--border, #45475a)",
        borderRadius: "4px", color: "inherit", fontSize: "0.85rem",
      }}
      title="Reasoning depth"
    >
      {OPTIONS.map(o => (
        <option key={o.value} value={o.value}>{o.label}</option>
      ))}
    </select>
  )
}
```

Then run:
```bash
cd ui && npm test -- CitedResponseView ReasoningBudgetSelector && cd ..
git add ui/src/components/CitedResponseView.tsx ui/src/components/ReasoningBudgetSelector.tsx
git commit -m "R17b-T05: CitedResponseView + ReasoningBudgetSelector — all UI tests green"
```

---

## R17b-T06

In `ui/src/components/AIChatPanel.tsx`:
- Replace plain text AI responses with `<CitedResponseView cited={...} />`
- Add `<ReasoningBudgetSelector>` to the toolbar
- Add "Save as Note" button on each AI response bubble:
  ```tsx
  <button onClick={() => invoke("save_ai_response_as_note_cmd", {
    bookId: book.id, title: `AI: ${question.slice(0, 50)}`,
    bodyHtml: `<p>${response.content}</p>`, bodyText: response.content,
  })}>Save as Note</button>
  ```

```bash
cd ui && npm test -- AIChatPanel && cd ..
git add ui/src/components/AIChatPanel.tsx
git commit -m "R17b-T06: AIChatPanel — citations, reasoning budget, save-as-note wired"
```

---

## R17b-T07 — Milestone Check + Visual Inspection

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
2. Open an EPUB book and trigger AI chat (Ctrl+Alt+A)
3. Ask: "What is the main theme of this book?"
4. Verify AI response appears with citation count badge (if chunks are available)
5. Click the citation badge — verify source snippets expand below the response
6. Change reasoning budget to "High" and ask again — verify response is more detailed
7. Click "Save as Note" on a response — verify it appears in the book's Notes tab
8. Change reasoning budget to "None" — verify responses are more direct/brief

```bash
pkill -x xcalibre 2>/dev/null || true
```

```bash
git add -A
git commit -m "R17b-T07: RMP-17 AI Advanced Features — all tests green, UI wired"
```
