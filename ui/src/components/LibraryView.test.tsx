import { beforeEach, describe, expect, test, vi } from "vitest"
import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { LibraryView } from "./LibraryView"
import { useLibraryStore, type Book } from "../store/libraryStore"
import { invokeMock, mockInvoke } from "../test/setup"

function makeBook(overrides: Partial<Book> = {}): Book {
  return {
    id: "book-1",
    title: "Dune",
    authors: ["Frank Herbert"],
    format: "epub",
    cover_path: "/covers/dune.jpg",
    progress_percent: 50,
    last_opened_at: "2026-04-19T00:00:00Z",
    ...overrides,
  }
}

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((res) => {
    resolve = res
  })
  return { promise, resolve }
}

describe("LibraryView", () => {
  beforeEach(() => {
    useLibraryStore.setState({
      books: [],
      loading: false,
      error: null,
    })
    invokeMock.mockClear()
  })

  test("shows skeleton while loading", async () => {
    const pending = deferred<Book[]>()
    invokeMock.mockImplementationOnce(() => pending.promise)

    const { container } = render(
      <LibraryView onSelectBook={vi.fn()} selectedIds={[]} onToggleSelected={vi.fn()} />,
    )

    await waitFor(() => {
      expect(container.querySelectorAll(".animate-pulse").length).toBeGreaterThan(0)
    })

    pending.resolve([])
    await waitFor(() => {
      expect(useLibraryStore.getState().loading).toBe(false)
    })
  })

  test("shows empty state", async () => {
    mockInvoke("list_books", [])

    render(<LibraryView onSelectBook={vi.fn()} selectedIds={[]} onToggleSelected={vi.fn()} />)

    expect(await screen.findByText("No books yet. Drag a file to ingest.")).toBeInTheDocument()
  })

  test("shows error message", async () => {
    mockInvoke("list_books", new Error("db error"))

    render(<LibraryView onSelectBook={vi.fn()} selectedIds={[]} onToggleSelected={vi.fn()} />)

    expect(await screen.findByText(/db error/i)).toBeInTheDocument()
  })

  test("renders a card per book", async () => {
    const books = [
      makeBook({ id: "book-1", title: "Dune" }),
      makeBook({ id: "book-2", title: "Children of Dune" }),
      makeBook({ id: "book-3", title: "God Emperor of Dune" }),
    ]
    mockInvoke("list_books", books)

    render(<LibraryView onSelectBook={vi.fn()} selectedIds={[]} onToggleSelected={vi.fn()} />)

    expect(await screen.findByText("Dune")).toBeInTheDocument()
    expect(screen.getByText("Children of Dune")).toBeInTheDocument()
    expect(screen.getByText("God Emperor of Dune")).toBeInTheDocument()
    expect(screen.getAllByText("Frank Herbert")).toHaveLength(3)
  })

  test("book with cover renders img", async () => {
    mockInvoke("list_books", [makeBook({ cover_path: "/covers/x.jpg" })])

    render(<LibraryView onSelectBook={vi.fn()} selectedIds={[]} onToggleSelected={vi.fn()} />)

    const image = await screen.findByAltText("Dune")
    expect(image).toHaveAttribute("src", "asset:///covers/x.jpg")
  })

  test("book without cover renders format text", async () => {
    mockInvoke("list_books", [makeBook({ cover_path: null, format: "pdf" })])

    render(<LibraryView onSelectBook={vi.fn()} selectedIds={[]} onToggleSelected={vi.fn()} />)

    expect(await screen.findByText("pdf")).toBeInTheDocument()
  })

  test("book with progress renders progress bar", async () => {
    mockInvoke("list_books", [makeBook({ progress_percent: 67 })])

    const { container } = render(
      <LibraryView onSelectBook={vi.fn()} selectedIds={[]} onToggleSelected={vi.fn()} />,
    )

    await screen.findByText("Dune")
    const progress = container.querySelector('div[style*="67%"]')
    expect(progress).not.toBeNull()
  })

  test("clicking checkbox calls onToggleSelected", async () => {
    const user = userEvent.setup()
    const onToggleSelected = vi.fn()
    mockInvoke("list_books", [makeBook()])

    render(<LibraryView onSelectBook={vi.fn()} selectedIds={[]} onToggleSelected={onToggleSelected} />)

    await user.click(await screen.findByRole("checkbox"))
    expect(onToggleSelected).toHaveBeenCalledWith("book-1")
  })

  test("selected book has ring class", async () => {
    mockInvoke("list_books", [makeBook()])

    const { container } = render(
      <LibraryView onSelectBook={vi.fn()} selectedIds={["book-1"]} onToggleSelected={vi.fn()} />,
    )

    await screen.findByText("Dune")
    expect(container.querySelector(".ring-2")).toBeInTheDocument()
  })

  test("clicking card body calls onSelectBook", async () => {
    const user = userEvent.setup()
    const onSelectBook = vi.fn()
    mockInvoke("list_books", [makeBook()])

    render(<LibraryView onSelectBook={onSelectBook} selectedIds={[]} onToggleSelected={vi.fn()} />)

    await user.click(await screen.findByText("Dune"))
    expect(onSelectBook).toHaveBeenCalledWith(
      expect.objectContaining({ id: "book-1", title: "Dune" }),
    )
  })

  test("fetchBooks called on mount", async () => {
    mockInvoke("list_books", [makeBook()])

    render(<LibraryView onSelectBook={vi.fn()} selectedIds={[]} onToggleSelected={vi.fn()} />)

    await waitFor(() => {
      expect(invokeMock.mock.calls[0]?.[0]).toBe("list_books")
    })
  })
})
