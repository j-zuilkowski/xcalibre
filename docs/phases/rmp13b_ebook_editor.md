# RMP-13b — Ebook Editor Level C (Green: Implementation)

> Prerequisite: rmp13a complete, rmp05b complete.
> TDD role: GREEN — implement EpubEditor backend, Tauri commands, and editor UI shell.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R13b-T01 | `processing/src/editor/mod.rs` — EpubEditor struct | ⬜ |
| R13b-T02 | Tauri commands: open/read/write/save/metadata/cover | ⬜ |
| R13b-T03 | Install CodeMirror + `FileTreePanel.tsx` | ⬜ |
| R13b-T04 | `EbookEditorShell.tsx` — main editor layout | ⬜ |
| R13b-T05 | `MetadataEditorPanel.tsx` + `CoverEditorPanel.tsx` | ⬜ |
| R13b-T06 | Milestone check + visual inspection | ⬜ |

---

## R13b-T01

Write `processing/src/editor/mod.rs`:
```rust
use crate::error::ProcessingError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use xcalibre_epub::{Container, EpubOPF};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestItem {
    pub id:         String,
    pub href:       String,
    pub media_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorMetadata {
    pub title:   Option<String>,
    pub authors: Vec<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
}

pub struct EpubEditor {
    container: Container,
    path:      PathBuf,
}

impl EpubEditor {
    pub fn open(path: &Path) -> Result<Self, ProcessingError> {
        let container = Container::open(path)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        Ok(Self { container, path: path.to_path_buf() })
    }

    pub fn spine_items(&self) -> Vec<String> {
        self.container.spine_items().unwrap_or_default()
    }

    pub fn manifest_items(&self) -> Vec<ManifestItem> {
        self.container.manifest_items()
            .unwrap_or_default()
            .into_iter()
            .map(|(id, href, media_type)| ManifestItem { id, href, media_type })
            .collect()
    }

    pub fn read_item(&self, href: &str) -> Result<Vec<u8>, ProcessingError> {
        self.container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))
    }

    pub fn write_item(&mut self, href: &str, content: &[u8]) -> Result<(), ProcessingError> {
        self.container.write_item(href, content)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))
    }

    pub fn metadata(&self) -> EditorMetadata {
        let opf = self.container.opf().unwrap_or_default();
        EditorMetadata {
            title:       opf.title().map(|s| s.to_string()),
            authors:     opf.authors().iter().map(|s| s.to_string()).collect(),
            language:    opf.language().map(|s| s.to_string()),
            publisher:   opf.publisher().map(|s| s.to_string()),
            description: opf.description().map(|s| s.to_string()),
        }
    }

    pub fn set_title(&mut self, title: &str) {
        if let Ok(mut opf) = self.container.opf() {
            opf.set_title(title);
            let _ = self.container.write_opf(&opf);
        }
    }

    pub fn set_authors(&mut self, authors: &[&str]) {
        if let Ok(mut opf) = self.container.opf() {
            opf.set_authors(authors);
            let _ = self.container.write_opf(&opf);
        }
    }

    pub fn cover_bytes(&self) -> Result<Vec<u8>, ProcessingError> {
        self.container.cover_bytes()
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))
    }

    pub fn set_cover(&mut self, data: &[u8], mime: &str) -> Result<(), ProcessingError> {
        self.container.set_cover(data, mime)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))
    }

    pub fn save(&mut self) -> Result<(), ProcessingError> {
        self.container.save(&self.path)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))
    }

    pub fn save_as(&self, dest: &Path) -> Result<(), ProcessingError> {
        self.container.save(dest)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))
    }
}
```

Add to `processing/src/lib.rs`:
```rust
pub mod editor;
```

Then run:
```bash
cargo test --workspace -- test_epub_editor test_epub_cover
git add processing/src/editor/mod.rs processing/src/lib.rs
git commit -m "R13b-T01: EpubEditor struct — all editor backend tests green"
```

---

## R13b-T02

