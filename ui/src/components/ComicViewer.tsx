import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Props {
  bookId: string
  onClose: () => void
}

export function ComicViewer({ bookId, onClose }: Props) {
  const [pages, setPages] = useState<string[]>([])
  const [currentPage, setCurrentPage] = useState(0)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    invoke<string[]>("list_comic_pages", { bookId })
      .then((p) => { setPages(p); setLoading(false) })
      .catch(() => setLoading(false))
  }, [bookId])

  const prev = () => setCurrentPage((p) => Math.max(0, p - 1))
  const next = () => setCurrentPage((p) => Math.min(pages.length - 1, p + 1))

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "ArrowRight" || e.key === "ArrowDown") next()
      if (e.key === "ArrowLeft"  || e.key === "ArrowUp")   prev()
      if (e.key === "Escape") onClose()
    }
    window.addEventListener("keydown", handler)
    return () => window.removeEventListener("keydown", handler)
  }, [pages.length])

  if (loading) {
    return (
      <div className="fixed inset-0 bg-black flex items-center justify-center z-50">
        <p className="text-white">Loading pages…</p>
      </div>
    )
  }

  if (pages.length === 0) {
    return (
      <div className="fixed inset-0 bg-black flex flex-col items-center justify-center z-50 gap-4">
        <p className="text-white">No pages found in this archive.</p>
        <button onClick={onClose} className="text-sm px-4 py-2 bg-white text-black rounded">Close</button>
      </div>
    )
  }

  return (
    <div className="fixed inset-0 bg-black flex flex-col z-50">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 bg-black/80">
        <button onClick={onClose} className="text-white text-sm hover:text-gray-300">✕ Close</button>
        <span className="text-white text-sm">
          Page {currentPage + 1} / {pages.length}
        </span>
        <div className="flex gap-2">
          <button onClick={prev} disabled={currentPage === 0}
            className="text-white text-sm px-2 disabled:opacity-30">◀</button>
          <button onClick={next} disabled={currentPage === pages.length - 1}
            className="text-white text-sm px-2 disabled:opacity-30">▶</button>
        </div>
      </div>

      {/* Page image */}
      <div className="flex-1 flex items-center justify-center overflow-hidden">
        <img
          src={`asset://${pages[currentPage]}`}
          alt={`Page ${currentPage + 1}`}
          className="max-h-full max-w-full object-contain"
        />
      </div>
    </div>
  )
}
