import { render, screen, fireEvent } from "@testing-library/react"
import { describe, it, expect, vi } from "vitest"
import { ReasoningBudgetSelector } from "./ReasoningBudgetSelector"

describe("ReasoningBudgetSelector", () => {
  it("renders selector with all options", () => {
    render(<ReasoningBudgetSelector value="auto" onChange={vi.fn()} />)
    const select = screen.getByTestId("reasoning-budget-select")
    expect(select).toBeInTheDocument()
    expect(screen.getByText("Auto")).toBeInTheDocument()
    expect(screen.getByText("None")).toBeInTheDocument()
    expect(screen.getByText("Low")).toBeInTheDocument()
    expect(screen.getByText("Medium")).toBeInTheDocument()
    expect(screen.getByText("High")).toBeInTheDocument()
  })

  it("shows current value as selected", () => {
    render(<ReasoningBudgetSelector value="high" onChange={vi.fn()} />)
    const select = screen.getByTestId<HTMLSelectElement>("reasoning-budget-select")
    expect(select.value).toBe("high")
  })

  it("calls onChange when selection changes", () => {
    const onChange = vi.fn()
    render(<ReasoningBudgetSelector value="auto" onChange={onChange} />)
    fireEvent.change(screen.getByTestId("reasoning-budget-select"), { target: { value: "medium" } })
    expect(onChange).toHaveBeenCalledWith("medium")
  })
})
