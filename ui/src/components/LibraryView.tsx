import { useEffect, useState } from "react"
import { useLibraryStore } from "../store/libraryStore"
import type { Book } from "../store/libraryStore"
import { LibrarySkeleton } from "./Skeleton"
import { SearchBar } from "./SearchBar"
import { invoke } from "@tauri-apps/api/core"

interface Props {
  onSelectBook?: (book: Book) => void
  selectedIds: string[]
  onToggleSelected: (bookId: string) => void
}

export function LibraryView({ onSelectBook, selectedIds, onToggleSelected }: Props) {
  const { books, loading, error, fetchBooks } = useLibraryStore()
  const [searchIds, setSearchIds] = useState<string[] | null>(null)

  useEffect(() => {
    fetchBooks()
  }, [fetchBooks])

  const handleSearch = async (query: string) => {
    if (!query) { setSearchIds(null); return }
    try {
      const ids = await invoke<string[]>("search_library_advanced", { query })
      setSearchIds(ids)
    } catch { setSearchIds([]) }
  }

  // Filter books when searchIds is set:
  const displayBooks = searchIds
    ? books.filter(b => searchIds.includes(b.id))
    : books

  if (loading) {
    return <LibrarySkeleton />
  }
  if (error)   return <div className="p-8 text-red-500">{error}</div>
  if (books.length === 0)
    return <div className="p-8 text-gray-400">No books yet. Drag a file to ingest.</div>

  return (
    <>
      <div className="p-4">
        <SearchBar onSearch={handleSearch} />
      </div>
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4 p-6">
        {displayBooks.map((book) => (
          <div
            key={book.id}
            onClick={() => onSelectBook?.(book)}
            className={`relative flex flex-col rounded-lg overflow-hidden shadow hover:shadow-md transition-shadow cursor-pointer bg-white dark:bg-gray-800 ${
              selectedIds.includes(book.id) ? "ring-2 ring-blue-500" : ""
            }`}
          >
            <label
              className="absolute top-2 right-2 z-10 flex h-6 w-6 items-center justify-center rounded bg-white/90 text-gray-700 shadow dark:bg-gray-900/90 dark:text-gray-100"
              onClick={(event) => event.stopPropagation()}
            >
              <input
                type="checkbox"
                checked={selectedIds.includes(book.id)}
                onClick={(event) => event.stopPropagation()}
                onChange={() => onToggleSelected(book.id)}
                className="h-4 w-4"
              />
            </label>
            <div className="h-48 bg-gray-200 dark:bg-gray-700 flex items-center justify-center">
              {book.cover_path ? (
                <img
                  src={`asset://${book.cover_path}`}
                  alt={book.title}
                  className="h-full w-full object-cover"
                />
              ) : (
                <span className="text-gray-400 text-sm">{book.format}</span>
              )}
            </div>
            <div className="p-2">
              <p className="text-sm font-medium truncate dark:text-white">{book.title}</p>
              <p className="text-xs text-gray-500 truncate">{book.authors.join(", ")}</p>
              {book.progress_percent > 0 && (
                <div className="mt-1 h-1 bg-gray-200 rounded-full">
                  <div
                    className="h-1 bg-blue-500 rounded-full"
                    style={{ width: `${book.progress_percent}%` }}
                  />
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
    </>
  )
}
