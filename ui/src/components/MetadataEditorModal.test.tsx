import { beforeEach, describe, expect, test, vi } from "vitest"
import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { MetadataEditorModal } from "./MetadataEditorModal"
import { invokeMock, mockInvoke } from "../test/setup"

const BOOK_DETAIL = {
  id: "1",
  title: "Dune",
  authors: ["Frank Herbert"],
  format: "epub",
  pubdate: "1965-08-01",
  description: "A desert planet.",
  publisher: "Chilton",
  series_name: "Dune",
  series_index: 1,
  rating: 5,
  tags: ["sci-fi"],
  identifiers: { isbn: "0-441-17271-7" },
  cover_path: null,
  progress_percent: 0,
  last_opened_at: null,
  reading_cfi: null,
}

describe("MetadataEditorModal", () => {
  beforeEach(() => {
    invokeMock.mockClear()
  })

  test("renders nothing when open=false", () => {
    render(
      <MetadataEditorModal
        open={false}
        bookIds={["1"]}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    )

    expect(screen.queryByText("Metadata editor")).not.toBeInTheDocument()
  })

  test("loads book details on open", async () => {
    mockInvoke("get_book_details", BOOK_DETAIL)

    render(
      <MetadataEditorModal
        open={true}
        bookIds={["1"]}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    )

    expect(await screen.findByLabelText("Title")).toHaveValue(BOOK_DETAIL.title)
  })

  test("all fields rendered", async () => {
    mockInvoke("get_book_details", BOOK_DETAIL)

    render(
      <MetadataEditorModal
        open={true}
        bookIds={["1"]}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    )

    await screen.findByLabelText("Title")

    expect(screen.getByLabelText("Title")).toBeInTheDocument()
    expect(screen.getByLabelText("Authors")).toBeInTheDocument()
    expect(screen.getByLabelText("Pub date")).toBeInTheDocument()
    expect(screen.getByLabelText("Description")).toBeInTheDocument()
    expect(screen.getByLabelText("Publisher")).toBeInTheDocument()
    expect(screen.getByLabelText("Series")).toBeInTheDocument()
    expect(screen.getByLabelText("Series index")).toBeInTheDocument()
    expect(screen.getByLabelText("Rating")).toBeInTheDocument()
    expect(screen.getByLabelText("Tags")).toBeInTheDocument()
    expect(screen.getByLabelText("Identifiers")).toBeInTheDocument()
  })

  test("Save calls update_book_details", async () => {
    const user = userEvent.setup()
    mockInvoke("get_book_details", BOOK_DETAIL)
    mockInvoke("update_book_details", null)

    render(
      <MetadataEditorModal
        open={true}
        bookIds={["1"]}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    )

    const title = await screen.findByLabelText("Title")
    await user.clear(title)
    await user.type(title, "Dune Messiah")
    await user.click(screen.getByRole("button", { name: "Save changes" }))

    expect(
      invokeMock.mock.calls.some(
        ([cmd, payload]) =>
          cmd === "update_book_details" &&
          Boolean(payload) &&
          (payload as {
            bookIds?: string[]
            details?: { title?: string }
          }).bookIds?.[0] === "1" &&
          (payload as {
            details?: { title?: string }
          }).details?.title === "Dune Messiah",
      ),
    ).toBe(true)
  })

  test("Save calls onSaved", async () => {
    const user = userEvent.setup()
    const onSaved = vi.fn().mockResolvedValue(undefined)
    mockInvoke("get_book_details", BOOK_DETAIL)
    mockInvoke("update_book_details", null)

    render(
      <MetadataEditorModal
        open={true}
        bookIds={["1"]}
        onClose={vi.fn()}
        onSaved={onSaved}
      />,
    )

    await user.click(await screen.findByRole("button", { name: "Save changes" }))

    await waitFor(() => {
      expect(onSaved).toHaveBeenCalledTimes(1)
    })
  })

  test("Cancel calls onClose", async () => {
    const user = userEvent.setup()
    const onClose = vi.fn()
    mockInvoke("get_book_details", BOOK_DETAIL)

    render(
      <MetadataEditorModal
        open={true}
        bookIds={["1"]}
        onClose={onClose}
        onSaved={vi.fn()}
      />,
    )

    await user.click(await screen.findByRole("button", { name: "Cancel" }))

    expect(onClose).toHaveBeenCalledTimes(1)
  })

  test("loading state shown while fetching", async () => {
    let resolveDetails!: (value: typeof BOOK_DETAIL) => void
    const pending = new Promise<typeof BOOK_DETAIL>((resolve) => {
      resolveDetails = resolve
    })
    invokeMock.mockImplementationOnce(() => pending)

    render(
      <MetadataEditorModal
        open={true}
        bookIds={["1"]}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    )

    expect(await screen.findByText("Loading metadata…")).toBeInTheDocument()

    resolveDetails(BOOK_DETAIL)
    expect(await screen.findByLabelText("Title")).toHaveValue(BOOK_DETAIL.title)
  })

  test("error shown if get_book_details fails", async () => {
    mockInvoke("get_book_details", new Error("db error"))

    render(
      <MetadataEditorModal
        open={true}
        bookIds={["1"]}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    )

    expect(await screen.findByText(/db error/i)).toBeInTheDocument()
  })

  test("tags field accepts comma-separated input", async () => {
    const user = userEvent.setup()
    mockInvoke("get_book_details", BOOK_DETAIL)
    mockInvoke("update_book_details", null)

    render(
      <MetadataEditorModal
        open={true}
        bookIds={["1"]}
        onClose={vi.fn()}
        onSaved={vi.fn()}
      />,
    )

    const tags = await screen.findByLabelText("Tags")
    await user.clear(tags)
    await user.type(tags, "fiction, sci-fi")
    await user.click(screen.getByRole("button", { name: "Save changes" }))

    expect(
      invokeMock.mock.calls.some(
        ([cmd, payload]) =>
          cmd === "update_book_details" &&
          Boolean(payload) &&
          Array.isArray(
            (payload as {
              details?: { tags?: string[] }
            }).details?.tags,
          ) &&
          JSON.stringify(
            (payload as {
              details?: { tags?: string[] }
            }).details?.tags,
          ) === JSON.stringify(["fiction", "sci-fi"]),
      ),
    ).toBe(true)
  })
})
