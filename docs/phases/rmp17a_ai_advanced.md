# RMP-17a — AI Advanced Features (Red: Failing Tests)

> Prerequisite: rmp08b complete (RAG pipeline + Ollama backend), rmp14b complete (multi-provider).
> TDD role: RED — define citations, reasoning budget, save-as-note, and localization APIs.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R17a-T01 | Failing tests: citation extraction from RAG chunks | ⬜ |
| R17a-T02 | Failing tests: reasoning budget configuration | ⬜ |
| R17a-T03 | Failing tests: save-AI-response-as-note command | ⬜ |
| R17a-T04 | Failing tests: CitedResponseView component | ⬜ |
| R17a-T05 | Failing tests: ReasoningBudgetSelector component | ⬜ |

---

## R17a-T01

Write `xcalibre-ai/tests/test_citations.rs`:
```rust
use xcalibre_ai::citations::{extract_citations, Citation, CitedResponse};

#[test]
fn test_no_citations_when_no_chunks() {
    let response = "This is a simple response with no citations.";
    let chunks: Vec<&str> = vec![];
    let cited = extract_citations(response, &chunks);
    assert!(cited.citations.is_empty());
    assert_eq!(cited.response_text, response);
}

#[test]
fn test_citations_extracted_from_matching_chunks() {
    let response = "The spice must flow. It is the key to space travel.";
    let chunks = vec![
        "The spice melange is the most valuable substance in the universe.",
        "Without the spice, the Spacing Guild cannot fold space.",
        "Rain fell on the mountains.",
    ];
    let cited = extract_citations(response, &chunks);
    // At least the chunk about space travel should be cited
    assert!(!cited.citations.is_empty(), "should find at least one relevant citation");
}

#[test]
fn test_citation_has_chunk_index() {
    let response = "Paul is the Kwisatz Haderach.";
    let chunks = vec!["Paul Atreides was trained as the Kwisatz Haderach by the Bene Gesserit."];
    let cited = extract_citations(response, &chunks);
    if !cited.citations.is_empty() {
        assert!(cited.citations[0].chunk_index < chunks.len());
    }
}

#[test]
fn test_citation_relevance_score_in_range() {
    let response = "Arrakis is a desert planet.";
    let chunks = vec!["Arrakis, known as Dune, is a harsh desert world."];
    let cited = extract_citations(response, &chunks);
    if !cited.citations.is_empty() {
        let score = cited.citations[0].relevance_score;
        assert!(score > 0.0 && score <= 1.0, "score must be in (0,1]: {score}");
    }
}
```

```bash
cargo test -p xcalibre-ai 2>&1 | grep -E "^error" | head -5
```

Expected: `citations` module not found. RED confirmed.

```bash
git add xcalibre-ai/tests/test_citations.rs
git commit -m "R17a-T01: failing tests for citation extraction"
```

---

## R17a-T02

Write `xcalibre-ai/tests/test_reasoning_budget.rs`:
```rust
use xcalibre_ai::reasoning::{ReasoningBudget, apply_reasoning_budget};
use xcalibre_ai::ChatMessage;

#[test]
fn test_budget_none_adds_no_system_message() {
    let messages: Vec<ChatMessage> = vec![];
    let result = apply_reasoning_budget(messages.clone(), ReasoningBudget::None);
    assert_eq!(result.len(), messages.len());
}

#[test]
fn test_budget_auto_adds_system_message() {
    let messages: Vec<ChatMessage> = vec![];
    let result = apply_reasoning_budget(messages, ReasoningBudget::Auto);
    assert!(result.len() >= 1, "Auto budget must add at least a system message");
}

#[test]
fn test_budget_high_adds_detailed_reasoning_prompt() {
    let messages: Vec<ChatMessage> = vec![];
    let result = apply_reasoning_budget(messages, ReasoningBudget::High);
    assert!(result.iter().any(|m| {
        m.content.to_lowercase().contains("step") ||
        m.content.to_lowercase().contains("reason")
    }), "High budget must add reasoning instructions");
}

#[test]
fn test_budget_from_str() {
    assert_eq!(ReasoningBudget::from_str("none"),   Some(ReasoningBudget::None));
    assert_eq!(ReasoningBudget::from_str("auto"),   Some(ReasoningBudget::Auto));
    assert_eq!(ReasoningBudget::from_str("low"),    Some(ReasoningBudget::Low));
    assert_eq!(ReasoningBudget::from_str("medium"), Some(ReasoningBudget::Medium));
    assert_eq!(ReasoningBudget::from_str("high"),   Some(ReasoningBudget::High));
    assert_eq!(ReasoningBudget::from_str("unknown"), None);
}
```

```bash
cargo test -p xcalibre-ai 2>&1 | grep -E "^error" | head -5
```

