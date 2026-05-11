# RMP-14a — AI Additional Providers (Red: Failing Tests)

> Prerequisite: rmp08b complete (xcalibre-ai crate + OllamaBackend must exist).
> TDD role: RED — define additional provider contracts via failing tests.
> Providers: OpenAI, Google Gemini, GitHub Copilot, LM Studio, OpenRouter.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R14a-T01 | Failing tests: OpenAI provider trait conformance | ⬜ |
| R14a-T02 | Failing tests: Google Gemini provider | ⬜ |
| R14a-T03 | Failing tests: LM Studio provider | ⬜ |
| R14a-T04 | Failing tests: OpenRouter provider | ⬜ |
| R14a-T05 | Failing tests: AIProviderSettingsPanel component | ⬜ |

---

## R14a-T01

Write `xcalibre-ai/tests/test_openai_provider.rs`:
```rust
use xcalibre_ai::{
    AiProvider, ChatMessage, MessageRole,
    openai::OpenAiBackend,
};

fn make_backend() -> OpenAiBackend {
    OpenAiBackend::new(
        "https://api.openai.com/v1",
        "test-key",
        "gpt-4o-mini",
        "text-embedding-3-small",
    )
}

#[test]
fn test_openai_backend_name() {
    let b = make_backend();
    assert_eq!(b.name(), "openai");
}

#[test]
fn test_openai_backend_stores_config() {
    let b = make_backend();
    assert_eq!(b.model(), "gpt-4o-mini");
    assert_eq!(b.embed_model(), "text-embedding-3-small");
}

#[tokio::test]
async fn test_openai_list_models_returns_error_with_bad_key() {
    // Should fail gracefully (not panic) with an invalid API key
    let b = OpenAiBackend::new(
        "https://api.openai.com/v1",
        "sk-invalid-key-for-testing",
        "gpt-4o",
        "text-embedding-3-small",
    );
    // We expect either an HTTP error or an auth error — but NOT a panic
    let result = b.list_models().await;
    assert!(result.is_err(), "invalid key must return error");
}
```

```bash
cargo test -p xcalibre-ai 2>&1 | grep -E "^error" | head -5
```

Expected: `openai::OpenAiBackend` not found. RED confirmed.

```bash
git add xcalibre-ai/tests/test_openai_provider.rs
git commit -m "R14a-T01: failing tests for OpenAI provider"
```

---

## R14a-T02

Write `xcalibre-ai/tests/test_gemini_provider.rs`:
```rust
use xcalibre_ai::{AiProvider, gemini::GeminiBackend};

fn make_backend() -> GeminiBackend {
    GeminiBackend::new(
        "https://generativelanguage.googleapis.com/v1beta",
        "test-key",
        "gemini-2.0-flash",
        "text-embedding-004",
    )
}

#[test]
fn test_gemini_backend_name() {
    assert_eq!(make_backend().name(), "gemini");
}

#[test]
fn test_gemini_backend_stores_model() {
    assert_eq!(make_backend().model(), "gemini-2.0-flash");
}

#[tokio::test]
async fn test_gemini_list_models_errors_gracefully() {
    let b = GeminiBackend::new(
        "https://generativelanguage.googleapis.com/v1beta",
        "invalid-key",
        "gemini-2.0-flash",
        "text-embedding-004",
    );
    assert!(b.list_models().await.is_err());
}
```

```bash
git add xcalibre-ai/tests/test_gemini_provider.rs
git commit -m "R14a-T02: failing tests for Google Gemini provider"
```

---

## R14a-T03

Write `xcalibre-ai/tests/test_lmstudio_provider.rs`:
```rust
use xcalibre_ai::{AiProvider, lmstudio::LmStudioBackend};

fn make_backend() -> LmStudioBackend {
    // LM Studio exposes an OpenAI-compatible API on localhost:1234
    LmStudioBackend::new("http://localhost:1234", "loaded-model-id", "nomic-embed-text")
}

#[test]
fn test_lmstudio_backend_name() {
    assert_eq!(make_backend().name(), "lmstudio");
}

#[test]
fn test_lmstudio_backend_model() {
    assert_eq!(make_backend().model(), "loaded-model-id");
}

#[tokio::test]
async fn test_lmstudio_errors_when_not_running() {
    // Port 19999 should not be running LM Studio
    let b = LmStudioBackend::new("http://localhost:19999", "model", "embed");
    let result = b.list_models().await;
    assert!(result.is_err(), "should fail when LM Studio is not running");
}
```

