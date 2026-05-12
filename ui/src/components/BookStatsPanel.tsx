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
  file_path?: string
}

interface BookStats {
  word_count:           number
  character_count:      number
  page_count_estimate:  number
  reading_time_minutes: number
}

interface Props { book: Book }

function fmtNum(n: number)   { return n.toLocaleString() }
function fmtTime(mins: number) {
  const h = Math.floor(mins / 60)
  const m = mins % 60
  return h > 0 ? `${h}h ${m}m` : `${m}m`
}

export function BookStatsPanel({ book }: Props) {
  const [stats,   setStats]   = useState<BookStats | null>(null)
  const [loading, setLoading] = useState(true)
  const [error,   setError]   = useState<string | null>(null)

  useEffect(() => {
    if (!book.file_path) { setLoading(false); return }
    invoke<BookStats>("get_book_stats", { filePath: book.file_path })
      .then(setStats)
      .catch(e => setError(String(e)))
      .finally(() => setLoading(false))
  }, [book.file_path])

  if (loading) {
    return <div data-testid="stats-loading" style={{ color: "var(--text-muted, #6c7086)", fontSize: "0.9rem" }}>Calculating…</div>
  }
  if (error) {
    return <div style={{ color: "var(--red, #f38ba8)", fontSize: "0.85rem" }}>Could not load stats.</div>
  }
  if (!stats) return null

  const rows = [
    { testId: "stat-word-count",    label: "Words",        value: fmtNum(stats.word_count) },
    { testId: "stat-char-count",    label: "Characters",   value: fmtNum(stats.character_count) },
    { testId: "stat-page-count",    label: "Est. Pages",   value: fmtNum(stats.page_count_estimate) },
    { testId: "stat-reading-time",  label: "Reading Time", value: fmtTime(stats.reading_time_minutes) },
  ]

  return (
    <section>
      <h3 style={{ margin: "0 0 0.75rem", fontSize: "1rem", fontWeight: 600 }}>Book Statistics</h3>
      <dl style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "0.5rem 1rem" }}>
        {rows.map(r => (
          <div key={r.testId} style={{ display: "contents" }}>
            <dt style={{ color: "var(--text-muted, #6c7086)", fontSize: "0.85rem" }}>{r.label}</dt>
            <dd data-testid={r.testId} style={{ margin: 0, fontWeight: 600 }}>{r.value}</dd>
          </div>
        ))}
      </dl>
    </section>
  )
}
