import { create } from "zustand"
import { invoke } from "@tauri-apps/api/core"

export interface Book {
  id: string
  title: string
  authors: string[]
  format: string
  cover_path: string | null
  local_path?: string | null
  progress_percent: number
  last_opened_at: string | null
  reading_cfi?: string | null
}

interface LibraryState {
  books: Book[]
  loading: boolean
  error: string | null
  fetchBooks: () => Promise<void>
  setBooks: (books: Book[]) => void
}

export const useLibraryStore = create<LibraryState>((set) => ({
  books: [],
  loading: false,
  error: null,
  setBooks: (books) => set({ books }),
  fetchBooks: async () => {
    set({ loading: true, error: null })
    try {
      const books = await invoke<Book[]>("list_books")
      set({ books, loading: false })
    } catch (e) {
      set({ error: String(e), loading: false })
    }
  },
}))
