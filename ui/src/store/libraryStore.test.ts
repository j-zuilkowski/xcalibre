import { beforeEach, describe, expect, test } from "vitest"
import { useLibraryStore, type Book } from "./libraryStore"
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

describe("useLibraryStore", () => {
  beforeEach(() => {
    useLibraryStore.setState({
      books: [],
      loading: false,
      error: null,
    })
    invokeMock.mockClear()
  })

  test("fetchBooks: populates books on success", async () => {
    const books = [makeBook(), makeBook({ id: "book-2", title: "Dune Messiah" })]
    mockInvoke("list_books", books)

    await useLibraryStore.getState().fetchBooks()

    expect(useLibraryStore.getState().books).toEqual(books)
    expect(useLibraryStore.getState().loading).toBe(false)
    expect(useLibraryStore.getState().error).toBeNull()
  })

  test("fetchBooks: sets error on invoke failure", async () => {
    mockInvoke("list_books", new Error("db error"))

    await useLibraryStore.getState().fetchBooks()

    expect(useLibraryStore.getState().books).toEqual([])
    expect(useLibraryStore.getState().error).toContain("db error")
    expect(useLibraryStore.getState().loading).toBe(false)
  })

  test("fetchBooks: sets loading true then false", async () => {
    const deferredResponse = deferred<Book[]>()
    invokeMock.mockImplementationOnce(() => deferredResponse.promise)

    const pending = useLibraryStore.getState().fetchBooks()

    expect(useLibraryStore.getState().loading).toBe(true)

    deferredResponse.resolve([makeBook()])
    await pending

    expect(useLibraryStore.getState().loading).toBe(false)
    expect(useLibraryStore.getState().books).toHaveLength(1)
  })

  test("setBooks: replaces books array", () => {
    useLibraryStore.getState().setBooks([makeBook({ id: "book-1" })])
    useLibraryStore.getState().setBooks([makeBook({ id: "book-2", title: "Children of Dune" })])

    expect(useLibraryStore.getState().books).toEqual([
      makeBook({ id: "book-2", title: "Children of Dune" }),
    ])
  })
})
