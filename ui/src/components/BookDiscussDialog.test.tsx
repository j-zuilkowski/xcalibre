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
    expect(screen.getByText(/Dune/)).toBeInTheDocument()
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