Write `src-tauri/src/commands/editor.rs`:
```rust
use tauri::State;
use xcalibre_processing::editor::{EpubEditor, EditorMetadata, ManifestItem};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;

// In-memory editor sessions keyed by book_id
pub type EditorSessions = Mutex<HashMap<String, EpubEditor>>;

#[derive(Debug, Serialize)]
pub struct OpenEpubResult {
    pub spine:    Vec<String>,
    pub manifest: Vec<ManifestItem>,
    pub metadata: EditorMetadata,
}

#[tauri::command]
pub async fn editor_open_epub(
    book_id:   String,
    file_path: String,
    sessions:  State<'_, EditorSessions>,
) -> Result<OpenEpubResult, String> {
    let editor = EpubEditor::open(std::path::Path::new(&file_path))
        .map_err(|e| e.to_string())?;
    let result = OpenEpubResult {
        spine:    editor.spine_items(),
        manifest: editor.manifest_items(),
        metadata: editor.metadata(),
    };
    sessions.lock().unwrap().insert(book_id, editor);
    Ok(result)
}

#[tauri::command]
pub async fn editor_read_item(
    book_id: String,
    href:    String,
    sessions: State<'_, EditorSessions>,
) -> Result<String, String> {
    let sessions = sessions.lock().unwrap();
    let editor = sessions.get(&book_id)
        .ok_or_else(|| format!("no editor session for {book_id}"))?;
    let bytes = editor.read_item(&href).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

#[tauri::command]
pub async fn editor_write_item(
    book_id:  String,
    href:     String,
    content:  String,
    sessions: State<'_, EditorSessions>,
) -> Result<(), String> {
    let mut sessions = sessions.lock().unwrap();
    let editor = sessions.get_mut(&book_id)
        .ok_or_else(|| format!("no editor session for {book_id}"))?;
    editor.write_item(&href, content.as_bytes()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn editor_save_epub(
    book_id:  String,
    sessions: State<'_, EditorSessions>,
) -> Result<(), String> {
    let mut sessions = sessions.lock().unwrap();
    let editor = sessions.get_mut(&book_id)
        .ok_or_else(|| format!("no editor session for {book_id}"))?;
    editor.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn editor_update_metadata(
    book_id:  String,
    title:    Option<String>,
    authors:  Option<Vec<String>>,
    sessions: State<'_, EditorSessions>,
) -> Result<(), String> {
    let mut sessions = sessions.lock().unwrap();
    let editor = sessions.get_mut(&book_id)
        .ok_or_else(|| format!("no editor session for {book_id}"))?;
    if let Some(t) = title   { editor.set_title(&t); }
    if let Some(a) = authors {
        let refs: Vec<&str> = a.iter().map(|s| s.as_str()).collect();
        editor.set_authors(&refs);
    }
    Ok(())
}

#[tauri::command]
pub async fn editor_set_cover(
    book_id:   String,
    data_b64:  String,
    mime_type: String,
    sessions:  State<'_, EditorSessions>,
) -> Result<(), String> {
    use base64::Engine;
    let data = base64::engine::general_purpose::STANDARD
        .decode(&data_b64)
        .map_err(|e| e.to_string())?;
    let mut sessions = sessions.lock().unwrap();
    let editor = sessions.get_mut(&book_id)
        .ok_or_else(|| format!("no editor session for {book_id}"))?;
    editor.set_cover(&data, &mime_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn editor_close(
    book_id:  String,
    sessions: State<'_, EditorSessions>,
) -> Result<(), String> {
    sessions.lock().unwrap().remove(&book_id);
    Ok(())
}
```

Add `base64 = "0.22"` to `src-tauri/Cargo.toml` dependencies.

Register `EditorSessions` as managed state and all commands in `src-tauri/src/main.rs`:
```rust
.manage(commands::editor::EditorSessions::default())
// In invoke_handler:
commands::editor::editor_open_epub,
commands::editor::editor_read_item,
commands::editor::editor_write_item,
commands::editor::editor_save_epub,
commands::editor::editor_update_metadata,
commands::editor::editor_set_cover,
commands::editor::editor_close,
```

```bash
cargo build --workspace
git add src-tauri/src/commands/editor.rs src-tauri/src/main.rs src-tauri/Cargo.toml
git commit -m "R13b-T02: EPUB editor Tauri commands — open/read/write/save/metadata/cover"
```

---

## R13b-T03

Install CodeMirror:
```bash
cd ui && npm install @codemirror/view @codemirror/state @codemirror/lang-html @codemirror/lang-css @codemirror/theme-one-dark && cd ..
```

