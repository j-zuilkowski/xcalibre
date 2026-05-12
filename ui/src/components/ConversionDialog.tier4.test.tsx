import { render, screen } from "@testing-library/react"
import { describe, it, expect, beforeEach } from "vitest"
import { ConversionDialog } from "./ConversionDialog"
import { mockInvoke } from "../test/setup"

const mockBook = {
  id: "b1", title: "Test Book", authors: ["A"], format: "EPUB",
  cover_path: null, progress_percent: 0, last_opened_at: null,
  file_path: "/tmp/test.epub",
}

beforeEach(() => { mockInvoke("convert_book", "/tmp/output.lrf") })

describe("ConversionDialog Tier 4 formats", () => {
  it("shows LRF option",  () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("LRF")).toBeInTheDocument() })
  it("shows PDB option",  () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("PDB")).toBeInTheDocument() })
  it("shows PML option",  () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("PML")).toBeInTheDocument() })
  it("shows RB option",   () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("RB")).toBeInTheDocument() })
  it("shows SNB option",  () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("SNB")).toBeInTheDocument() })
  it("shows TCR option",  () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("TCR")).toBeInTheDocument() })
})
