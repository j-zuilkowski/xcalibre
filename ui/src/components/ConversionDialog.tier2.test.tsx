import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { ConversionDialog } from "./ConversionDialog"
import { mockInvoke } from "../test/setup"

const mockBook = {
  id: "b1", title: "Test Book", authors: ["A"], format: "EPUB",
  cover_path: null, progress_percent: 0, last_opened_at: null,
  file_path: "/tmp/test.epub",
}

beforeEach(() => {
  mockInvoke("convert_book", "/tmp/Test Book.pdf")
})

describe("ConversionDialog Tier 2 formats", () => {
  it("shows PDF option", () => {
    render(<ConversionDialog book={mockBook} onClose={vi.fn()} />)
    expect(screen.getByText("PDF")).toBeInTheDocument()
  })

  it("shows MOBI option", () => {
    render(<ConversionDialog book={mockBook} onClose={vi.fn()} />)
    expect(screen.getByText("MOBI")).toBeInTheDocument()
  })

  it("calls convert_book with PDF format", async () => {
    render(<ConversionDialog book={mockBook} onClose={vi.fn()} />)
    fireEvent.change(screen.getByTestId("output-format-select"), { target: { value: "PDF" } })
    fireEvent.click(screen.getByTestId("convert-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("conversion-success")).toBeInTheDocument()
    )
  })
})