RED confirmed.

```bash
git add xcalibre-ai/tests/test_reasoning_budget.rs
git commit -m "R17a-T02: failing tests for reasoning budget"
```

---

## R17a-T03

Write `processing/tests/test_save_ai_note.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::notes_queries::{list_notes, NoteRow};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_save_ai_response_as_note() {
    let pool = setup().await;
    // Insert a book first
    sqlx::query(
        "INSERT INTO local_books (id, title, authors, format) VALUES ('b1', 'Dune', '[]', 'EPUB')"
    ).execute(&pool).await.unwrap();

    xcalibre_processing::db::ai_note::save_ai_response_as_note(
        &pool,
        "b1",
        "AI Summary",
        "<p>Paul Atreides travels to Arrakis.</p>",
        "Paul Atreides travels to Arrakis.",
    ).await.expect("save");

    let notes = list_notes(&pool, "b1").await.unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "AI Summary");
    assert!(notes[0].body_html.contains("Arrakis"));
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

Expected: `db::ai_note` not found. RED confirmed.

```bash
git add processing/tests/test_save_ai_note.rs
git commit -m "R17a-T03: failing test for save_ai_response_as_note"
```

---

## R17a-T04

Write `ui/src/components/CitedResponseView.test.tsx`:
```tsx
import { render, screen } from "@testing-library/react"
import { describe, it, expect } from "vitest"
import { CitedResponseView } from "./CitedResponseView"

const mockCited = {
  response_text: "The spice melange enables space travel.",
  citations: [
    { chunk_index: 0, chunk_text: "Spice is essential for navigation.", relevance_score: 0.9 },
    { chunk_index: 1, chunk_text: "Navigators use the spice to fold space.", relevance_score: 0.7 },
  ],
}

describe("CitedResponseView", () => {
  it("renders response text", () => {
    render(<CitedResponseView cited={mockCited} />)
    expect(screen.getByText(/spice melange enables/i)).toBeInTheDocument()
  })

  it("renders citation count badge", () => {
    render(<CitedResponseView cited={mockCited} />)
    expect(screen.getByTestId("citation-count")).toBeInTheDocument()
    expect(screen.getByTestId("citation-count")).toHaveTextContent("2")
  })

  it("renders citation snippets", () => {
    render(<CitedResponseView cited={mockCited} />)
    expect(screen.getByText(/Spice is essential/)).toBeInTheDocument()
    expect(screen.getByText(/fold space/)).toBeInTheDocument()
  })

  it("renders without citations when citations array is empty", () => {
    const noCitations = { response_text: "No context.", citations: [] }
    render(<CitedResponseView cited={noCitations} />)
    expect(screen.queryByTestId("citation-count")).not.toBeInTheDocument()
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/CitedResponseView.test.tsx
git commit -m "R17a-T04: failing tests for CitedResponseView"
```

---

## R17a-T05

Write `ui/src/components/ReasoningBudgetSelector.test.tsx`:
```tsx
import { render, screen, fireEvent } from "@testing-library/react"
import { describe, it, expect, vi } from "vitest"
import { ReasoningBudgetSelector } from "./ReasoningBudgetSelector"

describe("ReasoningBudgetSelector", () => {
  it("renders selector with all options", () => {
    render(<ReasoningBudgetSelector value="auto" onChange={vi.fn()} />)
    const select = screen.getByTestId("reasoning-budget-select")
    expect(select).toBeInTheDocument()
    expect(screen.getByText("Auto")).toBeInTheDocument()
    expect(screen.getByText("None")).toBeInTheDocument()
    expect(screen.getByText("Low")).toBeInTheDocument()
    expect(screen.getByText("Medium")).toBeInTheDocument()
    expect(screen.getByText("High")).toBeInTheDocument()
  })

  it("shows current value as selected", () => {
    render(<ReasoningBudgetSelector value="high" onChange={vi.fn()} />)
    const select = screen.getByTestId<HTMLSelectElement>("reasoning-budget-select")
    expect(select.value).toBe("high")
  })

  it("calls onChange when selection changes", () => {
    const onChange = vi.fn()
    render(<ReasoningBudgetSelector value="auto" onChange={onChange} />)
    fireEvent.change(screen.getByTestId("reasoning-budget-select"), { target: { value: "medium" } })
    expect(onChange).toHaveBeenCalledWith("medium")
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/ReasoningBudgetSelector.test.tsx
git commit -m "R17a-T05: failing tests for ReasoningBudgetSelector"
```

---

### ✅ RED Checkpoint

```bash
cargo test -p xcalibre-ai --workspace 2>&1 | grep -c "^error"
cd ui && npm test 2>&1 | grep -c "FAIL" && cd ..
```

Both should show failures. Proceed to **rmp17b**.
