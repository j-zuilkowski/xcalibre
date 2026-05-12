import { render, screen, fireEvent } from "@testing-library/react"
import { describe, it, expect, vi } from "vitest"
import { SearchBar } from "./SearchBar"

describe("SearchBar", () => {
  it("renders an input", () => {
    render(<SearchBar onSearch={vi.fn()} />)
    expect(screen.getByRole("textbox")).toBeInTheDocument()
  })

  it("calls onSearch when Enter is pressed", () => {
    const onSearch = vi.fn()
    render(<SearchBar onSearch={onSearch} />)
    const input = screen.getByRole("textbox")
    fireEvent.change(input, { target: { value: "title:rust" } })
    fireEvent.keyDown(input, { key: "Enter" })
    expect(onSearch).toHaveBeenCalledWith("title:rust")
  })

  it("shows field hint autocomplete when ':' is typed", async () => {
    render(<SearchBar onSearch={vi.fn()} />)
    const input = screen.getByRole("textbox")
    fireEvent.change(input, { target: { value: "title:" } })
    // Hint list should appear
    expect(screen.getByTestId("search-hints")).toBeInTheDocument()
  })

  it("shows no hints for bare text", () => {
    render(<SearchBar onSearch={vi.fn()} />)
    const input = screen.getByRole("textbox")
    fireEvent.change(input, { target: { value: "rust" } })
    expect(screen.queryByTestId("search-hints")).not.toBeInTheDocument()
  })

  it("clears input when clear button is clicked", () => {
    const onSearch = vi.fn()
    render(<SearchBar onSearch={onSearch} />)
    const input = screen.getByRole("textbox")
    fireEvent.change(input, { target: { value: "title:rust" } })
    fireEvent.click(screen.getByTestId("search-clear"))
    expect(input).toHaveValue("")
    expect(onSearch).toHaveBeenCalledWith("")
  })
})