Write `ui/src/components/FileTreePanel.tsx`:
```tsx
interface ManifestItem {
  id?: string
  href: string
  media_type: string
}

interface Props {
  items: ManifestItem[]
  selectedHref: string | null
  onSelectItem: (item: ManifestItem) => void
}

function baseName(href: string) {
  return href.split("/").pop() ?? href
}

function groupKey(mediaType: string): "html" | "css" | "images" | "other" {
  if (mediaType.includes("html") || mediaType.includes("xhtml")) return "html"
  if (mediaType.includes("css")) return "css"
  if (mediaType.includes("image")) return "images"
  return "other"
}

const GROUP_LABELS: Record<string, string> = {
  html: "Text", css: "Stylesheets", images: "Images", other: "Other",
}

export function FileTreePanel({ items, selectedHref, onSelectItem }: Props) {
  const groups: Record<string, ManifestItem[]> = { html: [], css: [], images: [], other: [] }
  for (const item of items) {
    groups[groupKey(item.media_type)].push(item)
  }

  return (
    <div
      data-testid="editor-file-tree"
      style={{
        width: "200px", flexShrink: 0, overflowY: "auto",
        borderRight: "1px solid var(--border, #45475a)",
        padding: "0.5rem 0",
      }}
    >
      {(["html", "css", "images", "other"] as const).map(key => {
        const grpItems = groups[key]
        if (grpItems.length === 0) return null
        return (
          <div key={key}>
            <div
              data-testid={`file-group-${key}`}
              style={{
                padding: "0.25rem 0.75rem",
                fontSize: "0.75rem", fontWeight: 700,
                color: "var(--text-muted, #6c7086)",
                textTransform: "uppercase", letterSpacing: "0.05em",
              }}
            >
              {GROUP_LABELS[key]}
            </div>
            {grpItems.map(item => (
              <div
                key={item.href}
                data-testid={`file-item-${item.href}`}
                aria-selected={item.href === selectedHref}
                role="option"
                onClick={() => onSelectItem(item)}
                style={{
                  padding: "0.3rem 0.75rem 0.3rem 1.25rem",
                  cursor: "pointer", fontSize: "0.85rem",
                  background: item.href === selectedHref
                    ? "var(--bg-overlay, #313244)" : "transparent",
                  borderLeft: item.href === selectedHref
                    ? "2px solid var(--blue, #89b4fa)" : "2px solid transparent",
                }}
              >
                {baseName(item.href)}
              </div>
            ))}
          </div>
        )
      })}
    </div>
  )
}
```

Then run:
```bash
cd ui && npm test -- FileTreePanel && cd ..
git add ui/src/components/FileTreePanel.tsx
git commit -m "R13b-T03: FileTreePanel + CodeMirror installed — all tree tests green"
```

---

## R13b-T04