```bash
git add xcalibre-ai/tests/test_lmstudio_provider.rs
git commit -m "R14a-T03: failing tests for LM Studio provider"
```

---

## R14a-T04

Write `xcalibre-ai/tests/test_openrouter_provider.rs`:
```rust
use xcalibre_ai::{AiProvider, openrouter::OpenRouterBackend};

fn make_backend() -> OpenRouterBackend {
    OpenRouterBackend::new(
        "https://openrouter.ai/api/v1",
        "test-key",
        "mistralai/mistral-7b-instruct",
        "openai/text-embedding-ada-002",
    )
}

#[test]
fn test_openrouter_backend_name() {
    assert_eq!(make_backend().name(), "openrouter");
}

#[test]
fn test_openrouter_backend_model() {
    assert_eq!(make_backend().model(), "mistralai/mistral-7b-instruct");
}

#[tokio::test]
async fn test_openrouter_errors_with_bad_key() {
    let b = OpenRouterBackend::new(
        "https://openrouter.ai/api/v1",
        "invalid",
        "mistralai/mistral-7b-instruct",
        "openai/text-embedding-ada-002",
    );
    assert!(b.list_models().await.is_err());
}
```

```bash
git add xcalibre-ai/tests/test_openrouter_provider.rs
git commit -m "R14a-T04: failing tests for OpenRouter provider"
```

---

## R14a-T05

Write `ui/src/components/AIProviderSettingsPanel.test.tsx`:
```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { AIProviderSettingsPanel } from "./AIProviderSettingsPanel"
import { mockInvoke } from "../test/setup"

const mockConfig = {
  provider: "ollama", model: "llama3", embed_model: "nomic-embed-text",
  base_url: "http://localhost:11434", api_key: null, reasoning_strategy: "auto",
  include_fields: '["title","authors"]',
}

beforeEach(() => {
  mockInvoke("get_ai_config", mockConfig)
  mockInvoke("save_ai_config", undefined)
  mockInvoke("ai_list_models", ["llama3", "llama3.1", "mistral"])
})

describe("AIProviderSettingsPanel", () => {
  it("renders provider selector", async () => {
    render(<AIProviderSettingsPanel onClose={vi.fn()} />)
    await waitFor(() =>
      expect(screen.getByTestId("ai-provider-select")).toBeInTheDocument()
    )
  })

  it("shows Ollama, OpenAI, Gemini, LM Studio, OpenRouter options", async () => {
    render(<AIProviderSettingsPanel onClose={vi.fn()} />)
    await waitFor(() => screen.getByTestId("ai-provider-select"))
    expect(screen.getByText("Ollama")).toBeInTheDocument()
    expect(screen.getByText("OpenAI")).toBeInTheDocument()
    expect(screen.getByText("Google Gemini")).toBeInTheDocument()
    expect(screen.getByText("LM Studio")).toBeInTheDocument()
    expect(screen.getByText("OpenRouter")).toBeInTheDocument()
  })

  it("shows API key field for non-local providers", async () => {
    render(<AIProviderSettingsPanel onClose={vi.fn()} />)
    await waitFor(() => screen.getByTestId("ai-provider-select"))
    fireEvent.change(screen.getByTestId("ai-provider-select"), { target: { value: "openai" } })
    await waitFor(() =>
      expect(screen.getByTestId("ai-api-key-input")).toBeInTheDocument()
    )
  })

  it("saves config on submit", async () => {
    const onClose = vi.fn()
    render(<AIProviderSettingsPanel onClose={onClose} />)
    await waitFor(() => screen.getByTestId("ai-provider-select"))
    fireEvent.click(screen.getByTestId("ai-settings-save-btn"))
    await waitFor(() => expect(onClose).toHaveBeenCalled())
  })

  it("shows test-connection button", async () => {
    render(<AIProviderSettingsPanel onClose={vi.fn()} />)
    await waitFor(() =>
      expect(screen.getByTestId("ai-test-connection-btn")).toBeInTheDocument()
    )
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/AIProviderSettingsPanel.test.tsx
git commit -m "R14a-T05: failing tests for AIProviderSettingsPanel"
```

---

### ✅ RED Checkpoint

```bash
cargo test -p xcalibre-ai 2>&1 | grep -c "^error"
cd ui && npm test 2>&1 | grep -c "FAIL" && cd ..
```

Both should show failures. Proceed to **rmp14b**.
