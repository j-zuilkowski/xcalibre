import { render, screen } from "@testing-library/react"
import { describe, it, expect } from "vitest"
import { CitedResponseView } from "./CitedResponseView"

const mockCited = {
  response_text: "The spice melange enables space travel.",
  citations: [
    { chunk_index: 0, chunk_text: "Spice is essential for navigation.", relevance_score: 0.9 },
    { chunk_index: 1, chunk_text: "Navigators use the spice to fold space.", relevance_score: 0.7 },
  ],
}

describe("CitedResponseView", () => {
  it("renders response text", () => {
    render(<CitedResponseView cited={mockCited} />)
    expect(screen.getByText(/spice melange enables/i)).toBeInTheDocument()
  })

  it("renders citation count badge", () => {
    render(<CitedResponseView cited={mockCited} />)
    expect(screen.getByTestId("citation-count")).toBeInTheDocument()
    expect(screen.getByTestId("citation-count")).toHaveTextContent("2")
  })

  it("renders citation snippets", () => {
    render(<CitedResponseView cited={mockCited} />)
    expect(screen.getByText(/Spice is essential/)).toBeInTheDocument()
    expect(screen.getByText(/fold space/)).toBeInTheDocument()
  })

  it("renders without citations when citations array is empty", () => {
    const noCitations = { response_text: "No context.", citations: [] }
    render(<CitedResponseView cited={noCitations} />)
    expect(screen.queryByTestId("citation-count")).not.toBeInTheDocument()
  })
})
