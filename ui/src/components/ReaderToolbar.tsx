import type { RefObject } from "react"
import { useEffect } from "react"
import { useSettingsStore } from "../store/settingsStore"

interface Props {
  iframeRef: RefObject<HTMLIFrameElement>
}

type Settings = ReturnType<typeof useSettingsStore.getState>

export function ReaderToolbar({ iframeRef }: Props) {
  const { fontSize, theme, fontFamily, setFontSize, setTheme, setFontFamily } =
    useSettingsStore()

  const postCss = (css: Record<string, string>) => {
    iframeRef.current?.contentWindow?.postMessage({ type: "xcalibre-css", css }, "*")
  }

  useEffect(() => {
    const font = fontFamily === "serif"
      ? "Georgia, serif"
      : fontFamily === "sans"
        ? "system-ui, sans-serif"
        : "monospace"

    postCss({
      "--size": `${fontSize}px`,
      "--font": font,
      "--theme": theme,
    })
  }, [fontSize, theme, fontFamily, iframeRef])

  return (
    <div className="flex items-center gap-4 px-4 py-2 border-b border-gray-200 dark:border-gray-700 text-sm">
      <label className="flex items-center gap-2">
        <span className="text-gray-600 dark:text-gray-400">Size</span>
        <input
          type="range"
          min={14}
          max={24}
          value={fontSize}
          onChange={(e) => {
            const n = Number(e.target.value)
            setFontSize(n)
            postCss({ "--size": `${n}px` })
          }}
          className="w-24"
        />
        <span className="w-6 text-center">{fontSize}</span>
      </label>

      <label className="flex items-center gap-2">
        <span className="text-gray-600 dark:text-gray-400">Theme</span>
        <select
          value={theme}
          onChange={(e) => {
            const t = e.target.value as Settings["theme"]
            setTheme(t)
            postCss({ "--theme": t })
          }}
          className="border border-gray-300 rounded px-1 py-0.5"
        >
          <option value="light">Light</option>
          <option value="dark">Dark</option>
          <option value="sepia">Sepia</option>
        </select>
      </label>

      <label className="flex items-center gap-2">
        <span className="text-gray-600 dark:text-gray-400">Font</span>
        <select
          value={fontFamily}
          onChange={(e) => {
            const f = e.target.value as Settings["fontFamily"]
            setFontFamily(f)
            postCss({
              "--font":
                f === "serif"
                  ? "Georgia, serif"
                  : f === "sans"
                    ? "system-ui, sans-serif"
                    : "monospace",
            })
          }}
          className="border border-gray-300 rounded px-1 py-0.5"
        >
          <option value="serif">Serif</option>
          <option value="sans">Sans</option>
          <option value="mono">Mono</option>
        </select>
      </label>
    </div>
  )
}
