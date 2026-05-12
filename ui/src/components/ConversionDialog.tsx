import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"

type OutputFormat = "TXT" | "HTML" | "DOCX" | "PDF" | "MOBI" | "FB2" | "RTF" | "HTMLZ"

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

interface Props {
  book: Book & { file_path?: string }
  onClose: () => void
}

export function ConversionDialog({ book, onClose }: Props) {
  const [format, setFormat]   = useState<OutputFormat>("TXT")
  const [loading, setLoading] = useState(false)
  const [outputPath, setOutputPath] = useState<string | null>(null)
  const [error, setError]     = useState<string | null>(null)

  async function handleConvert() {
    if (!book.file_path) {
      setError("Book file path is not available.")
      return
    }
    setLoading(true)
    setError(null)
    try {
      const path = await invoke<string>("convert_book", {
        epubPath:     book.file_path,
        outputFormat: format,
        outputDir:    null,
      })
      setOutputPath(path)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Convert book"
      style={{
        position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)",
        display: "flex", alignItems: "center", justifyContent: "center",
        zIndex: 1000,
      }}
    >
      <div style={{
        background: "var(--bg-surface, #1e1e2e)",
        borderRadius: "12px", padding: "2rem", minWidth: "360px",
        color: "var(--text-primary, #cdd6f4)",
      }}>
        <h2 style={{ margin: "0 0 1.5rem", fontSize: "1.2rem" }}>
          Convert "{book.title}"
        </h2>

        <label style={{ display: "block", marginBottom: "0.5rem", fontSize: "0.9rem" }}>
          Output format
        </label>
        <select
          data-testid="output-format-select"
          value={format}
          onChange={e => setFormat(e.target.value as OutputFormat)}
          style={{
            width: "100%", padding: "0.5rem",
            background: "var(--bg-overlay, #313244)",
            color: "inherit", border: "1px solid var(--border, #45475a)",
            borderRadius: "6px", marginBottom: "1.5rem",
          }}
        >
          <option value="TXT">TXT</option>
          <option value="HTML">HTML</option>
          <option value="DOCX">DOCX</option>
          <option value="PDF">PDF</option>
          <option value="MOBI">MOBI</option>
          <option value="FB2">FB2</option>
          <option value="RTF">RTF</option>
          <option value="HTMLZ">HTMLZ</option>
        </select>

        {error && (
          <p style={{ color: "var(--red, #f38ba8)", marginBottom: "1rem", fontSize: "0.9rem" }}>
            {error}
          </p>
        )}

        {outputPath && (
          <div
            data-testid="conversion-success"
            style={{
              background: "var(--green-dim, #1e3a2a)", borderRadius: "6px",
              padding: "0.75rem", marginBottom: "1rem", fontSize: "0.9rem",
            }}
          >
            Saved to: <code style={{ wordBreak: "break-all" }}>{outputPath}</code>
          </div>
        )}

        <div style={{ display: "flex", gap: "0.75rem", justifyContent: "flex-end" }}>
          <button
            onClick={onClose}
            style={{
              padding: "0.5rem 1.25rem",
              background: "var(--bg-overlay, #313244)",
              border: "none", borderRadius: "6px", cursor: "pointer",
              color: "inherit",
            }}
          >
            {outputPath ? "Close" : "Cancel"}
          </button>
          {!outputPath && (
            <button
              data-testid="convert-btn"
              onClick={handleConvert}
              disabled={loading}
              style={{
                padding: "0.5rem 1.25rem",
                background: "var(--blue, #89b4fa)",
                border: "none", borderRadius: "6px", cursor: "pointer",
                color: "#1e1e2e", fontWeight: 600,
                opacity: loading ? 0.6 : 1,
              }}
            >
              {loading ? "Converting…" : "Convert"}
            </button>
          )}
        </div>
      </div>
    </div>
  )
}
