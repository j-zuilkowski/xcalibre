import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { Sidebar } from "./Sidebar"
import { mockInvoke } from "../test/setup"

const mockVlibs = [
  { id: "vl1", name: "Science Fiction", search_expr: "tag:sci-fi", sort_field: "title", sort_asc: true },
  { id: "vl2", name: "Unread",          search_expr: "progress:0",  sort_field: "added_at", sort_asc: false },
]

beforeEach(() => {
  mockInvoke("list_virtual_libraries", mockVlibs)
  mockInvoke("list_libraries", [{ id: "lib1", name: "My Library", is_active: true }])
})

describe("Sidebar virtual libraries", () => {
  it("renders virtual library section header", async () => {
    render(<Sidebar onSelectFilter={vi.fn()} />)
    await waitFor(() =>
      expect(screen.getByTestId("vlib-section-header")).toBeInTheDocument()
    )
  })

  it("lists all virtual libraries", async () => {
    render(<Sidebar onSelectFilter={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByText("Science Fiction")).toBeInTheDocument()
      expect(screen.getByText("Unread")).toBeInTheDocument()
    })
  })

  it("calls onSelectFilter with search_expr when virtual library clicked", async () => {
    const onSelectFilter = vi.fn()
    render(<Sidebar onSelectFilter={onSelectFilter} />)
    await waitFor(() => screen.getByText("Science Fiction"))
    fireEvent.click(screen.getByText("Science Fiction"))
    expect(onSelectFilter).toHaveBeenCalledWith("tag:sci-fi")
  })

  it("shows add-virtual-library button", async () => {
    render(<Sidebar onSelectFilter={vi.fn()} />)
    await waitFor(() =>
      expect(screen.getByTestId("add-vlib-btn")).toBeInTheDocument()
    )
  })
})
