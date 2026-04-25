import { invoke } from "@tauri-apps/api/core"

export type HighlightColor = "yellow" | "green" | "blue" | "pink" | "purple"

export interface Annotation {
  id: string
  book_id: string
  type: "highlight" | "note" | "bookmark"
  cfi: string
  selected_text: string | null
  note: string | null
  color: HighlightColor
  synced: boolean
  created_at: string
}

/** Apply saved annotations to the reader iframe document. */
export function applyHighlights(
  iframeDoc: Document,
  annotations: Annotation[],
): void {
  for (const ann of annotations) {
    if (ann.type !== "highlight" || !ann.selected_text) continue
    try {
      // Use the browser's find API to locate and wrap the selected text.
      // This is a best-effort approach; CFI-based restoration is preferred
      // but requires a full CFI library integration.
      const walker = iframeDoc.createTreeWalker(
        iframeDoc.body,
        NodeFilter.SHOW_TEXT,
      )
      let node: Node | null
      while ((node = walker.nextNode())) {
        const text = node.textContent ?? ""
        const idx = text.indexOf(ann.selected_text)
        if (idx !== -1 && node.parentElement) {
          const range = iframeDoc.createRange()
          range.setStart(node, idx)
          range.setEnd(node, idx + ann.selected_text.length)
          const mark = iframeDoc.createElement("mark")
          mark.dataset.annotationId = ann.id
          mark.style.backgroundColor = colorToCSS(ann.color)
          mark.style.cursor = "pointer"
          range.surroundContents(mark)
          break
        }
      }
    } catch {
      // Skip annotations that cannot be applied (e.g., across element boundaries)
    }
  }
}

/** Listen for text selection in the reader iframe and prompt to highlight. */
export function setupHighlightListener(
  iframeDoc: Document,
  bookId: string,
  currentCfi: string,
  onAnnotationCreated: () => void,
): () => void {
  const handler = async () => {
    const selection = iframeDoc.getSelection()
    if (!selection || selection.isCollapsed) return
    const text = selection.toString().trim()
    if (text.length < 3) return

    const color: HighlightColor = "yellow"
    await invoke("create_annotation", {
      bookId,
      annotationType: "highlight",
      cfi: currentCfi,
      selectedText: text,
      note: null,
      color,
    }).catch(console.error)

    selection.removeAllRanges()
    onAnnotationCreated()
  }

  iframeDoc.addEventListener("mouseup", handler)
  return () => iframeDoc.removeEventListener("mouseup", handler)
}

function colorToCSS(color: HighlightColor): string {
  const map: Record<HighlightColor, string> = {
    yellow: "rgba(255,255,0,0.4)",
    green: "rgba(0,255,0,0.3)",
    blue: "rgba(0,180,255,0.3)",
    pink: "rgba(255,100,180,0.3)",
    purple: "rgba(180,0,255,0.3)",
  }
  return map[color] ?? map.yellow
}
