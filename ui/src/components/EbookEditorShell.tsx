import { useState, useEffect, useCallback, useRef } from "react"
import { invoke } from "@tauri-apps/api/core"
import { FileTreePanel } from "./FileTreePanel"

interface Book {
  id: string; title: string; authors: string[]; format: string
  cover_path: string | null; progress_percent: number; last_opened_at: string | null
  file_path?: string
}
interface ManifestItem { id?: string; href: string; media_type: string }

interface Props { book: Book; onClose: () => void }

export function EbookEditorShell({ book, onClose }: Props) {
  const [manifest, setManifest] = useState<ManifestItem[]>([])
  const [selected, setSelected] = useState<ManifestItem | null>(null)
  const [dirty, setDirty] = useState(false)
  const [saving, setSaving] = useState(false)
  const [loading, setLoading] = useState(true)
  const [content, setContent] = useState("")
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  useEffect(() => {
    if (!book.file_path) return
    invoke<{ spine: string[]; manifest: ManifestItem[]; metadata: { title?: string } }>(
      "editor_open_epub", { bookId: book.id, filePath: book.file_path }
    ).then(result => { setManifest(result.manifest) })
    .catch(console.error)
    .finally(() => setLoading(false))
    return () => { invoke("editor_close", { bookId: book.id }).catch(console.error) }
  }, [book.id, book.file_path])

  const loadItem = useCallback(async (item: ManifestItem) => {
    setSelected(item)
    const c = await invoke<string>("editor_read_item", { bookId: book.id, href: item.href })
    setContent(c)
    setDirty(false)
  }, [book.id])

  async function handleSave() {
    if (!selected) return
    setSaving(true)
    try {
      await invoke("editor_write_item", { bookId: book.id, href: selected.href, content })
      await invoke("editor_save_epub", { bookId: book.id })
      setDirty(false)
    } catch (e) { console.error(e) }
    finally { setSaving(false) }
  }

  return (
    <div style={{ position: "fixed", inset: 0, background: "var(--bg-base, #1e1e2e)", display: "flex", flexDirection: "column", zIndex: 2000, color: "var(--text-primary, #cdd6f4)" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "0.75rem", padding: "0.5rem 1rem", borderBottom: "1px solid var(--border, #45475a)" }}>
        <button data-testid="editor-close-btn" onClick={onClose} style={{ padding: "0.3rem 0.75rem", background: "transparent", border: "none", cursor: "pointer", color: "inherit", fontSize: "1.1rem" }}>←</button>
        <span style={{ fontWeight: 600 }}>{book.title} — Editor</span>
        {dirty && <span style={{ color: "var(--yellow, #f9e2af)", fontSize: "0.85rem" }}>Unsaved changes</span>}
        <div style={{ flex: 1 }} />
        <button data-testid="editor-save-btn" onClick={handleSave} disabled={saving || !dirty}
          style={{ padding: "0.4rem 1rem", background: "var(--blue, #89b4fa)", border: "none", borderRadius: "6px", cursor: "pointer", color: "#1e1e2e", fontWeight: 600, opacity: (saving || !dirty) ? 0.5 : 1 }}>
          {saving ? "Saving…" : "Save"}
        </button>
      </div>
      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        {loading ? <div style={{ padding: "2rem", color: "var(--text-muted, #6c7086)" }}>Opening EPUB…</div> : (
          <>
            <FileTreePanel items={manifest} selectedHref={selected?.href ?? null} onSelectItem={loadItem} />
            <div data-testid="editor-content-panel" style={{ flex: 1, overflow: "auto", padding: "1rem" }}>
              <textarea value={content} onChange={e => { setContent(e.target.value); setDirty(true) }}
                style={{ width: "100%", height: "100%", minHeight: "400px", background: "var(--bg-overlay, #313244)", color: "inherit", border: "none", padding: "1rem", fontFamily: "monospace", fontSize: "0.9rem", resize: "none" }} />
            </div>
          </>
        )}
      </div>
    </div>
  )
}
