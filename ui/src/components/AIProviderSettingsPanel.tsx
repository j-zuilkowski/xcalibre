import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"

const PROVIDERS = [
  { value: "ollama", label: "Ollama", needsKey: false, defaultUrl: "http://localhost:11434" },
  { value: "openai", label: "OpenAI", needsKey: true, defaultUrl: "https://api.openai.com/v1" },
  { value: "gemini", label: "Google Gemini", needsKey: true, defaultUrl: "https://generativelanguage.googleapis.com/v1beta" },
  { value: "lmstudio", label: "LM Studio", needsKey: false, defaultUrl: "http://localhost:1234" },
  { value: "openrouter", label: "OpenRouter", needsKey: true, defaultUrl: "https://openrouter.ai/api/v1" },
]

interface Props { onClose: () => void }

export function AIProviderSettingsPanel({ onClose }: Props) {
  const [provider, setProvider] = useState("ollama")
  const [baseUrl, setBaseUrl] = useState("http://localhost:11434")
  const [apiKey, setApiKey] = useState("")
  const [model, setModel] = useState("")
  const [embedModel, setEmbedModel] = useState("")
  const [models, setModels] = useState<string[]>([])
  const [testResult, setTestResult] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    invoke<any>("get_ai_config_cmd").then(cfg => {
      if (cfg) {
        setProvider(cfg.provider); setBaseUrl(cfg.base_url)
        setApiKey(cfg.has_api_key ? "••••••••" : ""); setModel(cfg.model)
        setEmbedModel(cfg.embed_model)
      }
    }).catch(console.error).finally(() => setLoading(false))
  }, [])

  async function handleTestConnection() {
    setTestResult(null)
    try {
      const list = await invoke<string[]>("ai_list_models")
      setModels(list)
      setTestResult(`Connected. ${list.length} model(s) available.`)
    } catch (e) { setTestResult(`Connection failed: ${e}`) }
  }

  async function handleSave() {
    setSaving(true)
    try {
      await invoke("save_ai_config_cmd", { provider, baseUrl, apiKey: apiKey || null, model, embedModel, reasoningStrategy: "auto" })
      onClose()
    } finally { setSaving(false) }
  }

  function handleProviderChange(p: string) {
    setProvider(p)
    const meta = PROVIDERS.find(x => x.value === p)
    if (meta) setBaseUrl(meta.defaultUrl)
    setModels([]); setTestResult(null)
  }

  const providerMeta = PROVIDERS.find(p => p.value === provider)

  return (
    <div role="dialog" aria-modal="true" style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }}>
      <div style={{ background: "var(--bg-surface, #1e1e2e)", borderRadius: "12px", padding: "2rem", minWidth: "420px", color: "var(--text-primary, #cdd6f4)" }}>
        <h2 style={{ margin: "0 0 1.5rem" }}>AI Provider Settings</h2>
        {loading ? <p>Loading…</p> : (<>
          <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.9rem" }}>Provider</label>
          <select data-testid="ai-provider-select" value={provider} onChange={e => handleProviderChange(e.target.value)}
            style={{ width: "100%", padding: "0.5rem", background: "var(--bg-overlay, #313244)", border: "1px solid var(--border, #45475a)", borderRadius: "6px", color: "inherit", marginBottom: "1rem" }}>
            {PROVIDERS.map(p => <option key={p.value} value={p.value}>{p.label}</option>)}
          </select>
          <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.9rem" }}>Base URL</label>
          <input data-testid="ai-base-url-input" value={baseUrl} onChange={e => setBaseUrl(e.target.value)}
            style={{ width: "100%", padding: "0.5rem", background: "var(--bg-overlay, #313244)", border: "1px solid var(--border, #45475a)", borderRadius: "6px", color: "inherit", marginBottom: "1rem", boxSizing: "border-box" }} />
          {providerMeta?.needsKey && (<>
            <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.9rem" }}>API Key</label>
            <input data-testid="ai-api-key-input" type="password" value={apiKey} onChange={e => setApiKey(e.target.value)}
              style={{ width: "100%", padding: "0.5rem", background: "var(--bg-overlay, #313244)", border: "1px solid var(--border, #45475a)", borderRadius: "6px", color: "inherit", marginBottom: "1rem", boxSizing: "border-box" }} />
          </>)}
          <label style={{ display: "block", marginBottom: "0.25rem", fontSize: "0.9rem" }}>Model</label>
          <input data-testid="ai-model-input" value={model} onChange={e => setModel(e.target.value)} list="ai-models-list"
            style={{ width: "100%", padding: "0.5rem", background: "var(--bg-overlay, #313244)", border: "1px solid var(--border, #45475a)", borderRadius: "6px", color: "inherit", marginBottom: "1rem", boxSizing: "border-box" }} />
          <datalist id="ai-models-list">{models.map(m => <option key={m} value={m} />)}</datalist>
          {testResult && <p style={{ fontSize: "0.85rem", padding: "0.5rem", background: testResult.includes("Connected") ? "var(--green-dim, #1e3a2a)" : "var(--red-dim, #3a1e1e)", borderRadius: "6px", marginBottom: "1rem" }}>{testResult}</p>}
          <div style={{ display: "flex", gap: "0.75rem", justifyContent: "flex-end" }}>
            <button data-testid="ai-test-connection-btn" onClick={handleTestConnection}
              style={{ padding: "0.5rem 1rem", background: "var(--bg-overlay, #313244)", border: "1px solid var(--border, #45475a)", borderRadius: "6px", cursor: "pointer", color: "inherit" }}>Test Connection</button>
            <button onClick={onClose} style={{ padding: "0.5rem 1rem", background: "var(--bg-overlay, #313244)", border: "none", borderRadius: "6px", cursor: "pointer", color: "inherit" }}>Cancel</button>
            <button data-testid="ai-settings-save-btn" onClick={handleSave} disabled={saving}
              style={{ padding: "0.5rem 1rem", background: "var(--blue, #89b4fa)", border: "none", borderRadius: "6px", cursor: "pointer", color: "#1e1e2e", fontWeight: 600, opacity: saving ? 0.6 : 1 }}>
              {saving ? "Saving…" : "Save"}
            </button>
          </div>
        </>)}
      </div>
    </div>
  )
}
