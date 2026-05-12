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
