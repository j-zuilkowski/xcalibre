import { useState, useRef, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Book {
  id: string; title: string; authors: string[]
  format: string; cover_path: string | null
  progress_percent: number; last_opened_at: string | null
}

interface Message { role: "user" | "assistant"; content: string }

const QUICK_ACTIONS = [
  { id: "summarize", label: "Summarize", prompt: "Give me a concise summary of this book." },
  { id: "chapters",  label: "Chapters",  prompt: "What are the main chapters or sections?" },
  { id: "read_next", label: "Read Next", prompt: "Based on this book, what should I read next?" },
  { id: "universe",  label: "Universe",  prompt: "Tell me about the world and universe of this book." },
  { id: "series",    label: "Series",    prompt: "Is this part of a series? What's the reading order?" },
]

interface Props { book: Book; onClose: () => void }

export function AIChatPanel({ book, onClose }: Props) {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState("")
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => { bottomRef.current?.scrollIntoView({ behavior: "smooth" }) }, [messages])

  const send = async (query: string) => {
    if (!query.trim() || loading) return
    const userMsg: Message = { role: "user", content: query }
    const nextMessages = [...messages, userMsg]
    setMessages(nextMessages)
    setInput("")
    setLoading(true)
    setError(null)
    try {
      const history = nextMessages.map(m => ({ role: m.role, content: m.content }))
      const resp = await invoke<{ content: string }>("ai_chat", {
        bookId: book.id, query, history,
      })
      setMessages(prev => [...prev, { role: "assistant", content: resp.content }])
    } catch (e) { setError(String(e)) }
    finally { setLoading(false) }
  }

  return (
    <div data-testid="ai-chat-panel" className="flex flex-col h-full bg-white dark:bg-gray-900">
      <div className="flex items-center justify-between px-4 py-3 border-b dark:border-gray-700">
        <h3 className="font-semibold text-sm dark:text-white">AI · {book.title}</h3>
        <button data-testid="ai-panel-close" onClick={onClose} className="text-gray-400 hover:text-gray-600 text-lg">×</button>
      </div>

      {messages.length === 0 && (
        <div className="p-4 flex flex-wrap gap-2">
          {QUICK_ACTIONS.map(a => (
            <button key={a.id} data-testid={`quick-action-${a.id}`} onClick={() => send(a.prompt)}
              className="text-xs px-3 py-1.5 bg-blue-50 dark:bg-blue-900/30 text-blue-600
                         dark:text-blue-400 rounded-full border border-blue-200 dark:border-blue-800
                         hover:bg-blue-100 transition-colors">
              {a.label}
            </button>
          ))}
        </div>
      )}

      <div className="flex-1 overflow-y-auto px-4 py-2 space-y-4">
        {messages.map((m, i) => (
          <div key={i} className={`flex ${m.role === "user" ? "justify-end" : "justify-start"}`}>
            <div className={`max-w-[85%] rounded-xl px-3 py-2 text-sm
              ${m.role === "user"
                ? "bg-blue-600 text-white"
                : "bg-gray-100 dark:bg-gray-800 dark:text-gray-200"}`}>
              {m.content}
            </div>
          </div>
        ))}
        {loading && (
          <div className="flex justify-start">
            <div className="bg-gray-100 dark:bg-gray-800 rounded-xl px-3 py-2 text-sm text-gray-500">
              Thinking…
            </div>
          </div>
        )}
        {error && <p className="text-red-500 text-xs">{error}</p>}
        <div ref={bottomRef}/>
      </div>

      <div className="px-4 py-3 border-t dark:border-gray-700 flex gap-2">
        <input
          value={input} onChange={e => setInput(e.target.value)}
          onKeyDown={e => e.key === "Enter" && send(input)}
          placeholder="Ask about this book…"
          className="flex-1 text-sm border rounded-lg px-3 py-2 dark:bg-gray-800 dark:text-white
                     dark:border-gray-600 outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button data-testid="send-message-btn" onClick={() => send(input)} disabled={loading}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium
                     hover:bg-blue-700 disabled:opacity-50">
          Send
        </button>
      </div>
    </div>
  )
}
