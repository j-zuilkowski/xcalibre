import { useRef, useState } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Props {
  value: string
  onChange: (value: string) => void
  placeholder?: string
  className?: string
  rows?: number
}

interface Suggestion { word: string; suggestions: string[] }

export function SpellCheckInput({ value, onChange, placeholder, className = "", rows = 4 }: Props) {
  const [popup, setPopup] = useState<{ x: number; y: number; data: Suggestion } | null>(null)
  const ref = useRef<HTMLTextAreaElement>(null)

  const handleContextMenu = async (e: React.MouseEvent) => {
    e.preventDefault()
    const ta = ref.current
    if (!ta) return
    const selStart = ta.selectionStart
    const selEnd = ta.selectionEnd
    const selected = value.slice(selStart, selEnd).trim()
    const word = selected || getWordAt(value, selStart)
    if (!word) return
    const result = await invoke<{ word: string; is_correct: boolean }>("check_word", { word })
    if (result.is_correct) return
    const sugg = await invoke<Suggestion>("get_suggestions", { word })
    setPopup({ x: e.clientX, y: e.clientY, data: sugg })
  }

  const applysuggestion = (suggestion: string) => {
    if (!popup) return
    const newValue = value.replace(popup.data.word, suggestion)
    onChange(newValue)
    setPopup(null)
  }

  const addToDictionary = async () => {
    if (!popup) return
    await invoke("add_to_dictionary", { word: popup.data.word })
    setPopup(null)
  }

  return (
    <div className="relative">
      <textarea
        ref={ref}
        value={value}
        onChange={e => onChange(e.target.value)}
        onContextMenu={handleContextMenu}
        placeholder={placeholder}
        rows={rows}
        className={`w-full border rounded px-3 py-2 text-sm dark:bg-gray-700 dark:text-white resize-y ${className}`}
      />
      {popup && (
        <div
          data-testid="spell-suggestions"
          style={{ position: "fixed", left: popup.x, top: popup.y, zIndex: 9999 }}
          className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-600
                     rounded shadow-lg min-w-32 py-1"
        >
          {popup.data.suggestions.slice(0, 5).map(s => (
            <button key={s} onClick={() => applysuggestion(s)}
              className="w-full text-left px-4 py-1.5 text-sm hover:bg-gray-50
                         dark:hover:bg-gray-700 dark:text-white">
              {s}
            </button>
          ))}
          <hr className="my-1 border-gray-200 dark:border-gray-600"/>
          <button onClick={addToDictionary}
            className="w-full text-left px-4 py-1.5 text-sm text-gray-500 hover:bg-gray-50">
            Add to Dictionary
          </button>
          <button onClick={() => setPopup(null)}
            className="w-full text-left px-4 py-1.5 text-sm text-gray-400 hover:bg-gray-50">
            Dismiss
          </button>
        </div>
      )}
    </div>
  )
}

function getWordAt(text: string, pos: number): string {
  const before = text.slice(0, pos).match(/\S+$/) ?? [""]
  const after = text.slice(pos).match(/^\S+/) ?? [""]
  return before[0] + after[0]
}