Write `ui/src/components/EbookEditorShell.tsx`:
```tsx
import { useState, useEffect, useCallback } from "react"
import { invoke } from "@tauri-apps/api/core"
import { FileTreePanel } from "./FileTreePanel"
import { EditorView, basicSetup } from "codemirror"
import { html as htmlLang } from "@codemirror/lang-html"
import { css as cssLang } from "@codemirror/lang-css"
import { oneDark } from "@codemirror/theme-one-dark"
import { useRef } from "react"

interface Book {
  id: string
  title: string
  authors: string[]
  format: string
  cover_path: string | null
  progress_percent: number
  last_opened_at: string | null
  file_path?: string
}

interface ManifestItem { id?: string; href: string; media_type: string }
interface EditorMetadata { title?: string; authors: string[]; language?: string }

interface Props {
  book: Book
  onClose: () => void
}

export function EbookEditorShell({ book, onClose }: Props) {
  const [manifest,  setManifest]  = useState<ManifestItem[]>([])
  const [selected,  setSelected]  = useState<ManifestItem | null>(null)
  const [dirty,     setDirty]     = useState(false)
  const [saving,    setSaving]    = useState(false)
  const [loading,   setLoading]   = useState(true)
  const editorDivRef = useRef<HTMLDivElement>(null)
  const editorViewRef = useRef<EditorView | null>(null)

  useEffect(() => {
    if (!book.file_path) return
    invoke<{ spine: string[]; manifest: ManifestItem[]; metadata: EditorMetadata }>(
      "editor_open_epub", { bookId: book.id, filePath: book.file_path }
    ).then(result => {
      setManifest(result.manifest)
    }).catch(console.error)
    .finally(() => setLoading(false))

    return () => { invoke("editor_close", { bookId: book.id }).catch(console.error) }
  }, [book.id, book.file_path])

  const loadItem = useCallback(async (item: ManifestItem) => {
    setSelected(item)
    const content = await invoke<string>("editor_read_item", { bookId: book.id, href: item.href })
    if (editorViewRef.current) { editorViewRef.current.destroy() }
    if (!editorDivRef.current) return

    const lang = item.media_type.includes("css") ? cssLang() : htmlLang()
    const view = new EditorView({
      doc: content,
      extensions: [
        basicSetup, lang, oneDark,
        EditorView.updateListener.of(update => {
          if (update.docChanged) setDirty(true)
        }),
      ],
      parent: editorDivRef.current,
    })
    editorViewRef.current = view
  }, [book.id])

  async function handleSave() {
    if (!selected || !editorViewRef.current) return
    const content = editorViewRef.current.state.doc.toString()
    setSaving(true)
    try {
      await invoke("editor_write_item", { bookId: book.id, href: selected.href, content })
      await invoke("editor_save_epub",  { bookId: book.id })
      setDirty(false)
    } catch (e) {
      console.error("EbookEditorShell save error:", e)
    } finally {
      setSaving(false)
    }
  }

  return (
    <div style={{
      position: "fixed", inset: 0, background: "var(--bg-base, #1e1e2e)",
      display: "flex", flexDirection: "column", zIndex: 2000,
      color: "var(--text-primary, #cdd6f4)",
    }}>
      {/* Toolbar */}
      <div style={{
        display: "flex", alignItems: "center", gap: "0.75rem",
        padding: "0.5rem 1rem",
        borderBottom: "1px solid var(--border, #45475a)",
      }}>
        <button
          data-testid="editor-close-btn"
          onClick={onClose}
          style={{ padding: "0.3rem 0.75rem", background: "transparent", border: "none",
                   cursor: "pointer", color: "inherit", fontSize: "1.1rem" }}
        >
          ←
        </button>
        <span style={{ fontWeight: 600 }}>{book.title} — Editor</span>
        {dirty && <span style={{ color: "var(--yellow, #f9e2af)", fontSize: "0.85rem" }}>Unsaved changes</span>}
        <div style={{ flex: 1 }} />
        <button
          data-testid="editor-save-btn"
          onClick={handleSave}
          disabled={saving || !dirty}
          style={{
            padding: "0.4rem 1rem", background: "var(--blue, #89b4fa)",
            border: "none", borderRadius: "6px", cursor: "pointer",
            color: "#1e1e2e", fontWeight: 600,
            opacity: (saving || !dirty) ? 0.5 : 1,
          }}
        >
          {saving ? "Saving…" : "Save"}
        </button>
      </div>

      {/* Body */}
      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        {loading ? (
          <div style={{ padding: "2rem", color: "var(--text-muted, #6c7086)" }}>Opening EPUB…</div>
        ) : (
          <>
            <FileTreePanel
              items={manifest}
              selectedHref={selected?.href ?? null}
              onSelectItem={loadItem}
            />
            <div
              data-testid="editor-content-panel"
              ref={editorDivRef}
              style={{ flex: 1, overflow: "auto" }}
            />
          </>
        )}
      </div>
    </div>
  )
}
```

Then run:
```bash
cd ui && npm test -- EbookEditorShell && cd ..
git add ui/src/components/EbookEditorShell.tsx
git commit -m "R13b-T04: EbookEditorShell — all shell tests green"
```

---

## R13b-T05

Write `ui/src/components/MetadataEditorPanel.tsx`:
```tsx
import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Props {
  bookId: string
  initialTitle: string
  initialAuthors: string[]
  onSaved: () => void
}

export function MetadataEditorPanel({ bookId, initialTitle, initialAuthors, onSaved }: Props) {
  const [title,   setTitle]   = useState(initialTitle)
  const [authors, setAuthors] = useState(initialAuthors.join(", "))
  const [saving,  setSaving]  = useState(false)

  async function handleSave() {
    setSaving(true)
    try {
      await invoke("editor_update_metadata", {
        bookId,
        title,
        authors: authors.split(",").map(s => s.trim()).filter(Boolean),
      })
      onSaved()
    } finally { setSaving(false) }
  }

  return (
    <div style={{ padding: "1rem", display: "flex", flexDirection: "column", gap: "0.75rem" }}>
      <h3 style={{ margin: 0 }}>Metadata</h3>
      <label>
        Title
        <input
          data-testid="meta-title-input"
          value={title}
          onChange={e => setTitle(e.target.value)}
          style={{ display: "block", width: "100%", marginTop: "0.25rem",
                   padding: "0.4rem", background: "var(--bg-overlay, #313244)",
                   border: "1px solid var(--border, #45475a)", borderRadius: "4px",
                   color: "inherit", boxSizing: "border-box" }}
        />
      </label>
      <label>
        Authors (comma-separated)
        <input
          data-testid="meta-authors-input"
          value={authors}
          onChange={e => setAuthors(e.target.value)}
          style={{ display: "block", width: "100%", marginTop: "0.25rem",
                   padding: "0.4rem", background: "var(--bg-overlay, #313244)",
                   border: "1px solid var(--border, #45475a)", borderRadius: "4px",
                   color: "inherit", boxSizing: "border-box" }}
        />
      </label>
      <button
        data-testid="meta-save-btn"
        onClick={handleSave}
        disabled={saving}
        style={{
          alignSelf: "flex-end", padding: "0.4rem 1rem",
          background: "var(--blue, #89b4fa)", border: "none",
          borderRadius: "6px", cursor: "pointer", color: "#1e1e2e", fontWeight: 600,
        }}
      >
        {saving ? "Saving…" : "Save Metadata"}
      </button>
    </div>
  )
}
```

