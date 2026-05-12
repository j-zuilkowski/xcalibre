import { useState, useRef } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Props { bookId: string; onSaved: () => void }

export function CoverEditorPanel({ bookId, onSaved }: Props) {
  const [preview, setPreview] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const fileRef = useRef<HTMLInputElement>(null)

  async function handleFile(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0]
    if (!file) return
    if (!file.type.startsWith("image/")) { setError("Please select an image file."); return }
    const url = URL.createObjectURL(file)
    setPreview(url); setError(null)
    const buffer = await file.arrayBuffer()
    const bytes = new Uint8Array(buffer)
    const b64 = btoa(String.fromCharCode(...bytes))
    setSaving(true)
    try {
      await invoke("editor_set_cover", { bookId, dataB64: b64, mimeType: file.type })
      await invoke("editor_save_epub", { bookId })
      onSaved()
    } catch (e) { setError(String(e)) } finally { setSaving(false) }
  }

  return (
    <div style={{ padding: "1rem", display: "flex", flexDirection: "column", gap: "0.75rem" }}>
      <h3 style={{ margin: 0 }}>Cover Image</h3>
      {preview && <img src={preview} alt="New cover preview" data-testid="cover-preview" style={{ maxWidth: "200px", maxHeight: "280px", borderRadius: "4px" }} />}
      {error && <span style={{ color: "var(--red, #f38ba8)", fontSize: "0.85rem" }}>{error}</span>}
      <input ref={fileRef} type="file" accept="image/jpeg,image/png,image/webp" onChange={handleFile} style={{ display: "none" }} data-testid="cover-file-input" />
      <button data-testid="cover-select-btn" onClick={() => fileRef.current?.click()} disabled={saving}
        style={{ padding: "0.4rem 1rem", background: "var(--bg-overlay, #313244)", border: "1px solid var(--border, #45475a)", borderRadius: "6px", cursor: "pointer", color: "inherit" }}>
        {saving ? "Saving…" : "Choose Cover Image…"}
      </button>
    </div>
  )
}
