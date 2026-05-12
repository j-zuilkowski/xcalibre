import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Book {
  id: string
  title: string
  authors: string[]
  format: string
  cover_path: string | null
  progress_percent: number
  last_opened_at: string | null
}

interface SimilarBook {
  id: string
  title: string
  authors: string
  score: number
}

interface Props {
  book: Book
  onOpenBook: (id: string) => void
}

export function SimilarBooksPanel({ book, onOpenBook }: Props) {
  const [similar, setSimilar] = useState<SimilarBook[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    invoke<SimilarBook[]>("find_similar_books_cmd", { bookId: book.id, limit: 8 })
      .then(setSimilar)
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [book.id])

  return (
    <section>
      <h3 data-testid="similar-books-header" style={{ margin: "0 0 0.75rem", fontSize: "1rem", fontWeight: 600 }}>
        Similar Books
      </h3>
      {loading && <p style={{ color: "var(--text-muted, #6c7086)", fontSize: "0.9rem" }}>Loading…</p>}
      {!loading && similar.length === 0 && (
        <p data-testid="similar-books-empty" style={{ color: "var(--text-muted, #6c7086)", fontSize: "0.9rem" }}>
          No similar books found in your library.
        </p>
      )}
      {!loading && similar.length > 0 && (
        <ul style={{ listStyle: "none", padding: 0, margin: 0 }}>
          {similar.map(s => (
            <li key={s.id} onClick={() => onOpenBook(s.id)}
              style={{ padding: "0.5rem 0.75rem", borderRadius: "6px", cursor: "pointer", marginBottom: "0.25rem" }}
              onMouseEnter={e => (e.currentTarget.style.background = "var(--bg-overlay, #313244)")}
              onMouseLeave={e => (e.currentTarget.style.background = "transparent")}>
              <div style={{ fontWeight: 500 }}>{s.title}</div>
              <div style={{ fontSize: "0.8rem", color: "var(--text-muted, #6c7086)" }}>{s.authors}</div>
            </li>
          ))}
        </ul>
      )}
    </section>
  )
}
