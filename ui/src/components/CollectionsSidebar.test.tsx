import { beforeEach, describe, expect, test, vi } from "vitest"
import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { CollectionsSidebar } from "./CollectionsSidebar"
import { useLibraryStore, type Book } from "../store/libraryStore"
import { invokeMock, mockInvoke } from "../test/setup"

const COLLECTION_A = { id: "c1", name: "Sci-Fi", created_at: "2024-01-01" }
const COLLECTION_B = { id: "c2", name: "Classics", created_at: "2024-01-02" }
const BOOK_A: Book = {
  id: "b1",
  title: "Dune",
  authors: ["Herbert"],
  format: "epub",
  cover_path: null,
  progress_percent: 0,
  last_opened_at: null,
  reading_cfi: null,
}
const BOOK_B: Book = {
  id: "b2",
  title: "Foundation",
  authors: ["Asimov"],
  format: "epub",
  cover_path: null,
  progress_percent: 0,
  last_opened_at: null,
  reading_cfi: null,
}

describe("CollectionsSidebar", () => {
  beforeEach(() => {
    useLibraryStore.setState({ books: [], loading: false, error: null })
  })

  test("renders All Books button", async () => {
    mockInvoke("list_collections", [])

    render(<CollectionsSidebar selectedIds={[]} />)

    expect(await screen.findByRole("button", { name: "All Books" })).toBeInTheDocument()
  })

  test("renders collection list", async () => {
    mockInvoke("list_collections", [COLLECTION_A, COLLECTION_B])

    render(<CollectionsSidebar selectedIds={[]} />)

    expect(await screen.findByRole("button", { name: COLLECTION_A.name })).toBeInTheDocument()
    expect(screen.getByRole("button", { name: COLLECTION_B.name })).toBeInTheDocument()
  })

  test("clicking All Books calls fetchBooks", async () => {
    const user = userEvent.setup()
    mockInvoke("list_collections", [])
    mockInvoke("list_books", [BOOK_A])

    render(<CollectionsSidebar selectedIds={[]} />)
    await screen.findByRole("button", { name: "All Books" })
    invokeMock.mockClear()

    await user.click(screen.getByRole("button", { name: "All Books" }))

    await waitFor(() => {
      expect(invokeMock.mock.calls.some(([cmd]) => cmd === "list_books")).toBe(true)
    })
  })

  test("clicking collection filters books", async () => {
    const user = userEvent.setup()
    mockInvoke("list_collections", [COLLECTION_A, COLLECTION_B])
    mockInvoke("get_books_in_collection", ["b1"])
    mockInvoke("list_books", [BOOK_A, BOOK_B])

    render(<CollectionsSidebar selectedIds={[]} />)

    await user.click(await screen.findByRole("button", { name: COLLECTION_A.name }))

    await waitFor(() => {
      expect(useLibraryStore.getState().books).toEqual([BOOK_A])
    })
  })

  test('create_collection called on "+" click', async () => {
    const user = userEvent.setup()
    mockInvoke("list_collections", [])

    render(<CollectionsSidebar selectedIds={[]} />)
    await screen.findByRole("button", { name: "All Books" })
    invokeMock.mockClear()

    await user.type(screen.getByPlaceholderText("New collection…"), "Sci-Fi")
    await user.click(screen.getByRole("button", { name: /^\+$/ }))

    await waitFor(() => {
      expect(
        invokeMock.mock.calls.some(
          ([cmd, payload]) => cmd === "create_collection" && payload && (payload as { name?: string }).name === "Sci-Fi",
        ),
      ).toBe(true)
    })
  })

  test('"+" with empty input does nothing', async () => {
    const user = userEvent.setup()
    mockInvoke("list_collections", [])

    render(<CollectionsSidebar selectedIds={[]} />)
    await screen.findByRole("button", { name: "All Books" })
    invokeMock.mockClear()

    await user.click(screen.getByRole("button", { name: /^\+$/ }))

    expect(invokeMock.mock.calls.some(([cmd]) => cmd === "create_collection")).toBe(false)
  })

  test('"Add selected to collection" disabled when no active collection', async () => {
    mockInvoke("list_collections", [])

    render(<CollectionsSidebar selectedIds={["1"]} />)

    const button = await screen.findByRole("button", { name: "+ Add selected to collection" })
    expect(button).toBeDisabled()
  })

  test("Enter key in input creates collection", async () => {
    const user = userEvent.setup()
    mockInvoke("list_collections", [])

    render(<CollectionsSidebar selectedIds={[]} />)
    await screen.findByRole("button", { name: "All Books" })
    invokeMock.mockClear()

    await user.type(screen.getByPlaceholderText("New collection…"), "Sci-Fi{enter}")

    await waitFor(() => {
      expect(
        invokeMock.mock.calls.some(
          ([cmd, payload]) => cmd === "create_collection" && payload && (payload as { name?: string }).name === "Sci-Fi",
        ),
      ).toBe(true)
    })
  })
})
