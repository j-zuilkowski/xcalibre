import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { BackupRestoreDialog } from "./BackupRestoreDialog"
import { mockInvoke } from "../test/setup"

const pickExport  = vi.fn(() => Promise.resolve("/tmp/backup.xcalibre"))
const pickRestore = vi.fn(() => Promise.resolve("/tmp/backup.xcalibre"))

beforeEach(() => {
  mockInvoke("export_library_backup_cmd", "/tmp/backup.xcalibre")
  mockInvoke("restore_library_backup_cmd", { books_restored: 42 })
  pickExport.mockResolvedValue("/tmp/backup.xcalibre")
  pickRestore.mockResolvedValue("/tmp/backup.xcalibre")
})

const props = () => ({
  onClose: vi.fn(),
  onPickExportPath:  pickExport,
  onPickRestorePath: pickRestore,
})

describe("BackupRestoreDialog", () => {
  it("shows export and restore tabs", () => {
    render(<BackupRestoreDialog {...props()} />)
    expect(screen.getByTestId("backup-tab")).toBeInTheDocument()
    expect(screen.getByTestId("restore-tab")).toBeInTheDocument()
  })

  it("shows export button in backup tab", () => {
    render(<BackupRestoreDialog {...props()} />)
    expect(screen.getByTestId("export-backup-btn")).toBeInTheDocument()
  })

  it("shows success message after export", async () => {
    render(<BackupRestoreDialog {...props()} />)
    fireEvent.click(screen.getByTestId("export-backup-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("backup-success")).toBeInTheDocument()
    )
  })

  it("shows restore button in restore tab", () => {
    render(<BackupRestoreDialog {...props()} />)
    fireEvent.click(screen.getByTestId("restore-tab"))
    expect(screen.getByTestId("restore-backup-btn")).toBeInTheDocument()
  })

  it("shows books-restored count after restore", async () => {
    render(<BackupRestoreDialog {...props()} />)
    fireEvent.click(screen.getByTestId("restore-tab"))
    fireEvent.click(screen.getByTestId("restore-backup-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("restore-success")).toHaveTextContent("42")
    )
  })
})
