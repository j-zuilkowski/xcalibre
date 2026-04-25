import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { useLibraryStore } from "../store/libraryStore"

export function FilterBar() {
  const [formats] = useState(["EPUB", "PDF", "MOBI", "AZW3", "CBZ", "CBR", "TXT"])
  const [statuses] = useState(["PENDING", "READY_TO_PUSH", "PUSHING", "RETRYING", "COMPLETED", "FAILED"])
  const [authors, setAuthors] = useState<string[]>([])
  const [series, setSeries] = useState<string[]>([])
  const [tags, setTags] = useState<string[]>([])
  const [format, setFormat] = useState("")
  const [status, setStatus] = useState("")
  const [author, setAuthor] = useState("")
  const [seriesName, setSeriesName] = useState("")
  const [tag, setTag] = useState("")
  const setBooks = useLibraryStore((s) => s.setBooks)

  useEffect(() => {
    invoke<string[]>("list_authors").then(setAuthors).catch(() => {})
    invoke<string[]>("list_series").then(setSeries).catch(() => {})
    invoke<string[]>("list_tags").then(setTags).catch(() => {})
  }, [])

  const apply = async (
    nextFormat: string,
    nextStatus: string,
    nextAuthor: string,
    nextSeries: string,
    nextTag: string,
  ) => {
    try {
      const results = await invoke<any[]>("filter_books", {
        format: nextFormat || null,
        status: nextStatus || null,
        author: nextAuthor || null,
        series: nextSeries || null,
        tag: nextTag || null,
      })
      setBooks(results)
    } catch {
      // Ignore filter errors and keep the current list.
    }
  }

  const clear = async () => {
    setFormat("")
    setStatus("")
    setAuthor("")
    setSeriesName("")
    setTag("")
    await useLibraryStore.getState().fetchBooks()
  }

  return (
    <div className="flex flex-wrap items-center gap-3 px-4 py-2 bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 text-sm">
      <select
        value={format}
        onChange={(e) => {
          const value = e.target.value
          setFormat(value)
          apply(value, status, author, seriesName, tag)
        }}
        className="border border-gray-300 rounded px-2 py-1 bg-white dark:bg-gray-700"
      >
        <option value="">All formats</option>
        {formats.map((f) => <option key={f} value={f}>{f}</option>)}
      </select>

      <select
        value={status}
        onChange={(e) => {
          const value = e.target.value
          setStatus(value)
          apply(format, value, author, seriesName, tag)
        }}
        className="border border-gray-300 rounded px-2 py-1 bg-white dark:bg-gray-700"
      >
        <option value="">All statuses</option>
        {statuses.map((s) => <option key={s} value={s}>{s}</option>)}
      </select>

      <input
        type="text"
        value={author}
        onChange={(e) => {
          const value = e.target.value
          setAuthor(value)
          apply(format, status, value, seriesName, tag)
        }}
        placeholder="Author…"
        className="border border-gray-300 rounded px-2 py-1 bg-white dark:bg-gray-700 w-36"
      />

      <select
        value={seriesName}
        onChange={(e) => {
          const value = e.target.value
          setSeriesName(value)
          apply(format, status, author, value, tag)
        }}
        className="border border-gray-300 rounded px-2 py-1 bg-white dark:bg-gray-700"
      >
        <option value="">All series</option>
        {series.map((s) => <option key={s} value={s}>{s}</option>)}
      </select>

      <select
        value={tag}
        onChange={(e) => {
          const value = e.target.value
          setTag(value)
          apply(format, status, author, seriesName, value)
        }}
        className="border border-gray-300 rounded px-2 py-1 bg-white dark:bg-gray-700"
      >
        <option value="">All tags</option>
        {tags.map((t) => <option key={t} value={t}>{t}</option>)}
      </select>

      <button
        onClick={clear}
        className="px-3 py-1 text-xs text-gray-500 hover:text-gray-700 border border-gray-300 rounded"
      >
        Clear
      </button>
    </div>
  )
}
