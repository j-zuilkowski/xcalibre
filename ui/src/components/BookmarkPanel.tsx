import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Bookmark {
  id: string
  book_id: string
  cfi: string
  label: string | null
  created_at: string
}

interface Props {
  bookId: string
  currentCfi: string
  onJump: (cfi: string) => void
  onClose: () => void
}

export function BookmarkPanel({ bookId, currentCfi, onJump, onClose }: Props) {
  const [bookmarks, setBookmarks] = useState<Bookmark[]>([])

  const load = () =>
    invoke<Bookmark[]>("list_bookmarks", { bookId }).then(setBookmarks).catch(console.error)

  useEffect(() => {
    load()
  }, [bookId])

  const add = async () => {
    await invoke("add_bookmark", { bookId, cfi: currentCfi, label: null })
    load()
  }

  const remove = async (id: string) => {
    await invoke("delete_bookmark", { bookmarkId: id })
    load()
  }

  return (
    <div className="w-72 h-full bg-white dark:bg-gray-900 border-l border-gray-200 dark:border-gray-700 flex flex-col p-4 overflow-y-auto">
      <div className="flex items-center justify-between mb-3">
        <h2 className="font-semibold text-gray-800 dark:text-gray-200">Bookmarks</h2>
        <button onClick={onClose} className="text-gray-500 hover:text-gray-700 text-lg">
          ×
        </button>
      </div>
      <button
        onClick={add}
        className="mb-4 px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700"
      >
        + Add bookmark here
      </button>
      {bookmarks.length === 0 && (
        <p className="text-sm text-gray-400">No bookmarks yet.</p>
      )}
      {bookmarks.map((bm) => (
        <div
          key={bm.id}
          className="flex items-center justify-between py-2 border-b border-gray-100 dark:border-gray-800"
        >
          <button
            onClick={() => onJump(bm.cfi)}
            className="text-sm text-left text-blue-600 hover:underline truncate flex-1"
          >
            {bm.label ?? bm.cfi.slice(0, 20) + "…"}
          </button>
          <button
            onClick={() => remove(bm.id)}
            className="ml-2 text-red-400 hover:text-red-600 text-xs"
          >
            ✕
          </button>
        </div>
      ))}
    </div>
  )
}
