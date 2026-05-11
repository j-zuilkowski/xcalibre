import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { CreateLibraryModal } from "./CreateLibraryModal"
import { mockInvoke } from "../test/setup"

beforeEach(() => {
  mockInvoke("get_app_data_dir", "/tmp/test_app_data")
  mockInvoke("create_library_cmd", "new-lib-id")
  mockInvoke("set_active_library_cmd", undefined)
})

describe("CreateLibraryModal", () => {
  it("shows validation error when name is empty", async () => {
    render(<CreateLibraryModal onCreated={vi.fn()} />)
    fireEvent.click(screen.getByTestId("confirm-create-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("create-error")).toHaveTextContent("Name is required")
    )
  })

  it("calls onCreated with id after successful creation", async () => {
    const onCreated = vi.fn()
    render(<CreateLibraryModal onCreated={onCreated} />)
    fireEvent.change(screen.getByTestId("library-name-input"), { target: { value: "My Library" } })
    fireEvent.click(screen.getByTestId("confirm-create-btn"))
    await waitFor(() => expect(onCreated).toHaveBeenCalledWith("new-lib-id"))
  })

  it("defaults to in_place layout", () => {
    render(<CreateLibraryModal onCreated={vi.fn()} />)
    expect(screen.getByTestId("layout-select")).toHaveValue("in_place")
  })
})
