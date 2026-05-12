import { render, screen } from "@testing-library/react"
import { describe, it, expect, beforeEach } from "vitest"
import { ConversionDialog } from "./ConversionDialog"
import { mockInvoke } from "../test/setup"

const mockBook = {
  id: "b1", title: "Test Book", authors: ["A"], format: "EPUB",
  cover_path: null, progress_percent: 0, last_opened_at: null,
  file_path: "/tmp/test.epub",
}

beforeEach(() => {
  mockInvoke("convert_book", "/tmp/Test Book.fb2")
})

describe("ConversionDialog Tier 3 formats", () => {
  it("shows FB2 option",   () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("FB2")).toBeInTheDocument() })
  it("shows RTF option",   () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("RTF")).toBeInTheDocument() })
  it("shows HTMLZ option", () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("HTMLZ")).toBeInTheDocument() })
})
