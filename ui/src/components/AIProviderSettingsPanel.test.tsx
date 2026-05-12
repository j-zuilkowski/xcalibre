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
