import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { LibrarySwitcher, Library } from "./LibrarySwitcher"
import { mockInvoke } from "../test/setup"

const mockLibraries: Library[] = [
  { id: "a", name: "Main", db_path: "/tmp/a.db", layout: "in_place", is_active: true },
  { id: "b", name: "Research", db_path: "/tmp/b.db", layout: "managed", is_active: false },
]

beforeEach(() => {
  mockInvoke("list_libraries_cmd", mockLibraries)
})

describe("LibrarySwitcher", () => {
  it("renders library names after load", async () => {
    render(<LibrarySwitcher onSwitch={vi.fn()} onCreateNew={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByTestId("library-item-a")).toHaveTextContent("Main")
      expect(screen.getByTestId("library-item-b")).toHaveTextContent("Research")
    })
  })

  it("shows loading state initially", () => {
    mockInvoke("list_libraries_cmd", new Promise(() => {}))
    render(<LibrarySwitcher onSwitch={vi.fn()} onCreateNew={vi.fn()} />)
    expect(screen.getByTestId("switcher-loading")).toBeInTheDocument()
  })

  it("calls onSwitch when a library is clicked", async () => {
    const onSwitch = vi.fn()
    render(<LibrarySwitcher onSwitch={onSwitch} onCreateNew={vi.fn()} />)
    await waitFor(() => screen.getByTestId("library-item-b"))
    fireEvent.click(screen.getByTestId("library-item-b"))
    expect(onSwitch).toHaveBeenCalledWith(mockLibraries[1])
  })

  it("calls onCreateNew when + New Library is clicked", async () => {
    const onCreate = vi.fn()
    render(<LibrarySwitcher onSwitch={vi.fn()} onCreateNew={onCreate} />)
    await waitFor(() => screen.getByTestId("create-library-btn"))
    fireEvent.click(screen.getByTestId("create-library-btn"))
    expect(onCreate).toHaveBeenCalled()
  })
})
