import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { useLibraryStore } from "../store/libraryStore"

interface Collection {
  id: string
  name: string
  created_at: string
}

interface Props {
  selectedIds: string[]
}

export function CollectionsSidebar({ selectedIds }: Props) {
  const [collections, setCollections] = useState<Collection[]>([])
  const [newName, setNewName] = useState("")
  const [active, setActive] = useState<string | null>(null)
  const setBooks = useLibraryStore((s) => s.setBooks)

  const load = () =>
    invoke<Collection[]>("list_collections")
      .then(setCollections)
      .catch(() => {})

  useEffect(() => {
    load()
  }, [])

  const openCollection = async (id: string) => {
    setActive(id)
    const ids = await invoke<string[]>("get_books_in_collection", { collectionId: id }).catch(
      () => [] as string[],
    )
    if (ids.length === 0) {
      setBooks([])
      return
    }
    const allBooks = await invoke<any[]>("list_books").catch(() => [] as any[])
    setBooks(allBooks.filter((book: any) => ids.includes(book.id)))
  }

  const showAll = async () => {
    setActive(null)
    await useLibraryStore.getState().fetchBooks()
  }

  const newCollection = async () => {
    if (!newName.trim()) return
    await invoke("create_collection", { name: newName.trim() })
    setNewName("")
    load()
  }

  const addSelected = async () => {
    if (!active || selectedIds.length === 0) return
    await Promise.all(
      selectedIds.map((bookId) =>
        invoke("add_book_to_collection", { collectionId: active, bookId }),
      ),
    )
    await openCollection(active)
  }

  return (
    <aside className="w-52 shrink-0 h-full bg-gray-50 dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 flex flex-col p-3">
      <button
        onClick={showAll}
        className={`text-left px-2 py-1.5 rounded text-sm mb-1 ${
          active === null
            ? "bg-blue-100 text-blue-700 font-medium"
            : "hover:bg-gray-100 dark:hover:bg-gray-800"
        }`}
      >
        All Books
      </button>

      <div className="flex-1 overflow-y-auto">
        {collections.map((collection) => (
          <button
            key={collection.id}
            onClick={() => openCollection(collection.id)}
            className={`w-full text-left px-2 py-1.5 rounded text-sm truncate ${
              active === collection.id
                ? "bg-blue-100 text-blue-700 font-medium"
                : "hover:bg-gray-100 dark:hover:bg-gray-800"
            }`}
          >
            {collection.name}
          </button>
        ))}
      </div>

      <button
        onClick={addSelected}
        disabled={!active || selectedIds.length === 0}
        className="mt-2 text-xs text-blue-600 hover:underline text-left disabled:opacity-40 disabled:hover:no-underline"
      >
        + Add selected to collection
      </button>
      <div className="mt-2 flex gap-1">
        <input
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && newCollection()}
          placeholder="New collection…"
          className="flex-1 min-w-0 text-xs border rounded px-1 py-0.5 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
        />
        <button
          onClick={newCollection}
          className="text-xs px-1.5 py-0.5 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          +
        </button>
      </div>
    </aside>
  )
}
