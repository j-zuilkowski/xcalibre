import { render, screen, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { SimilarBooksPanel } from "./SimilarBooksPanel"
import { mockInvoke } from "../test/setup"

const mockBook = {
  id: "b1", title: "Dune", authors: ["Frank Herbert"],
  format: "EPUB", cover_path: null, progress_percent: 0, last_opened_at: null,
}

const mockSimilar = [
  { id: "b2", title: "Dune Messiah",  authors: "Frank Herbert", score: 3 },
  { id: "b3", title: "Foundation",    authors: "Isaac Asimov",  score: 1 },
]

beforeEach(() => {
  mockInvoke("find_similar_books_cmd", mockSimilar)
})

describe("SimilarBooksPanel", () => {
  it("renders section header", async () => {
    render(<SimilarBooksPanel book={mockBook} onOpenBook={vi.fn()} />)
    expect(screen.getByTestId("similar-books-header")).toBeInTheDocument()
  })

  it("lists similar books", async () => {
    render(<SimilarBooksPanel book={mockBook} onOpenBook={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByText("Dune Messiah")).toBeInTheDocument()
      expect(screen.getByText("Foundation")).toBeInTheDocument()
    })
  })

  it("shows empty state when no similar books", async () => {
    mockInvoke("find_similar_books_cmd", [])
    render(<SimilarBooksPanel book={mockBook} onOpenBook={vi.fn()} />)
    await waitFor(() =>
      expect(screen.getByTestId("similar-books-empty")).toBeInTheDocument()
    )
  })

  it("calls onOpenBook when a result is clicked", async () => {
    const onOpenBook = vi.fn()
    render(<SimilarBooksPanel book={mockBook} onOpenBook={onOpenBook} />)
    await waitFor(() => screen.getByText("Dune Messiah"))
    screen.getByText("Dune Messiah").click()
    expect(onOpenBook).toHaveBeenCalledWith("b2")
  })
})
