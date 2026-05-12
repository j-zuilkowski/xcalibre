import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { ConversionDialog } from "./ConversionDialog"
import { mockInvoke } from "../test/setup"

const mockBook = {
  id: "b1", title: "Test Book", authors: ["A"], format: "EPUB",
  cover_path: null, progress_percent: 0, last_opened_at: null,
}

beforeEach(() => {
  mockInvoke("convert_book", "/tmp/Test Book.txt")
})

describe("ConversionDialog", () => {
  it("renders format selector", () => {
    render(<ConversionDialog book={mockBook} onClose={vi.fn()} />)
    expect(screen.getByTestId("output-format-select")).toBeInTheDocument()
  })

  it("shows TXT, HTML, DOCX options", () => {
    render(<ConversionDialog book={mockBook} onClose={vi.fn()} />)
    expect(screen.getByText("TXT")).toBeInTheDocument()
    expect(screen.getByText("HTML")).toBeInTheDocument()
    expect(screen.getByText("DOCX")).toBeInTheDocument()
  })

  it("calls convert_book on submit", async () => {
    render(<ConversionDialog book={mockBook} onClose={vi.fn()} />)
    fireEvent.change(screen.getByTestId("output-format-select"), { target: { value: "TXT" } })
    fireEvent.click(screen.getByTestId("convert-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("conversion-success")).toBeInTheDocument()
    )
  })
})
