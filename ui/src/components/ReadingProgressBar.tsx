interface Props {
  percent:   number
  showLabel?: boolean
  color?:    string
  height?:   number
}

export function ReadingProgressBar({
  percent,
  showLabel = false,
  color  = "var(--blue, #89b4fa)",
  height = 6,
}: Props) {
  const clamped = Math.min(100, Math.max(0, percent))

  return (
    <div style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}>
      <div
        data-testid="reading-progress-bar"
        role="progressbar"
        aria-valuenow={clamped}
        aria-valuemin={0}
        aria-valuemax={100}
        style={{
          flex: 1, height: `${height}px`,
          background: "var(--bg-overlay, #313244)",
          borderRadius: `${height / 2}px`, overflow: "hidden",
        }}
      >
        <div
          data-testid="reading-progress-fill"
          style={{
            width:        `${clamped}%`,
            height:       "100%",
            background:   clamped === 100 ? "var(--green, #a6e3a1)" : color,
            borderRadius: `${height / 2}px`,
            transition:   "width 0.3s ease",
          }}
        />
      </div>
      {showLabel && (
        <span style={{ fontSize: "0.8rem", color: "var(--text-muted, #6c7086)", minWidth: "3ch" }}>
          {Math.round(clamped)}%
        </span>
      )}
    </div>
  )
}
