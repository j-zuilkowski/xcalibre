import { render, screen, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { EbookEditorShell } from "./EbookEditorShell"
import { mockInvoke } from "../test/setup"

const mockBook = {
  id: "b1", title: "Test Book", authors: ["Author"],
  format: "EPUB", cover_path: null, progress_percent: 0, last_opened_at: null,
  file_path: "/tmp/test.epub",
}

beforeEach(() => {
  mockInvoke("editor_open_epub", {
    spine: ["OEBPS/ch01.xhtml"],
    manifest: [{ href: "OEBPS/ch01.xhtml", media_type: "application/xhtml+xml" }],
    metadata: { title: "Test Book", authors: ["Author"] },
  })
  mockInvoke("editor_read_item", "<html><body><p>Chapter 1</p></body></html>")
  mockInvoke("editor_write_item", undefined)
  mockInvoke("editor_save_epub", undefined)
})

describe("EbookEditorShell", () => {
  it("renders file tree and editor panels", async () => {
    render(<EbookEditorShell book={mockBook} onClose={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByTestId("editor-file-tree")).toBeInTheDocument()
      expect(screen.getByTestId("editor-content-panel")).toBeInTheDocument()
    })
  })

  it("shows close button", () => {
    render(<EbookEditorShell book={mockBook} onClose={vi.fn()} />)
    expect(screen.getByTestId("editor-close-btn")).toBeInTheDocument()
  })

  it("calls onClose when close button clicked", () => {
    const onClose = vi.fn()
    render(<EbookEditorShell book={mockBook} onClose={onClose} />)
    screen.getByTestId("editor-close-btn").click()
    expect(onClose).toHaveBeenCalled()
  })

  it("shows save button", async () => {
    render(<EbookEditorShell book={mockBook} onClose={vi.fn()} />)
    await waitFor(() =>
      expect(screen.getByTestId("editor-save-btn")).toBeInTheDocument()
    )
  })
})
