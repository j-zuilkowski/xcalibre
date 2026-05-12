import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { SpellCheckInput } from "./SpellCheckInput"
import { mockInvoke } from "../test/setup"

beforeEach(() => {
  mockInvoke("check_word", { word: "colour", is_correct: false })
  mockInvoke("get_suggestions", { word: "colour", suggestions: ["color", "coulor"] })
  mockInvoke("add_to_dictionary", undefined)
})

describe("SpellCheckInput", () => {
  it("renders a textarea", () => {
    render(<SpellCheckInput value="" onChange={vi.fn()} />)
    expect(screen.getByRole("textbox")).toBeInTheDocument()
  })

  it("shows suggestion popover for misspelled word on right-click", async () => {
    render(<SpellCheckInput value="colour" onChange={vi.fn()} />)
    const textarea = screen.getByRole("textbox")
    fireEvent.contextMenu(textarea)
    await waitFor(() =>
      expect(screen.getByTestId("spell-suggestions")).toBeInTheDocument()
    )
    expect(screen.getByText("color")).toBeInTheDocument()
  })

  it("closes popover when suggestion is clicked", async () => {
    const onChange = vi.fn()
    render(<SpellCheckInput value="colour text" onChange={onChange} />)
    fireEvent.contextMenu(screen.getByRole("textbox"))
    await waitFor(() => screen.getByTestId("spell-suggestions"))
    fireEvent.click(screen.getByText("color"))
    expect(screen.queryByTestId("spell-suggestions")).not.toBeInTheDocument()
  })
})
