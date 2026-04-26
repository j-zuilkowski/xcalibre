import { beforeEach, describe, expect, test } from "vitest"
import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { FilterBar } from "./FilterBar"
import { useLibraryStore } from "../store/libraryStore"
import { invokeMock, mockInvoke } from "../test/setup"

describe("FilterBar", () => {
  beforeEach(() => {
    useLibraryStore.setState({ books: [], loading: false, error: null })
    mockInvoke("list_authors", ["Orwell"])
    mockInvoke("list_series", ["Foundation"])
    mockInvoke("list_tags", ["sci-fi"])
    mockInvoke("filter_books", [])
    mockInvoke("list_books", [])
  })

  test("all dropdowns render", async () => {
    render(<FilterBar />)

    expect(await screen.findByText("All formats")).toBeInTheDocument()
    expect(screen.getAllByRole("combobox")).toHaveLength(4)
    expect(screen.getByPlaceholderText("Author…")).toBeInTheDocument()
  })

  test('format change triggers filter_books', async () => {
    const user = userEvent.setup()
    render(<FilterBar />)

    await user.selectOptions(screen.getAllByRole("combobox")[0], "EPUB")

    await waitFor(() => {
      expect(
        invokeMock.mock.calls.some(
          ([cmd, payload]) =>
            cmd === "filter_books" &&
            Boolean(payload) &&
            (payload as { format?: string | null }).format === "EPUB",
        ),
      ).toBe(true)
    })
  })

  test('author input triggers filter_books', async () => {
    const user = userEvent.setup()
    render(<FilterBar />)

    await user.type(screen.getByPlaceholderText("Author…"), "Orwell")

    await waitFor(() => {
      expect(
        invokeMock.mock.calls.some(
          ([cmd, payload]) =>
            cmd === "filter_books" &&
            Boolean(payload) &&
            (payload as { author?: string | null }).author === "Orwell",
        ),
      ).toBe(true)
    })
  })

  test('Clear button resets all filters and calls fetchBooks', async () => {
    const user = userEvent.setup()
    render(<FilterBar />)

    await user.selectOptions(screen.getAllByRole("combobox")[0], "EPUB")
    await user.type(screen.getByPlaceholderText("Author…"), "Orwell")
    invokeMock.mockClear()

    await user.click(screen.getByRole("button", { name: "Clear" }))

    await waitFor(() => {
      expect(invokeMock.mock.calls.some(([cmd]) => cmd === "list_books")).toBe(true)
    })

    expect(screen.getAllByRole("combobox")[0]).toHaveValue("")
    expect(screen.getAllByRole("combobox")[1]).toHaveValue("")
    expect(screen.getAllByRole("combobox")[2]).toHaveValue("")
    expect(screen.getAllByRole("combobox")[3]).toHaveValue("")
    expect(screen.getByPlaceholderText("Author…")).toHaveValue("")
  })

  test("authors are fetched from list_authors", async () => {
    render(<FilterBar />)

    await waitFor(() => {
      expect(
        invokeMock.mock.calls.some(([cmd]) => cmd === "list_authors"),
      ).toBe(true)
    })
  })
})
