import { render, screen, fireEvent } from "@testing-library/react"
import { describe, it, expect, vi } from "vitest"
import { FileTreePanel } from "./FileTreePanel"

const mockItems = [
  { href: "OEBPS/ch01.xhtml", media_type: "application/xhtml+xml" },
  { href: "OEBPS/ch02.xhtml", media_type: "application/xhtml+xml" },
  { href: "OEBPS/style.css",  media_type: "text/css" },
  { href: "OEBPS/cover.jpg",  media_type: "image/jpeg" },
]

describe("FileTreePanel", () => {
  it("renders all manifest items", () => {
    render(<FileTreePanel items={mockItems} onSelectItem={vi.fn()} selectedHref={null} />)
    expect(screen.getByText("ch01.xhtml")).toBeInTheDocument()
    expect(screen.getByText("ch02.xhtml")).toBeInTheDocument()
    expect(screen.getByText("style.css")).toBeInTheDocument()
    expect(screen.getByText("cover.jpg")).toBeInTheDocument()
  })

  it("calls onSelectItem when file clicked", () => {
    const onSelectItem = vi.fn()
    render(<FileTreePanel items={mockItems} onSelectItem={onSelectItem} selectedHref={null} />)
    fireEvent.click(screen.getByText("ch01.xhtml"))
    expect(onSelectItem).toHaveBeenCalledWith(mockItems[0])
  })

  it("highlights selected item", () => {
    render(<FileTreePanel items={mockItems} onSelectItem={vi.fn()} selectedHref="OEBPS/ch01.xhtml" />)
    const item = screen.getByTestId("file-item-OEBPS/ch01.xhtml")
    expect(item).toHaveAttribute("aria-selected", "true")
  })

  it("groups items by type", () => {
    render(<FileTreePanel items={mockItems} onSelectItem={vi.fn()} selectedHref={null} />)
    expect(screen.getByTestId("file-group-html")).toBeInTheDocument()
    expect(screen.getByTestId("file-group-css")).toBeInTheDocument()
    expect(screen.getByTestId("file-group-images")).toBeInTheDocument()
  })
})
