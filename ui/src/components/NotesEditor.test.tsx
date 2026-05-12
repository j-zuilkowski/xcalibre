import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { NotesEditor } from "./NotesEditor"
import { mockInvoke } from "../test/setup"

const mockNote = {
  id: "n1", book_id: "b1", title: "My Note",
  body_html: "<p>Test content</p>", body_text: "Test content",
  created_at: "2026-01-01", updated_at: "2026-01-01",
}

beforeEach(() => {
  mockInvoke("update_note", undefined)
  mockInvoke("create_note", mockNote)
})

describe("NotesEditor", () => {
  it("renders title input and editor area", () => {
    render(<NotesEditor bookId="b1" note={mockNote} onSave={vi.fn()} onDelete={vi.fn()} />)
    expect(screen.getByTestId("note-title-input")).toBeInTheDocument()
    expect(screen.getByTestId("note-editor-area")).toBeInTheDocument()
  })

  it("populates title from note prop", () => {
    render(<NotesEditor bookId="b1" note={mockNote} onSave={vi.fn()} onDelete={vi.fn()} />)
    const input = screen.getByTestId<HTMLInputElement>("note-title-input")
    expect(input.value).toBe("My Note")
  })

  it("shows save button and calls update_note on save", async () => {
    const onSave = vi.fn()
    render(<NotesEditor bookId="b1" note={mockNote} onSave={onSave} onDelete={vi.fn()} />)
    fireEvent.click(screen.getByTestId("note-save-btn"))
    await waitFor(() => expect(onSave).toHaveBeenCalled())
  })

  it("shows delete button", () => {
    render(<NotesEditor bookId="b1" note={mockNote} onSave={vi.fn()} onDelete={vi.fn()} />)
    expect(screen.getByTestId("note-delete-btn")).toBeInTheDocument()
  })

  it("renders in new-note mode when note is null", () => {
    render(<NotesEditor bookId="b1" note={null} onSave={vi.fn()} onDelete={vi.fn()} />)
    const input = screen.getByTestId<HTMLInputElement>("note-title-input")
    expect(input.value).toBe("")
  })
})
