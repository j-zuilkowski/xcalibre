import { useEffect, useRef, useState } from "react"
import { BookDiscussDialog } from "./components/BookDiscussDialog"
import { invoke } from "@tauri-apps/api/core"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { LibraryView } from "./components/LibraryView"
import { SearchBar } from "./components/SearchBar"
import { FilterBar } from "./components/FilterBar"
import { ReaderView } from "./components/ReaderView"
import { CollectionsSidebar } from "./components/CollectionsSidebar"
import { BulkActionBar } from "./components/BulkActionBar"
import { SettingsModal } from "./components/SettingsModal"
import { ComicViewer } from "./components/ComicViewer"
import { FormatOpener } from "./components/FormatOpener"
import { UpdateBanner } from "./components/UpdateBanner"
import { MetadataEditorModal } from "./components/MetadataEditorModal"
import type { Book } from "./store/libraryStore"
import { useLibraryStore } from "./store/libraryStore"
import { useSettingsStore } from "./store/settingsStore"
import { useKeyboard } from "./hooks/useKeyboard"
import { useUpdater } from "./hooks/useUpdater"

function App() {
  const [selectedBook, setSelectedBook] = useState<Book | null>(null)
  const [selectedIds, setSelectedIds] = useState<string[]>([])
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [metadataEditorOpen, setMetadataEditorOpen] = useState(false)
  const [metadataBookIds, setMetadataBookIds] = useState<string[]>([])
  const [showDiscuss, setShowDiscuss] = useState(false)
  const [dropStatus, setDropStatus] = useState<string | null>(null)
  
  // Ctrl+Alt+A shortcut for AI book discussion
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.altKey && e.key === "a") {
        if (selectedBook) setShowDiscuss(true)
      }
    }
    window.addEventListener("keydown", handleKey)
    return () => window.removeEventListener("keydown", handleKey)
  }, [selectedBook])
  
  const [importLog, setImportLog] = useState<
    Array<{ id: string; text: string; tone: "success" | "error" }>
  >([])
  const searchRef = useRef<HTMLInputElement>(null)
  const fetchBooks = useLibraryStore((s) => s.fetchBooks)
  const theme = useSettingsStore((s) => s.theme)

  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark")
  }, [theme])
  const { updateAvailable, version } = useUpdater()
  const statusTimer = useRef<ReturnType<typeof setTimeout> | null>(null)

  const flashStatus = (message: string) => {
    setDropStatus(message)
    if (statusTimer.current) {
      clearTimeout(statusTimer.current)
    }
    statusTimer.current = setTimeout(() => {
      setDropStatus(null)
    }, 2500)
  }

  const toggleSelected = (bookId: string) => {
    setSelectedIds((ids) =>
      ids.includes(bookId) ? ids.filter((id) => id !== bookId) : [...ids, bookId],
    )
  }

  const pushImportLog = (text: string, tone: "success" | "error") => {
    const id = `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
    setImportLog((entries) => [{ id, text, tone }, ...entries].slice(0, 6))
  }

  useEffect(() => {
    let unlisten: (() => void) | null = null
    let cancelled = false
    const internals = (window as Window & {
      __TAURI_INTERNALS__?: { metadata?: { currentWindow?: unknown } }
    }).__TAURI_INTERNALS__

    if (!internals?.metadata?.currentWindow) {
      return () => {}
    }

    void getCurrentWindow()
      .onDragDropEvent(async (event) => {
        if (event.payload.type !== "drop" || event.payload.paths.length === 0) {
          return
        }

        flashStatus(`Importing ${event.payload.paths.length} file${event.payload.paths.length === 1 ? "" : "s"}…`)

        let imported = 0
        let failed = 0
        for (const path of event.payload.paths) {
          try {
            await invoke("ingest_file", { path })
            imported += 1
            const name = path.split(/[\\/]/).pop() || path
            pushImportLog(`Imported ${name}`, "success")
          } catch (error) {
            failed += 1
            console.error("Failed to ingest dropped file", path, error)
            const name = path.split(/[\\/]/).pop() || path
            pushImportLog(`Failed ${name}`, "error")
          }
        }

        await fetchBooks()
        if (failed === 0) {
          flashStatus(`Imported ${imported} file${imported === 1 ? "" : "s"}`)
        } else {
          flashStatus(`Imported ${imported}, failed ${failed}`)
        }
      })
      .then((cleanup) => {
        if (cancelled) {
          cleanup()
          return
        }
        unlisten = cleanup
      })

    return () => {
      cancelled = true
      if (statusTimer.current) {
        clearTimeout(statusTimer.current)
      }
      unlisten?.()
    }
  }, [fetchBooks])

  useKeyboard({
    "meta+,": () => setSettingsOpen(true),
    "ctrl+,": () => setSettingsOpen(true),
    "meta+f": () => searchRef.current?.focus(),
    "ctrl+f": () => searchRef.current?.focus(),
    escape: () => setSettingsOpen(false),
  })

  const importCalibre = async () => {
    const libraryPath = window.prompt("Calibre library folder:")
    if (!libraryPath?.trim()) return
    await invoke("import_calibre", { libraryPath: libraryPath.trim() })
    await fetchBooks()
  }

  const repairLibrary = async (bookIds: string[]) => {
    try {
      const repaired = await invoke<Array<{ book_id: string }>>("repair_books", { bookIds })
      await fetchBooks()
      setSelectedIds([])
      flashStatus(`Repaired ${repaired.length} book(s)`)
    } catch (error) {
      console.error("Failed to repair library", error)
      flashStatus("Repair failed")
    }
  }

  const convertSelected = async () => {
    const count = selectedIds.length
    if (count === 0) return
    const shouldTweak = count === 1 && selectedBook?.format.toUpperCase() === "EPUB"
    try {
      for (const bookId of selectedIds) {
        await invoke("convert_book_to_epub", { bookId, tweak: shouldTweak })
      }
      await fetchBooks()
      setSelectedIds([])
      flashStatus(`Converted ${count} book${count === 1 ? "" : "s"} to EPUB`)
    } catch (error) {
      console.error("Failed to convert selected books", error)
      flashStatus("Conversion failed")
    }
  }

  const openMetadataEditor = () => {
    if (selectedIds.length === 0) return
    setMetadataBookIds(selectedIds)
    setMetadataEditorOpen(true)
  }

  const selectedFormat = selectedBook?.format.toUpperCase() ?? ""
  const selectedPath = selectedBook?.local_path ?? ""

  return (
    <div className="h-screen overflow-hidden bg-gray-50 dark:bg-gray-900 flex">
      <CollectionsSidebar selectedIds={selectedIds} />
      <div className="flex-1 min-w-0 flex flex-col">
        <header className="h-14 flex items-center justify-between gap-4 px-6 border-b border-gray-200 dark:border-gray-700">
          <div>
            <h1 className="text-lg font-semibold dark:text-white">xCalibre</h1>
            <p className="text-xs text-gray-500 dark:text-gray-400">Local-first ebook reader</p>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={importCalibre}
              className="px-3 py-1.5 text-sm rounded-lg bg-gray-900 text-white hover:bg-gray-800 dark:bg-gray-100 dark:text-gray-900 dark:hover:bg-white"
            >
              Import Calibre
            </button>
            <button
              onClick={() => void repairLibrary([])}
              className="px-3 py-1.5 text-sm rounded-lg border border-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800"
            >
              Repair library
            </button>
            {dropStatus && (
              <div className="rounded-full bg-blue-100 text-blue-800 px-2 py-1 text-xs font-medium">
                {dropStatus}
              </div>
            )}
            {updateAvailable && (
              <div className="rounded-full bg-amber-100 text-amber-800 px-2 py-1 text-xs font-medium">
                Update {version}
              </div>
            )}
            <SearchBar inputRef={searchRef} />
            <button
              onClick={() => setSettingsOpen(true)}
              className="px-3 py-1.5 text-sm rounded-lg border border-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800"
            >
              Settings
            </button>
          </div>
        </header>
        {importLog.length > 0 && (
          <div className="px-6 py-2 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
            <div className="flex flex-wrap items-center gap-2 text-xs">
              <span className="text-gray-500 dark:text-gray-400 uppercase tracking-wide">
                Recent imports
              </span>
              {importLog.map((entry) => (
                <span
                  key={entry.id}
                  className={`rounded-full px-2 py-1 font-medium ${
                    entry.tone === "success"
                      ? "bg-emerald-100 text-emerald-800"
                      : "bg-red-100 text-red-800"
                  }`}
                >
                  {entry.text}
                </span>
              ))}
            </div>
          </div>
        )}
        <FilterBar />
        <main className="flex-1 min-h-0">
          <LibraryView
            selectedIds={selectedIds}
            onToggleSelected={toggleSelected}
            onSelectBook={setSelectedBook}
          />
        </main>
        {selectedBook &&
          (selectedFormat === "EPUB" ? (
            <ReaderView
              bookId={selectedBook.id}
              format={selectedBook.format}
              initialCfi={selectedBook.reading_cfi}
              onClose={() => setSelectedBook(null)}
            />
          ) : selectedFormat === "CBZ" || selectedFormat === "CBR" ? (
            <ComicViewer bookId={selectedBook.id} onClose={() => setSelectedBook(null)} />
          ) : (
            <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 p-6">
              <div className="w-full max-w-lg rounded-2xl border border-gray-800 bg-gray-950 p-6 text-gray-100 shadow-2xl">
                <div className="flex items-start justify-between gap-4">
                  <div>
                    <p className="text-xs uppercase tracking-wide text-gray-400">Open externally</p>
                    <h2 className="mt-1 text-xl font-semibold">{selectedBook.title}</h2>
                    <p className="mt-1 text-sm text-gray-400">
                      {selectedFormat || "Unknown format"}
                    </p>
                  </div>
                  <button
                    onClick={() => setSelectedBook(null)}
                    className="rounded-lg border border-gray-800 px-3 py-1 text-sm text-gray-300 hover:bg-gray-900"
                  >
                    Close
                  </button>
                </div>
                <div className="mt-6 flex items-center gap-3">
                  {selectedPath ? (
                    <FormatOpener
                      bookId={selectedBook.id}
                      format={selectedBook.format}
                      filePath={selectedPath}
                    />
                  ) : (
                    <p className="text-sm text-gray-400">
                      No local file path is available for this book.
                    </p>
                  )}
                </div>
              </div>
            </div>
          ))}
        <BulkActionBar
          selectedIds={selectedIds}
          selectedBook={selectedBook}
          onDone={() => setSelectedIds([])}
          onEditMetadata={openMetadataEditor}
          onRepair={() => repairLibrary(selectedIds)}
          onConvert={convertSelected}
        />
        <SettingsModal open_={settingsOpen} onClose={() => setSettingsOpen(false)} />
        <MetadataEditorModal
          open={metadataEditorOpen}
          bookIds={metadataBookIds}
          onClose={() => {
            setMetadataEditorOpen(false)
            setMetadataBookIds([])
          }}
          onSaved={async () => {
            await fetchBooks()
            setSelectedIds([])
            setMetadataEditorOpen(false)
            setMetadataBookIds([])
          }}
        />
        <UpdateBanner />
      </div>
    </div>
  )
}

export default App
