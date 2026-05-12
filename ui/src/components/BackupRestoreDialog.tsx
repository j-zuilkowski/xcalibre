import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Props {
  onClose: () => void
  onPickExportPath?: () => Promise<string | null>
  onPickRestorePath?: () => Promise<string | null>
}

type Tab = "backup" | "restore"

async function defaultPickExport(): Promise<string | null> {
  try {
    const { save } = await import("@tauri-apps/plugin-dialog")
    return await save({
      defaultPath: `xcalibre-backup-${new Date().toISOString().slice(0,10)}.xcalibre`,
      filters: [{ name: "xCalibre Backup", extensions: ["xcalibre"] }],
    })
  } catch { return null }
}

async function defaultPickRestore(): Promise<string | null> {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog")
    const result = await open({
      filters: [{ name: "xCalibre Backup", extensions: ["xcalibre"] }],
    })
    return Array.isArray(result) ? null : result
  } catch { return null }
}

export function BackupRestoreDialog({
  onClose,
  onPickExportPath  = defaultPickExport,
  onPickRestorePath = defaultPickRestore,
}: Props) {
  const [tab,           setTab]       = useState<Tab>("backup")
  const [loading,       setLoading]   = useState(false)
  const [backupSuccess, setBkSuccess] = useState<string | null>(null)
  const [restoreResult, setRsResult]  = useState<{ books_restored: number } | null>(null)
  const [error,         setError]     = useState<string | null>(null)
  const [includeFiles,  setIncFiles]  = useState(false)

  async function handleExport() {
    const path = await onPickExportPath()
    if (!path) return
    setLoading(true); setError(null)
    try {
      const out = await invoke<string>("export_library_backup_cmd", {
        includeFiles, outPath: path,
      })
      setBkSuccess(out)
    } catch (e) { setError(String(e)) }
    finally { setLoading(false) }
  }

  async function handleRestore() {
    const path = await onPickRestorePath()
    if (!path) return
    setLoading(true); setError(null)
    try {
      const result = await invoke<{ books_restored: number }>("restore_library_backup_cmd", {
        backupPath: path,
      })
      setRsResult(result)
    } catch (e) { setError(String(e)) }
    finally { setLoading(false) }
  }

  return (
    <div role="dialog" aria-modal="true" style={{
      position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)",
      display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000,
    }}>
      <div style={{
        background: "var(--bg-surface, #1e1e2e)", borderRadius: "12px",
        padding: "2rem", minWidth: "400px", color: "var(--text-primary, #cdd6f4)",
      }}>
        <h2 style={{ margin: "0 0 1.5rem" }}>Backup & Restore</h2>

        <div style={{ display: "flex", gap: "0.5rem", marginBottom: "1.5rem" }}>
          {(["backup", "restore"] as Tab[]).map(t => (
            <button
              key={t}
              data-testid={`${t}-tab`}
              onClick={() => { setTab(t); setError(null); setBkSuccess(null); setRsResult(null) }}
              style={{
                padding: "0.4rem 1rem",
                background: tab === t ? "var(--blue, #89b4fa)" : "var(--bg-overlay, #313244)",
                border: "none", borderRadius: "6px", cursor: "pointer",
                color: tab === t ? "#1e1e2e" : "inherit", fontWeight: tab === t ? 600 : 400,
              }}
            >
              {t === "backup" ? "Export Backup" : "Restore"}
            </button>
          ))}
        </div>

        {error && (
          <p style={{ color: "var(--red, #f38ba8)", fontSize: "0.9rem", marginBottom: "1rem" }}>
            {error}
          </p>
        )}

        {tab === "backup" && (
          <>
            <label style={{ display: "flex", alignItems: "center", gap: "0.5rem", marginBottom: "1rem" }}>
              <input
                type="checkbox"
                checked={includeFiles}
                onChange={e => setIncFiles(e.target.checked)}
              />
              Include book files (larger backup)
            </label>
            {backupSuccess && (
              <div data-testid="backup-success" style={{
                background: "var(--green-dim, #1e3a2a)", borderRadius: "6px",
                padding: "0.75rem", marginBottom: "1rem", fontSize: "0.9rem",
              }}>
                Backup saved to: <code style={{ wordBreak: "break-all" }}>{backupSuccess}</code>
              </div>
            )}
            <button
              data-testid="export-backup-btn"
              onClick={handleExport}
              disabled={loading}
              style={{
                width: "100%", padding: "0.6rem",
                background: "var(--blue, #89b4fa)", border: "none",
                borderRadius: "6px", cursor: "pointer", color: "#1e1e2e",
                fontWeight: 600, opacity: loading ? 0.6 : 1,
              }}
            >
              {loading ? "Exporting…" : "Choose Location & Export"}
            </button>
          </>
        )}

        {tab === "restore" && (
          <>
            {restoreResult && (
              <div data-testid="restore-success" style={{
                background: "var(--green-dim, #1e3a2a)", borderRadius: "6px",
                padding: "0.75rem", marginBottom: "1rem", fontSize: "0.9rem",
              }}>
                Restored <strong>{restoreResult.books_restored}</strong> book(s) successfully.
              </div>
            )}
            <p style={{ fontSize: "0.9rem", color: "var(--text-muted, #6c7086)", marginBottom: "1rem" }}>
              Select a .xcalibre backup file to restore. Existing books will not be duplicated.
            </p>
            <button
              data-testid="restore-backup-btn"
              onClick={handleRestore}
              disabled={loading}
              style={{
                width: "100%", padding: "0.6rem",
                background: "var(--blue, #89b4fa)", border: "none",
                borderRadius: "6px", cursor: "pointer", color: "#1e1e2e",
                fontWeight: 600, opacity: loading ? 0.6 : 1,
              }}
            >
              {loading ? "Restoring…" : "Choose Backup File & Restore"}
            </button>
          </>
        )}

        <div style={{ display: "flex", justifyContent: "flex-end", marginTop: "1.5rem" }}>
          <button
            onClick={onClose}
            style={{
              padding: "0.4rem 1rem", background: "var(--bg-overlay, #313244)",
              border: "none", borderRadius: "6px", cursor: "pointer", color: "inherit",
            }}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  )
}
