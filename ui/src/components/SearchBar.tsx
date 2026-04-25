import { useState, useCallback, useRef, type RefObject } from "react"
import { invoke } from "@tauri-apps/api/core"
import { useLibraryStore } from "../store/libraryStore"

interface Props {
  inputRef?: RefObject<HTMLInputElement>
}

export function SearchBar({ inputRef }: Props) {
  const [query, setQuery] = useState("")
  const setBooks = useLibraryStore((s) => s.setBooks)
  const timer = useRef<ReturnType<typeof setTimeout>>()

  const search = useCallback(
    (q: string) => {
      clearTimeout(timer.current)
      timer.current = setTimeout(async () => {
        const query = q.trim()
        if (query.length < 2) {
          await useLibraryStore.getState().fetchBooks()
          return
        }
        try {
          const results = await invoke<any[]>("search_books", { query })
          setBooks(results)
        } catch {
          // Keep the current list if search fails.
        }
      }, 200)
    },
    [setBooks],
  )

  return (
    <input
      type="search"
      ref={inputRef}
      value={query}
      onChange={(e) => {
        setQuery(e.target.value)
        search(e.target.value)
      }}
      placeholder="Search title, author, text…"
      className="w-64 px-3 py-1.5 text-sm border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-400"
    />
  )
}
