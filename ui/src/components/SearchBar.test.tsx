import { beforeEach, afterEach, describe, expect, test, vi } from "vitest"
import { act, fireEvent, render, screen } from "@testing-library/react"
import { SearchBar } from "./SearchBar"
import { useLibraryStore, type Book } from "../store/libraryStore"
import { invokeMock, mockInvoke } from "../test/setup"

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

describe("SearchBar", () => {
  beforeEach(() => {
    vi.useFakeTimers()
    useLibraryStore.setState({ books: [], loading: false, error: null })
    mockInvoke("list_books", [])
    mockInvoke("search_books", [])
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  test("typing fewer than 2 chars does not search", async () => {
    render(<SearchBar />)

    fireEvent.change(screen.getByPlaceholderText("Search title, author, text…"), {
      target: { value: "a" },
    })
    await act(async () => {
      await vi.advanceTimersByTimeAsync(200)
    })

    expect(invokeMock.mock.calls.some(([cmd]) => cmd === "search_books")).toBe(false)
  })

  test("typing 2+ chars calls search_books", async () => {
    mockInvoke("search_books", [BOOK_A])
    render(<SearchBar />)

    fireEvent.change(screen.getByPlaceholderText("Search title, author, text…"), {
      target: { value: "du" },
    })
    await act(async () => {
      await vi.advanceTimersByTimeAsync(200)
    })

    expect(
      invokeMock.mock.calls.some(
        ([cmd, payload]) =>
          cmd === "search_books" &&
          Boolean(payload) &&
          (payload as { query?: string }).query === "du",
      ),
    ).toBe(true)
  })

  test("clearing to <2 chars calls list_books", async () => {
    mockInvoke("search_books", [BOOK_A])
    render(<SearchBar />)

    const input = screen.getByPlaceholderText("Search title, author, text…")
    fireEvent.change(input, { target: { value: "dune" } })
    await act(async () => {
      await vi.advanceTimersByTimeAsync(200)
    })

    invokeMock.mockClear()

    fireEvent.change(input, { target: { value: "" } })
    await act(async () => {
      await vi.advanceTimersByTimeAsync(200)
    })

    expect(invokeMock.mock.calls.some(([cmd]) => cmd === "list_books")).toBe(true)
  })

  test("results replace books in store", async () => {
    mockInvoke("search_books", [BOOK_A])
    render(<SearchBar />)

    fireEvent.change(screen.getByPlaceholderText("Search title, author, text…"), {
      target: { value: "du" },
    })
    await act(async () => {
      await vi.advanceTimersByTimeAsync(200)
    })

    expect(useLibraryStore.getState().books).toEqual([BOOK_A])
  })

  test("search error is swallowed", async () => {
    useLibraryStore.setState({ books: [BOOK_A], loading: false, error: null })
    mockInvoke("search_books", new Error("failed"))
    render(<SearchBar />)

    fireEvent.change(screen.getByPlaceholderText("Search title, author, text…"), {
      target: { value: "du" },
    })
    await act(async () => {
      await vi.advanceTimersByTimeAsync(200)
    })

    expect(useLibraryStore.getState().books).toEqual([BOOK_A])
  })
})
