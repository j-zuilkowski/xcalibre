import { invoke } from "@tauri-apps/api/core"
import type { Book } from "../store/libraryStore"
import { useLibraryStore } from "../store/libraryStore"

interface Props {
  selectedIds: string[]
  selectedBook: Book | null
  onDone: () => void
  onEditMetadata: () => void
  onRepair: () => Promise<void>
  onConvert: () => Promise<void>
}

export function BulkActionBar({ selectedIds, selectedBook, onDone, onEditMetadata, onRepair, onConvert }: Props) {
  const fetchBooks = useLibraryStore((s) => s.fetchBooks)

  if (selectedIds.length === 0) return null

  const reingest = async () => {
    await invoke("bulk_reingest_books", { bookIds: selectedIds })
    await fetchBooks()
    onDone()
  }

  const remove = async () => {
    if (!window.confirm(`Delete ${selectedIds.length} book(s)?`)) return
    await invoke("bulk_delete_books", { bookIds: selectedIds })
    await fetchBooks()
    onDone()
  }

  const exportMeta = async () => {
    const csv = await invoke<string>("bulk_export_metadata", { bookIds: selectedIds })
    const blob = new Blob([csv], { type: "text/csv" })
    const url = URL.createObjectURL(blob)
    const a = document.createElement("a")
    a.href = url
    a.download = "xcalibre_export.csv"
    a.click()
    URL.revokeObjectURL(url)
  }

  const tweakLabel =
    selectedIds.length === 1 && selectedBook?.format?.toUpperCase() === "EPUB"
      ? "Tweak EPUB"
      : "Convert to EPUB"

  return (
    <div className="fixed bottom-4 left-[calc(50%+6.5rem)] -translate-x-1/2 flex items-center gap-2
                    bg-white dark:bg-gray-800 shadow-xl rounded-xl px-5 py-3 border border-gray-200 dark:border-gray-700 z-40">
      <span className="text-sm font-medium dark:text-white">{selectedIds.length} selected</span>
      <button onClick={onEditMetadata} className="text-sm px-3 py-1.5 rounded bg-sky-500 text-white hover:bg-sky-600">
        Edit metadata
      </button>
      <button onClick={() => void onConvert()} className="text-sm px-3 py-1.5 rounded bg-cyan-600 text-white hover:bg-cyan-500">
        {tweakLabel}
      </button>
      <button onClick={() => void onRepair()} className="text-sm px-3 py-1.5 rounded bg-amber-500 text-white hover:bg-amber-600">
        Repair
      </button>
      <button onClick={reingest} className="text-sm px-3 py-1.5 rounded bg-blue-500 text-white hover:bg-blue-600">
        Re-ingest
      </button>
      <button onClick={exportMeta} className="text-sm px-3 py-1.5 rounded bg-green-500 text-white hover:bg-green-600">
        Export CSV
      </button>
      <button onClick={remove} className="text-sm px-3 py-1.5 rounded bg-red-500 text-white hover:bg-red-600">
        Delete
      </button>
      <button onClick={onDone} className="text-sm px-3 py-1.5 rounded bg-gray-200 dark:bg-gray-700 dark:text-white hover:bg-gray-300">
        Cancel
      </button>
    </div>
  )
}
