import { render, screen } from "@testing-library/react"
import { describe, it, expect } from "vitest"
import { ReadingProgressBar } from "./ReadingProgressBar"

describe("ReadingProgressBar", () => {
  it("renders with 0% progress", () => {
    render(<ReadingProgressBar percent={0} />)
    const bar = screen.getByTestId("reading-progress-bar")
    expect(bar).toBeInTheDocument()
    expect(bar).toHaveAttribute("aria-valuenow", "0")
  })

  it("renders with 50% progress", () => {
    render(<ReadingProgressBar percent={50} />)
    const fill = screen.getByTestId("reading-progress-fill")
    expect(fill).toHaveStyle({ width: "50%" })
  })

  it("renders with 100% progress", () => {
    render(<ReadingProgressBar percent={100} />)
    const fill = screen.getByTestId("reading-progress-fill")
    expect(fill).toHaveStyle({ width: "100%" })
  })

  it("shows percentage label", () => {
    render(<ReadingProgressBar percent={75} showLabel />)
    expect(screen.getByText("75%")).toBeInTheDocument()
  })

  it("clamps values above 100", () => {
    render(<ReadingProgressBar percent={150} />)
    const fill = screen.getByTestId("reading-progress-fill")
    expect(fill).toHaveStyle({ width: "100%" })
  })

  it("clamps values below 0", () => {
    render(<ReadingProgressBar percent={-10} />)
    const fill = screen.getByTestId("reading-progress-fill")
    expect(fill).toHaveStyle({ width: "0%" })
  })
})
