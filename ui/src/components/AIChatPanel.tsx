import { useState, useRef, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { CitedResponseView } from "./CitedResponseView"
import { ReasoningBudgetSelector } from "./ReasoningBudgetSelector"

interface Book {
  id: string; title: string; authors: string[]
  format: string; cover_path: string | null
  progress_percent: number; last_opened_at: string | null
}

interface Citation { chunk_index: number; chunk_text: string; relevance_score: number }

interface Message {
  role: "user" | "assistant"
  content: string
  citations?: Citation[]
}

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
  const [reasoningBudget, setReasoningBudget] = useState("none")
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
      const resp = await invoke<{ content: string; citations: Citation[] }>("ai_chat", {
        bookId: book.id, query, history, reasoningBudget,
      })
      setMessages(prev => [...prev, {
        role: "assistant",
        content: resp.content,
        citations: resp.citations ?? [],
      }])
    } catch (e) { setError(String(e)) }
    finally { setLoading(false) }
  }

  const saveAsNote = async (content: string, query: string) => {
    try {
      await invoke("save_ai_response_as_note_cmd", {
        bookId: book.id,
        title: `AI: ${query.slice(0, 50)}`,
        bodyHtml: `<p>${content}</p>`,
        bodyText: content,
      })
    } catch (e) { console.error("save note failed:", e) }
  }

  return (
    <div data-testid="ai-chat-panel" className="flex flex-col h-full bg-white dark:bg-gray-900">
      <div className="flex items-center justify-between px-4 py-3 border-b dark:border-gray-700">
        <h3 className="font-semibold text-sm dark:text-white">AI · {book.title}</h3>
        <div className="flex items-center gap-2">
          <ReasoningBudgetSelector value={reasoningBudget} onChange={setReasoningBudget} />
          <button data-testid="ai-panel-close" onClick={onClose} className="text-gray-400 hover:text-gray-600 text-lg">×</button>
        </div>
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
            {m.role === "user" ? (
              <div className="max-w-[85%] rounded-xl px-3 py-2 text-sm bg-blue-600 text-white">
                {m.content}
              </div>
            ) : (
              <div className="max-w-[90%] rounded-xl px-3 py-2 text-sm
                              bg-gray-100 dark:bg-gray-800 dark:text-gray-200">
                <CitedResponseView cited={{ response_text: m.content, citations: m.citations ?? [] }} />
                <button
                  onClick={() => saveAsNote(m.content, messages[i - 1]?.content ?? "AI response")}
                  style={{
                    marginTop: "0.4rem", fontSize: "0.75rem", padding: "0.2rem 0.5rem",
                    background: "transparent", border: "1px solid var(--border, #45475a)",
                    borderRadius: "4px", cursor: "pointer", color: "var(--text-muted, #6c7086)",
                  }}
                >
                  Save as Note
                </button>
              </div>
            )}
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
