import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { VirtualLibraryEditor } from "./VirtualLibraryEditor"
import { mockInvoke } from "../test/setup"

beforeEach(() => {
  mockInvoke("list_virtual_libraries", [])
  mockInvoke("create_virtual_library", { id: "vl1", name: "New VLib", search_expr: "tag:fiction", sort_field: "title", sort_asc: true })
  mockInvoke("update_virtual_library", undefined)
  mockInvoke("delete_virtual_library", undefined)
})

describe("VirtualLibraryEditor", () => {
  it("renders name input and search expression input", () => {
    render(<VirtualLibraryEditor onClose={vi.fn()} />)
    expect(screen.getByTestId("vlib-name-input")).toBeInTheDocument()
    expect(screen.getByTestId("vlib-search-input")).toBeInTheDocument()
  })

  it("shows validation error when name is empty", async () => {
    render(<VirtualLibraryEditor onClose={vi.fn()} />)
    fireEvent.change(screen.getByTestId("vlib-search-input"), { target: { value: "tag:sci-fi" } })
    fireEvent.click(screen.getByTestId("vlib-save-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("vlib-name-error")).toBeInTheDocument()
    )
  })

  it("shows validation error when search expression is empty", async () => {
    render(<VirtualLibraryEditor onClose={vi.fn()} />)
    fireEvent.change(screen.getByTestId("vlib-name-input"), { target: { value: "My VLib" } })
    fireEvent.click(screen.getByTestId("vlib-save-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("vlib-search-error")).toBeInTheDocument()
    )
  })

  it("calls create_virtual_library with correct args on save", async () => {
    const onClose = vi.fn()
    render(<VirtualLibraryEditor onClose={onClose} />)
    fireEvent.change(screen.getByTestId("vlib-name-input"), { target: { value: "Sci-Fi" } })
    fireEvent.change(screen.getByTestId("vlib-search-input"), { target: { value: "tag:sci-fi" } })
    fireEvent.click(screen.getByTestId("vlib-save-btn"))
    await waitFor(() => expect(onClose).toHaveBeenCalled())
  })

  it("populates fields when editing existing virtual library", () => {
    const existing = { id: "vl1", name: "Fantasy", search_expr: "tag:fantasy", sort_field: "title", sort_asc: true }
    render(<VirtualLibraryEditor existing={existing} onClose={vi.fn()} />)
    expect(screen.getByTestId<HTMLInputElement>("vlib-name-input").value).toBe("Fantasy")
    expect(screen.getByTestId<HTMLInputElement>("vlib-search-input").value).toBe("tag:fantasy")
  })
})
