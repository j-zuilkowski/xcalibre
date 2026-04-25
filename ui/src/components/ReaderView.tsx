import { useEffect, useRef, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { BookmarkPanel } from "./BookmarkPanel"
import { ReaderToolbar } from "./ReaderToolbar"
import { AnnotationsSidebar } from "./AnnotationsSidebar"
import type { Annotation } from "../reader/highlights"
import { applyHighlights } from "../reader/highlights"
import { useReaderKeyboard } from "../hooks/useReaderKeyboard"
import { useSettingsStore } from "../store/settingsStore"

interface Props {
  bookId: string
  format: string
  initialCfi?: string | null
  onClose: () => void
}

type Panel = "bookmarks" | "annotations" | null

function parseSpineIndex(cfi: string | null | undefined): number {
  if (!cfi) return 0
  const index = Number.parseInt(cfi.split(":")[0] ?? "0", 10)
  return Number.isFinite(index) && index >= 0 ? index : 0
}

export function ReaderView({ bookId, format, initialCfi, onClose }: Props) {
  const iframeRef = useRef<HTMLIFrameElement>(null)
  const highlightCleanupRef = useRef<(() => void) | null>(null)
  const currentCfiRef = useRef(initialCfi ?? "0:0:0")
  const [chapterHtml, setChapterHtml] = useState<string | null>(null)
  const [chapterError, setChapterError] = useState<string | null>(null)
  const [spine, setSpine] = useState<string[]>([])
  const [spineIndex, setSpineIndex] = useState(0)
  const [currentCfi, setCurrentCfi] = useState(initialCfi ?? "0:0:0")
  const [progress, setProgress] = useState(0)
  const [annotations, setAnnotations] = useState<Annotation[]>([])
  const [annotationsVersion, setAnnotationsVersion] = useState(0)
  const [panel, setPanel] = useState<Panel>(null)
  const { fontSize, theme, fontFamily, setFontSize } = useSettingsStore()

  const postCss = () => {
    const font = fontFamily === "serif"
      ? "Georgia, serif"
      : fontFamily === "sans"
        ? "system-ui, sans-serif"
        : "monospace"
    iframeRef.current?.contentWindow?.postMessage(
      {
        type: "xcalibre-css",
        css: {
          "--size": `${fontSize}px`,
          "--font": font,
          "--theme": theme,
        },
      },
      "*",
    )
  }

  useEffect(() => {
    currentCfiRef.current = currentCfi
  }, [currentCfi])

  useEffect(() => {
    let active = true
    invoke<string[]>("get_spine", { jobId: bookId })
      .then((items) => {
        if (!active) return
        setSpine(items)
        setSpineIndex(items.length > 0 ? Math.min(items.length - 1, parseSpineIndex(initialCfi)) : 0)
      })
      .catch(console.error)

    return () => {
      active = false
    }
  }, [bookId, initialCfi])

  useEffect(() => {
    const load = async () => {
      try {
        const rows = await invoke<Annotation[]>("get_annotations", { bookId })
        setAnnotations(rows)
      } catch (error) {
        console.error("failed to load annotations", error)
        setAnnotations([])
      }
    }

    void load()
  }, [bookId, annotationsVersion])

  useEffect(() => {
    const listener = () => setAnnotationsVersion((value) => value + 1)
    window.addEventListener("xcalibre-annotations-updated", listener as EventListener)
    return () => window.removeEventListener("xcalibre-annotations-updated", listener as EventListener)
  }, [])

  useEffect(() => {
    const handler = (event: MessageEvent) => {
      const data = event.data as
        | { type?: string; jobId?: string; cfi?: string; progress?: number }
        | undefined

      if (!data || data.type !== "xcalibre-cfi" || data.jobId !== bookId) return
      if (typeof data.cfi === "string") {
        currentCfiRef.current = data.cfi
        setCurrentCfi(data.cfi)
      }
      if (typeof data.progress === "number") {
        setProgress(data.progress)
        void invoke("update_progress", { bookId, percent: data.progress }).catch(console.error)
      }

      void invoke("update_position", {
        bookId,
        position: data.cfi ?? currentCfiRef.current,
      }).catch(console.error)
    }

    window.addEventListener("message", handler)
    return () => window.removeEventListener("message", handler)
  }, [bookId])

  useEffect(() => {
    const href = spine[spineIndex]
    if (!href) {
      setChapterHtml(null)
      return
    }

    let active = true
    setChapterHtml(null)
    setChapterError(null)
    invoke<string>("get_epub_chapter_html", { bookId, href })
      .then((html) => {
        if (active) {
          setChapterHtml(html)
          setChapterError(null)
        }
      })
      .catch((error) => {
        console.error("failed to load chapter html", error)
        if (active) {
          setChapterHtml(null)
          setChapterError(error instanceof Error ? error.message : String(error))
        }
      })

    return () => {
      active = false
    }
  }, [bookId, spine, spineIndex])

  useEffect(() => {
    if (chapterHtml) {
      postCss()
    }
  }, [chapterHtml, fontSize, theme, fontFamily])

  useReaderKeyboard({
    nextPage: () => setSpineIndex((index) => Math.min(Math.max(0, spine.length - 1), index + 1)),
    prevPage: () => setSpineIndex((index) => Math.max(0, index - 1)),
    close: onClose,
    increaseFontSize: () => setFontSize(Math.min(32, fontSize + 1)),
    decreaseFontSize: () => setFontSize(Math.max(12, fontSize - 1)),
  })

  useEffect(() => {
    return () => {
      highlightCleanupRef.current?.()
      highlightCleanupRef.current = null
    }
  }, [])

  const jumpToCfi = (cfi: string) => {
    const parsedIndex = parseSpineIndex(cfi)
    const nextIndex = spine.length > 0
      ? Math.min(Math.max(0, parsedIndex), spine.length - 1)
      : 0

    setSpineIndex(nextIndex)
    currentCfiRef.current = cfi
    setCurrentCfi(cfi)
    iframeRef.current?.contentWindow?.postMessage(
      {
        type: "xcalibre-context",
        spineIndex: nextIndex,
        totalSpineItems: spine.length,
        cfi,
      },
      "*",
    )
  }

  const handleIframeLoad = () => {
    const iframe = iframeRef.current
    const doc = iframe?.contentDocument
    const win = iframe?.contentWindow
    if (!iframe || !doc || !win) return

    highlightCleanupRef.current?.()
    applyHighlights(doc, annotations)

    const onMouseUp = async () => {
      const selection = doc.getSelection()
      if (!selection || selection.isCollapsed) return
      const text = selection.toString().trim()
      if (text.length < 3) return

      try {
        await invoke("create_annotation", {
          bookId,
          annotationType: "highlight",
          cfi: currentCfiRef.current,
          selectedText: text,
          note: null,
          color: "yellow",
        })
        window.dispatchEvent(new Event("xcalibre-annotations-updated"))
      } catch (error) {
        console.error("failed to create annotation", error)
      } finally {
        selection.removeAllRanges()
      }
    }

    doc.addEventListener("mouseup", onMouseUp)
    highlightCleanupRef.current = () => doc.removeEventListener("mouseup", onMouseUp)

    postCss()
    win.postMessage(
      {
        type: "xcalibre-context",
        spineIndex,
        totalSpineItems: spine.length,
        cfi: currentCfiRef.current,
      },
      "*",
    )
  }

  const currentHref = spine[spineIndex]
  const progressLabel = `${Math.round(progress)}%`

  return (
    <div className="fixed inset-0 z-50 flex bg-gray-950 text-gray-100">
      <div className="flex min-w-0 flex-1 flex-col">
        <div className="border-b border-gray-800 bg-gray-900/95 px-4 py-3 backdrop-blur">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="min-w-0">
              <p className="truncate text-sm font-semibold">{format.toUpperCase()} Reader</p>
              <p className="truncate text-xs text-gray-400">
                {currentHref ?? "Loading spine..."} {progressLabel}
              </p>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setSpineIndex((index) => Math.max(0, index - 1))}
                disabled={spineIndex === 0}
                className="rounded-lg border border-gray-700 px-3 py-1.5 text-sm disabled:opacity-40"
              >
                Prev
              </button>
              <button
                onClick={() =>
                  setSpineIndex((index) => Math.min(Math.max(0, spine.length - 1), index + 1))
                }
                disabled={spineIndex >= Math.max(0, spine.length - 1)}
                className="rounded-lg border border-gray-700 px-3 py-1.5 text-sm disabled:opacity-40"
              >
                Next
              </button>
              <button
                onClick={() => setPanel(panel === "bookmarks" ? null : "bookmarks")}
                className={`rounded-lg px-3 py-1.5 text-sm ${
                  panel === "bookmarks" ? "bg-blue-600 text-white" : "border border-gray-700"
                }`}
              >
                Bookmarks
              </button>
              <button
                onClick={() => setPanel(panel === "annotations" ? null : "annotations")}
                className={`rounded-lg px-3 py-1.5 text-sm ${
                  panel === "annotations" ? "bg-blue-600 text-white" : "border border-gray-700"
                }`}
              >
                Annotations
              </button>
              <button
                onClick={onClose}
                className="rounded-lg border border-gray-700 px-3 py-1.5 text-sm"
              >
                Close
              </button>
            </div>
          </div>
          <div className="mt-3">
            <ReaderToolbar iframeRef={iframeRef} />
          </div>
        </div>

        <div className="flex min-h-0 flex-1 bg-black">
          <div className="min-w-0 flex-1">
            {!chapterHtml ? (
              <div className="flex h-full items-center justify-center text-sm text-gray-400">
                {chapterError ?? "Loading reader..."}
              </div>
            ) : (
              <iframe
                key={`${bookId}-${spineIndex}-${annotationsVersion}`}
                ref={iframeRef}
                srcDoc={chapterHtml}
                title={`${format} reader`}
                className="h-full w-full bg-white"
                onLoad={handleIframeLoad}
              />
            )}
          </div>

          {panel && (
            <aside className="w-80 border-l border-gray-800 bg-gray-900">
              {panel === "bookmarks" ? (
                <BookmarkPanel
                  bookId={bookId}
                  currentCfi={currentCfi}
                  onJump={jumpToCfi}
                  onClose={() => setPanel(null)}
                />
              ) : (
                <AnnotationsSidebar bookId={bookId} onJumpTo={jumpToCfi} />
              )}
            </aside>
          )}
        </div>
      </div>
    </div>
  )
}