Write `ui/src/components/CoverEditorPanel.tsx`:
```tsx
import { useState, useRef } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Props {
  bookId: string
  onSaved: () => void
}

export function CoverEditorPanel({ bookId, onSaved }: Props) {
  const [preview, setPreview] = useState<string | null>(null)
  const [saving,  setSaving]  = useState(false)
  const [error,   setError]   = useState<string | null>(null)
  const fileRef = useRef<HTMLInputElement>(null)

  async function handleFile(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0]
    if (!file) return
    if (!file.type.startsWith("image/")) { setError("Please select an image file."); return }
    const url = URL.createObjectURL(file)
    setPreview(url)
    setError(null)

    const buffer = await file.arrayBuffer()
    const bytes  = new Uint8Array(buffer)
    const b64    = btoa(String.fromCharCode(...bytes))

    setSaving(true)
    try {
      await invoke("editor_set_cover", { bookId, dataB64: b64, mimeType: file.type })
      await invoke("editor_save_epub",  { bookId })
      onSaved()
    } catch (e) {
      setError(String(e))
    } finally { setSaving(false) }
  }

  return (
    <div style={{ padding: "1rem", display: "flex", flexDirection: "column", gap: "0.75rem" }}>
      <h3 style={{ margin: 0 }}>Cover Image</h3>
      {preview && (
        <img
          src={preview}
          alt="New cover preview"
          data-testid="cover-preview"
          style={{ maxWidth: "200px", maxHeight: "280px", borderRadius: "4px" }}
        />
      )}
      {error && <span style={{ color: "var(--red, #f38ba8)", fontSize: "0.85rem" }}>{error}</span>}
      <input
        ref={fileRef}
        type="file"
        accept="image/jpeg,image/png,image/webp"
        onChange={handleFile}
        style={{ display: "none" }}
        data-testid="cover-file-input"
      />
      <button
        data-testid="cover-select-btn"
        onClick={() => fileRef.current?.click()}
        disabled={saving}
        style={{
          padding: "0.4rem 1rem",
          background: "var(--bg-overlay, #313244)",
          border: "1px solid var(--border, #45475a)",
          borderRadius: "6px", cursor: "pointer", color: "inherit",
        }}
      >
        {saving ? "Saving…" : "Choose Cover Image…"}
      </button>
    </div>
  )
}
```

```bash
cd ui && npm test && cd ..
git add ui/src/components/MetadataEditorPanel.tsx ui/src/components/CoverEditorPanel.tsx
git commit -m "R13b-T05: MetadataEditorPanel + CoverEditorPanel components"
```

---

## R13b-T06 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm test && cd ..
```

**Visual inspection:**
1. Launch the app: `cd src-tauri && cargo tauri dev`
2. Right-click an EPUB book → "Edit…" to open EbookEditorShell
3. Verify file tree shows grouped items (Text, Stylesheets, Images)
4. Click a `.xhtml` item — verify CodeMirror editor loads with HTML content
5. Make a text change — verify "Unsaved changes" appears in toolbar
6. Click Save — verify file is updated (reopen in editor to confirm)
7. Switch to Metadata tab — change title → Save Metadata → close → reopen, verify new title
8. Switch to Cover tab — select a JPEG — verify preview appears and cover is saved

```bash
git add -A
git commit -m "R13b-T06: RMP-13 Ebook Editor Level C — all tests green, UI wired"
```
