interface ManifestItem {
  id?: string
  href: string
  media_type: string
}

interface Props {
  items: ManifestItem[]
  selectedHref: string | null
  onSelectItem: (item: ManifestItem) => void
}

function baseName(href: string) {
  return href.split("/").pop() ?? href
}

function groupKey(mediaType: string): "html" | "css" | "images" | "other" {
  if (mediaType.includes("html") || mediaType.includes("xhtml")) return "html"
  if (mediaType.includes("css")) return "css"
  if (mediaType.includes("image")) return "images"
  return "other"
}

const GROUP_LABELS: Record<string, string> = {
  html: "Text", css: "Stylesheets", images: "Images", other: "Other",
}

export function FileTreePanel({ items, selectedHref, onSelectItem }: Props) {
  const groups: Record<string, ManifestItem[]> = { html: [], css: [], images: [], other: [] }
  for (const item of items) {
    groups[groupKey(item.media_type)].push(item)
  }

  return (
    <div data-testid="editor-file-tree" style={{ width: "200px", flexShrink: 0, overflowY: "auto", borderRight: "1px solid var(--border, #45475a)", padding: "0.5rem 0" }}>
      {(["html", "css", "images", "other"] as const).map(key => {
        const grpItems = groups[key]
        if (grpItems.length === 0) return null
        return (
          <div key={key}>
            <div data-testid={`file-group-${key}`} style={{ padding: "0.25rem 0.75rem", fontSize: "0.75rem", fontWeight: 700, color: "var(--text-muted, #6c7086)", textTransform: "uppercase", letterSpacing: "0.05em" }}>
              {GROUP_LABELS[key]}
            </div>
            {grpItems.map(item => (
              <div key={item.href} data-testid={`file-item-${item.href}`} aria-selected={item.href === selectedHref} role="option" onClick={() => onSelectItem(item)}
                style={{ padding: "0.3rem 0.75rem 0.3rem 1.25rem", cursor: "pointer", fontSize: "0.85rem", background: item.href === selectedHref ? "var(--bg-overlay, #313244)" : "transparent", borderLeft: item.href === selectedHref ? "2px solid var(--blue, #89b4fa)" : "2px solid transparent" }}>
                {baseName(item.href)}
              </div>
            ))}
          </div>
        )
      })}
    </div>
  )
}
