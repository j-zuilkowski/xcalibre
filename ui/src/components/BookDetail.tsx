import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Book } from "../store/libraryStore"
import { ReadingProgressBar } from "./ReadingProgressBar"

interface CustomColumn {
  id: string; name: string; label: string; col_type: string
}

interface Props {
  book: Book
  onClose: () => void
  onDiscuss?: () => void
}

export function BookDetail({ book, onClose, onDiscuss }: Props) {
  const [customCols,   setCustomCols]   = useState<CustomColumn[]>([])
  const [customValues, setCustomValues] = useState<Record<string, string | null>>({})
  const [readingStats, setReadingStats] = useState<{ total_minutes: number; session_count: number; page_count: number | null } | null>(null)

  useEffect(() => {
    invoke<CustomColumn[]>("list_custom_columns_cmd", { libraryId: null })
      .then(setCustomCols)
      .catch(() => {})
    invoke<Record<string, string | null>>("get_book_custom_values_cmd", { bookId: book.id })
      .then(setCustomValues)
      .catch(() => {})
    invoke<{ total_minutes: number; session_count: number; page_count: number | null }>(
      "get_reading_stats_cmd", { bookId: book.id }
    ).then(setReadingStats).catch(() => {})
  }, [book.id])

  function handleCustomChange(colId: string, value: string) {
    setCustomValues(prev => ({ ...prev, [colId]: value }))
    invoke("set_book_custom_value_cmd", { bookId: book.id, columnId: colId, value })
      .catch(() => {})
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-md p-6">
        <button
          onClick={onClose}
          className="float-right text-gray-400 hover:text-gray-600 text-xl leading-none"
        >
          ×
        </button>
        <h2 className="text-lg font-semibold dark:text-white mb-1">{book.title}</h2>
        <p className="text-sm text-gray-500 mb-4">{book.authors.join(", ")}</p>
        <p className="text-xs text-gray-400 uppercase tracking-wide mb-1">Format</p>
        <p className="text-sm dark:text-gray-300 mb-4">{book.format}</p>
        {book.progress_percent > 0 && (
          <>
            <p className="text-xs text-gray-400 uppercase tracking-wide mb-1">Progress</p>
            <div style={{ marginBottom: "0.5rem" }}>
              <ReadingProgressBar percent={book.progress_percent} showLabel />
            </div>
          </>
        )}
        {readingStats && (readingStats.session_count > 0 || readingStats.page_count != null) && (
          <div style={{ marginTop: "0.75rem" }}>
            <p className="text-xs text-gray-400 uppercase tracking-wide mb-1">Reading History</p>
            {readingStats.page_count != null && (
              <p className="text-sm dark:text-gray-300">{readingStats.page_count} pages</p>
            )}
            {readingStats.session_count > 0 && (
              <p className="text-sm dark:text-gray-300">
                {readingStats.session_count} session{readingStats.session_count !== 1 ? "s" : ""} · {Math.round(readingStats.total_minutes)} min total
              </p>
            )}
          </div>
        )}
        {customCols.length > 0 && (
          <div style={{ marginTop: "1rem" }}>
            <p className="text-xs text-gray-400 uppercase tracking-wide mb-2">Custom</p>
            {customCols.map(col => (
              <div key={col.id} style={{ marginBottom: "0.5rem" }}>
                <label
                  style={{ display: "block", fontSize: "0.8rem",
                           color: "var(--text-muted, #6c7086)", marginBottom: "0.2rem" }}
                >
                  {col.label}
                </label>
                <input
                  value={customValues[col.id] ?? ""}
                  onChange={e => handleCustomChange(col.id, e.target.value)}
                  style={{
                    width: "100%", padding: "0.3rem 0.5rem",
                    background: "var(--bg-overlay, #313244)",
                    border: "1px solid var(--border, #45475a)",
                    borderRadius: "4px", color: "inherit", fontSize: "0.9rem",
                  }}
                />
              </div>
            ))}
          </div>
        )}
        {onDiscuss && (
          <button
            data-testid="discuss-book-btn"
            onClick={onDiscuss}
            className="mt-4 w-full py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg text-sm font-medium"
          >
            Discuss with AI
          </button>
        )}
      </div>
    </div>
  )
}
