import { afterEach, beforeEach, describe, expect, test, vi } from "vitest"
import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { BulkActionBar } from "./BulkActionBar"
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

describe("BulkActionBar", () => {
  beforeEach(() => {
    useLibraryStore.setState({ books: [], loading: false, error: null })
    mockInvoke("list_books", [])
    mockInvoke("bulk_reingest_books", null)
    mockInvoke("bulk_delete_books", null)
    mockInvoke("bulk_export_metadata", "id,title\n1,x")
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  test("returns null when selectedIds is empty", () => {
    const { container } = render(
      <BulkActionBar
        selectedIds={[]}
        selectedBook={null}
        onDone={vi.fn()}
        onEditMetadata={vi.fn()}
        onRepair={vi.fn()}
        onConvert={vi.fn()}
      />,
    )

    expect(container).toBeEmptyDOMElement()
  })

  test("shows count", () => {
    render(
      <BulkActionBar
        selectedIds={["a", "b"]}
        selectedBook={BOOK_A}
        onDone={vi.fn()}
        onEditMetadata={vi.fn()}
        onRepair={vi.fn()}
        onConvert={vi.fn()}
      />,
    )

    expect(screen.getByText("2 selected")).toBeInTheDocument()
  })

  test("all 7 buttons rendered", () => {
    render(
      <BulkActionBar
        selectedIds={["a"]}
        selectedBook={BOOK_A}
        onDone={vi.fn()}
        onEditMetadata={vi.fn()}
        onRepair={vi.fn()}
        onConvert={vi.fn()}
      />,
    )

    expect(screen.getByRole("button", { name: "Edit metadata" })).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Tweak EPUB" })).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Repair" })).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Re-ingest" })).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Export CSV" })).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Delete" })).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Cancel" })).toBeInTheDocument()
  })

  test('single EPUB selection shows "Tweak EPUB"', () => {
    render(
      <BulkActionBar
        selectedIds={["a"]}
        selectedBook={BOOK_A}
        onDone={vi.fn()}
        onEditMetadata={vi.fn()}
        onRepair={vi.fn()}
        onConvert={vi.fn()}
      />,
    )

    expect(screen.getByRole("button", { name: "Tweak EPUB" })).toBeInTheDocument()
  })

  test('single non-EPUB shows "Convert to EPUB"', () => {
    render(
      <BulkActionBar
        selectedIds={["a"]}
        selectedBook={{ ...BOOK_A, format: "pdf" }}
        onDone={vi.fn()}
        onEditMetadata={vi.fn()}
        onRepair={vi.fn()}
        onConvert={vi.fn()}
      />,
    )

    expect(screen.getByRole("button", { name: "Convert to EPUB" })).toBeInTheDocument()
  })

  test("Cancel calls onDone", async () => {
    const user = userEvent.setup()
    const onDone = vi.fn()

    render(
      <BulkActionBar
        selectedIds={["a"]}
        selectedBook={BOOK_A}
        onDone={onDone}
        onEditMetadata={vi.fn()}
        onRepair={vi.fn()}
        onConvert={vi.fn()}
      />,
    )

    await user.click(screen.getByRole("button", { name: "Cancel" }))
    expect(onDone).toHaveBeenCalledTimes(1)
  })

  test("Edit metadata calls onEditMetadata", async () => {
    const user = userEvent.setup()
    const onEditMetadata = vi.fn()

    render(
      <BulkActionBar
        selectedIds={["a"]}
        selectedBook={BOOK_A}
        onDone={vi.fn()}
        onEditMetadata={onEditMetadata}
        onRepair={vi.fn()}
        onConvert={vi.fn()}
      />,
    )

    await user.click(screen.getByRole("button", { name: "Edit metadata" }))
    expect(onEditMetadata).toHaveBeenCalledTimes(1)
  })

  test("Repair calls onRepair", async () => {
    const user = userEvent.setup()
    const onRepair = vi.fn().mockResolvedValue(undefined)

    render(
      <BulkActionBar
        selectedIds={["a"]}
        selectedBook={BOOK_A}
        onDone={vi.fn()}
        onEditMetadata={vi.fn()}
        onRepair={onRepair}
        onConvert={vi.fn()}
      />,
    )

    await user.click(screen.getByRole("button", { name: "Repair" }))
    expect(onRepair).toHaveBeenCalledTimes(1)
  })

  test("Convert calls onConvert", async () => {
    const user = userEvent.setup()
    const onConvert = vi.fn().mockResolvedValue(undefined)

    render(
      <BulkActionBar
        selectedIds={["a"]}
        selectedBook={BOOK_A}
        onDone={vi.fn()}
        onEditMetadata={vi.fn()}
        onRepair={vi.fn()}
        onConvert={onConvert}
      />,
    )

    await user.click(screen.getByRole("button", { name: "Tweak EPUB" }))
    expect(onConvert).toHaveBeenCalledTimes(1)
  })

  test("Re-ingest calls invoke(bulk_reingest_books)", async () => {
    const user = userEvent.setup()
    const onDone = vi.fn()

    render(
      <BulkActionBar
        selectedIds={["a"]}
        selectedBook={BOOK_A}
        onDone={onDone}
        onEditMetadata={vi.fn()}
        onRepair={vi.fn()}
        onConvert={vi.fn()}
      />,
    )

    invokeMock.mockClear()
    await user.click(screen.getByRole("button", { name: "Re-ingest" }))

    await waitFor(() => {
      expect(
        invokeMock.mock.calls.some(([cmd, payload]) => {
          return cmd === "bulk_reingest_books" && Boolean(payload)
        }),
      ).toBe(true)
    })
    expect(onDone).toHaveBeenCalledTimes(1)
  })

  test("Delete prompts confirm then calls invoke", async () => {
    const user = userEvent.setup()
    vi.spyOn(window, "confirm").mockReturnValue(true)

    render(
      <BulkActionBar
        selectedIds={["a"]}
        selectedBook={BOOK_A}
        onDone={vi.fn()}
        onEditMetadata={vi.fn()}
        onRepair={vi.fn()}
        onConvert={vi.fn()}
      />,
    )

    invokeMock.mockClear()
    await user.click(screen.getByRole("button", { name: "Delete" }))

    await waitFor(() => {
      expect(
        invokeMock.mock.calls.some(([cmd, payload]) => {
          return cmd === "bulk_delete_books" && Boolean(payload)
        }),
      ).toBe(true)
    })
  })

  test("Delete does nothing if confirm returns false", async () => {
    const user = userEvent.setup()
    vi.spyOn(window, "confirm").mockReturnValue(false)

    render(
      <BulkActionBar
        selectedIds={["a"]}
        selectedBook={BOOK_A}
        onDone={vi.fn()}
        onEditMetadata={vi.fn()}
        onRepair={vi.fn()}
        onConvert={vi.fn()}
      />,
    )

    invokeMock.mockClear()
    await user.click(screen.getByRole("button", { name: "Delete" }))

    expect(invokeMock.mock.calls.some(([cmd]) => cmd === "bulk_delete_books")).toBe(false)
  })

  test("Export CSV triggers download", async () => {
    const user = userEvent.setup()
    const createObjectURL = vi.fn(() => "blob:mock")
    const revokeObjectURL = vi.fn()
    const click = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {})
    Object.defineProperty(URL, "createObjectURL", {
      value: createObjectURL,
      writable: true,
    })
    Object.defineProperty(URL, "revokeObjectURL", {
      value: revokeObjectURL,
      writable: true,
    })

    render(
      <BulkActionBar
        selectedIds={["a"]}
        selectedBook={BOOK_A}
        onDone={vi.fn()}
        onEditMetadata={vi.fn()}
        onRepair={vi.fn()}
        onConvert={vi.fn()}
      />,
    )

    invokeMock.mockClear()
    await user.click(screen.getByRole("button", { name: "Export CSV" }))

    await waitFor(() => {
      expect(createObjectURL).toHaveBeenCalledTimes(1)
    })
    expect(click).toHaveBeenCalledTimes(1)
    expect(revokeObjectURL).toHaveBeenCalledWith("blob:mock")
  })
})
