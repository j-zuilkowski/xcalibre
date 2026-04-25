import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Annotation } from "../reader/highlights"

interface Props {
  bookId: string
  onJumpTo: (cfi: string) => void
}

const COLOR_LABELS: Record<string, string> = {
  yellow: "🟡", green: "🟢", blue: "🔵", pink: "🩷", purple: "🟣",
}

export function AnnotationsSidebar({ bookId, onJumpTo }: Props) {
  const [annotations, setAnnotations] = useState<Annotation[]>([])
  const [editingId, setEditingId] = useState<string | null>(null)
  const [noteText, setNoteText] = useState("")

  const load = () =>
    invoke<Annotation[]>("get_annotations", { bookId })
      .then(setAnnotations)
      .catch(() => {})

  useEffect(() => { load() }, [bookId])

  const remove = async (id: string) => {
    await invoke("delete_annotation", { id }).catch(console.error)
    load()
    window.dispatchEvent(new Event("xcalibre-annotations-updated"))
  }

  const saveNote = async (id: string) => {
    await invoke("update_annotation_note", { id, note: noteText }).catch(console.error)
    setEditingId(null)
    load()
    window.dispatchEvent(new Event("xcalibre-annotations-updated"))
  }

  if (annotations.length === 0) {
    return (
      <div className="p-4 text-sm text-gray-400 dark:text-gray-500">
        No annotations yet. Select text in the reader to highlight.
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-2 p-3 overflow-y-auto">
      <p className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
        Annotations ({annotations.length})
      </p>
      {annotations.map((ann) => (
        <div
          key={ann.id}
          className="rounded-lg border border-gray-200 dark:border-gray-700 p-2 bg-white dark:bg-gray-800 text-sm"
        >
          <div className="flex items-start justify-between gap-1">
            <button
              onClick={() => onJumpTo(ann.cfi)}
              className="flex-1 text-left text-gray-700 dark:text-gray-300 hover:text-blue-500 truncate"
              title="Jump to location"
            >
              {COLOR_LABELS[ann.color] ?? "📌"}{" "}
              {ann.selected_text ?? ann.type}
            </button>
            <button
              onClick={() => remove(ann.id)}
              className="text-gray-400 hover:text-red-500 shrink-0 text-xs"
            >
              ✕
            </button>
          </div>

          {ann.note && editingId !== ann.id && (
            <p
              className="mt-1 text-xs text-gray-500 dark:text-gray-400 cursor-pointer"
              onClick={() => { setEditingId(ann.id); setNoteText(ann.note ?? "") }}
            >
              {ann.note}
            </p>
          )}

          {editingId === ann.id ? (
            <div className="mt-1 flex gap-1">
              <input
                value={noteText}
                onChange={(e) => setNoteText(e.target.value)}
                className="flex-1 text-xs border rounded px-1 py-0.5 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
              />
              <button onClick={() => saveNote(ann.id)} className="text-xs px-1.5 bg-blue-500 text-white rounded">✓</button>
              <button onClick={() => setEditingId(null)} className="text-xs px-1.5 bg-gray-200 dark:bg-gray-600 rounded dark:text-white">✕</button>
            </div>
          ) : (
            !ann.note && (
              <button
                onClick={() => { setEditingId(ann.id); setNoteText("") }}
                className="mt-1 text-xs text-blue-400 hover:text-blue-500"
              >
                + Add note
              </button>
            )
          )}
        </div>
      ))}
    </div>
  )
}
