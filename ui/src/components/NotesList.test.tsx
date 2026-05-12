import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { NotesList } from "./NotesList"
import { mockInvoke } from "../test/setup"

const mockNotes = [
  { id: "n1", book_id: "b1", title: "First Note",  body_html: "<p>Content A</p>", body_text: "Content A", created_at: "2026-01-01", updated_at: "2026-01-01" },
  { id: "n2", book_id: "b1", title: "Second Note", body_html: "<p>Content B</p>", body_text: "Content B", created_at: "2026-01-02", updated_at: "2026-01-02" },
]

beforeEach(() => {
  mockInvoke("list_notes", mockNotes)
  mockInvoke("delete_note", undefined)
  mockInvoke("search_notes", [mockNotes[0]])
})

describe("NotesList", () => {
  it("renders all notes for a book", async () => {
    render(<NotesList bookId="b1" onSelectNote={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByText("First Note")).toBeInTheDocument()
      expect(screen.getByText("Second Note")).toBeInTheDocument()
    })
  })

  it("calls onSelectNote when note clicked", async () => {
    const onSelectNote = vi.fn()
    render(<NotesList bookId="b1" onSelectNote={onSelectNote} />)
    await waitFor(() => screen.getByText("First Note"))
    fireEvent.click(screen.getByText("First Note"))
    expect(onSelectNote).toHaveBeenCalledWith(mockNotes[0])
  })

  it("shows add-note button", async () => {
    render(<NotesList bookId="b1" onSelectNote={vi.fn()} />)
    expect(screen.getByTestId("add-note-btn")).toBeInTheDocument()
  })

  it("filters notes via search", async () => {
    render(<NotesList bookId="b1" onSelectNote={vi.fn()} />)
    await waitFor(() => screen.getByTestId("notes-search-input"))
    fireEvent.change(screen.getByTestId("notes-search-input"), { target: { value: "Content A" } })
    await waitFor(() => {
      expect(screen.getByText("First Note")).toBeInTheDocument()
      expect(screen.queryByText("Second Note")).not.toBeInTheDocument()
    })
  })
})
