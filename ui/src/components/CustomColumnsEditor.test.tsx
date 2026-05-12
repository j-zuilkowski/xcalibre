import { render, screen, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { CustomColumnsEditor } from "./CustomColumnsEditor"
import { mockInvoke } from "../test/setup"

const mockColumns = [
  { id: "c1", name: "rating", label: "Rating", col_type: "integer", is_multiple: false, display_in_grid: true },
  { id: "c2", name: "status", label: "Status", col_type: "text",    is_multiple: false, display_in_grid: true },
]

beforeEach(() => {
  mockInvoke("list_custom_columns_cmd", mockColumns)
  mockInvoke("create_custom_column_cmd", { id: "c3", name: "new_col", label: "New Col", col_type: "text", is_multiple: false, display_in_grid: true })
  mockInvoke("update_custom_column_cmd", undefined)
  mockInvoke("delete_custom_column_cmd", undefined)
})

describe("CustomColumnsEditor", () => {
  it("lists existing custom columns", async () => {
    render(<CustomColumnsEditor onClose={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByText("Rating")).toBeInTheDocument()
      expect(screen.getByText("Status")).toBeInTheDocument()
    })
  })

  it("shows add-column button", async () => {
    render(<CustomColumnsEditor onClose={vi.fn()} />)
    expect(screen.getByTestId("add-column-btn")).toBeInTheDocument()
  })

  it("shows delete button for each column", async () => {
    render(<CustomColumnsEditor onClose={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getAllByTestId(/delete-column-/)).toHaveLength(2)
    })
  })

  it("shows column type for each column", async () => {
    render(<CustomColumnsEditor onClose={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByText("integer")).toBeInTheDocument()
    })
  })
})
