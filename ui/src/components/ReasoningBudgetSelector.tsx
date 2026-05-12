const OPTIONS = [
  { value: "none",   label: "None" },
  { value: "auto",   label: "Auto" },
  { value: "low",    label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high",   label: "High" },
]

interface Props {
  value: string
  onChange: (value: string) => void
}

export function ReasoningBudgetSelector({ value, onChange }: Props) {
  return (
    <select
      data-testid="reasoning-budget-select"
      value={value}
      onChange={e => onChange(e.target.value)}
      style={{
        padding: "0.3rem 0.5rem",
        background: "var(--bg-overlay, #313244)",
        border: "1px solid var(--border, #45475a)",
        borderRadius: "4px", color: "inherit", fontSize: "0.85rem",
      }}
      title="Reasoning depth"
    >
      {OPTIONS.map(o => (
        <option key={o.value} value={o.value}>{o.label}</option>
      ))}
    </select>
  )
}
