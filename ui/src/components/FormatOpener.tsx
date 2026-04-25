import { invoke } from "@tauri-apps/api/core"

interface Props {
  bookId: string
  format: string
  filePath: string
}

const RENDERABLE_IN_APP = new Set(["EPUB", "CBZ", "CBR"])

/** For formats the in-app reader cannot render, open in the OS default app. */
export function FormatOpener({ bookId, format, filePath }: Props) {
  if (RENDERABLE_IN_APP.has(format.toUpperCase())) return null

  const open = () =>
    invoke("open_in_os", { path: filePath }).catch(console.error)

  return (
    <button
      onClick={open}
      className="flex items-center gap-2 px-4 py-2 rounded-lg bg-blue-500 text-white hover:bg-blue-600 text-sm font-medium"
    >
      Open {format.toUpperCase()} in {getAppHint(format)}
    </button>
  )
}

function getAppHint(format: string): string {
  const f = format.toUpperCase()
  if (f === "PDF") return "PDF viewer"
  if (f === "MOBI" || f === "AZW3" || f === "AZW4") return "Kindle / Books"
  return "default app"
}
