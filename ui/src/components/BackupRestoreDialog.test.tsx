import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { BackupRestoreDialog } from "./BackupRestoreDialog"
import { mockInvoke } from "../test/setup"

beforeEach(() => {
  mockInvoke("export_library_backup", "/tmp/backup.xcalibre")
  mockInvoke("restore_library_backup", { books_restored: 42 })
})

describe("BackupRestoreDialog", () => {
  it("shows export and restore tabs", () => {
    render(<BackupRestoreDialog onClose={vi.fn()} />)
    expect(screen.getByTestId("backup-tab")).toBeInTheDocument()
    expect(screen.getByTestId("restore-tab")).toBeInTheDocument()
  })

  it("shows export button in backup tab", () => {
    render(<BackupRestoreDialog onClose={vi.fn()} />)
    expect(screen.getByTestId("export-backup-btn")).toBeInTheDocument()
  })

  it("shows success message after export", async () => {
    render(<BackupRestoreDialog onClose={vi.fn()} />)
    fireEvent.click(screen.getByTestId("export-backup-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("backup-success")).toBeInTheDocument()
    )
  })

  it("shows restore button in restore tab", () => {
    render(<BackupRestoreDialog onClose={vi.fn()} />)
    fireEvent.click(screen.getByTestId("restore-tab"))
    expect(screen.getByTestId("restore-backup-btn")).toBeInTheDocument()
  })

  it("shows books-restored count after restore", async () => {
    render(<BackupRestoreDialog onClose={vi.fn()} />)
    fireEvent.click(screen.getByTestId("restore-tab"))
    fireEvent.click(screen.getByTestId("restore-backup-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("restore-success")).toHaveTextContent("42")
    )
  })
})
