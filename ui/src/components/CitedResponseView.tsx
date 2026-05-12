import { useState } from "react"

interface Citation {
  chunk_index:     number
  chunk_text:      string
  relevance_score: number
}

interface CitedResponse {
  response_text: string
  citations:     Citation[]
}

interface Props { cited: CitedResponse }

export function CitedResponseView({ cited }: Props) {
  const [showCitations, setShowCitations] = useState(true)
  const { response_text, citations } = cited

  return (
    <div>
      <p style={{ margin: "0 0 0.5rem", lineHeight: 1.6 }}>{response_text}</p>

      {citations.length > 0 && (
        <div>
          <button
            onClick={() => setShowCitations(s => !s)}
            style={{
              padding: "0.25rem 0.5rem", background: "transparent",
              border: "1px solid var(--border, #45475a)", borderRadius: "4px",
              cursor: "pointer", color: "var(--text-muted, #6c7086)",
              fontSize: "0.8rem", display: "flex", alignItems: "center", gap: "0.4rem",
            }}
          >
            <span data-testid="citation-count">{citations.length}</span>
            {citations.length === 1 ? "source" : "sources"}
            {showCitations ? " ▲" : " ▼"}
          </button>

          {showCitations && (
            <ul style={{
              listStyle: "none", padding: 0, margin: "0.5rem 0 0",
              borderLeft: "2px solid var(--border, #45475a)",
              paddingLeft: "0.75rem",
            }}>
              {citations.map((c, i) => (
                <li key={i} style={{
                  fontSize: "0.82rem", color: "var(--text-muted, #6c7086)",
                  marginBottom: "0.4rem", fontStyle: "italic",
                }}>
                  "{c.chunk_text.slice(0, 120)}{c.chunk_text.length > 120 ? "…" : ""}"
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  )
}
