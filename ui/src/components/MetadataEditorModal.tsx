import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Identifier {
  id_type: string
  value: string
}

interface BookDetails {
  id: string
  title: string
  authors: string[]
  pubdate: string | null
  description: string | null
  publisher: string | null
  series_name: string | null
  series_index: number | null
  rating: number
  tags: string[]
  identifiers: Identifier[]
}

interface Props {
  open: boolean
  bookIds: string[]
  onClose: () => void
  onSaved: () => Promise<void> | void
}

export function MetadataEditorModal({ open, bookIds, onClose, onSaved }: Props) {
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [title, setTitle] = useState("")
  const [authors, setAuthors] = useState("")
  const [pubdate, setPubdate] = useState("")
  const [description, setDescription] = useState("")
  const [publisher, setPublisher] = useState("")
  const [seriesName, setSeriesName] = useState("")
  const [seriesIndex, setSeriesIndex] = useState("")
  const [rating, setRating] = useState("0")
  const [tags, setTags] = useState("")
  const [identifiers, setIdentifiers] = useState("")

  useEffect(() => {
    if (!open || bookIds.length === 0) {
      return
    }

    let cancelled = false
    setLoading(true)
    setError(null)

    invoke<BookDetails>("get_book_details", { bookId: bookIds[0] })
      .then((details) => {
        if (cancelled) return
        setTitle(details.title ?? "")
        setAuthors((details.authors ?? []).join(", "))
        setPubdate(details.pubdate ?? "")
        setDescription(details.description ?? "")
        setPublisher(details.publisher ?? "")
        setSeriesName(details.series_name ?? "")
        setSeriesIndex(details.series_index?.toString() ?? "")
        setRating((details.rating ?? 0).toString())
        setTags((details.tags ?? []).join(", "))
        setIdentifiers(
          (details.identifiers ?? [])
            .map((identifier) => `${identifier.id_type}=${identifier.value}`)
            .join("\n"),
        )
      })
      .catch((err) => {
        if (!cancelled) {
          setError(String(err))
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false)
      })

    return () => {
      cancelled = true
    }
  }, [bookIds, open])

  if (!open) return null

  const save = async () => {
    const parsedAuthors = authors
      .split(/[\n,]/)
      .map((value) => value.trim())
      .filter(Boolean)
    const parsedTags = tags
      .split(/[\n,]/)
      .map((value) => value.trim())
      .filter(Boolean)
    const parsedIdentifiers = identifiers
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean)
      .map((line) => {
        const separator = line.indexOf("=")
        if (separator <= 0) return null
        return {
          idType: line.slice(0, separator).trim(),
          value: line.slice(separator + 1).trim(),
        }
      })
      .filter((entry): entry is { idType: string; value: string } => Boolean(entry))

    setSaving(true)
    setError(null)
    try {
      await invoke("update_book_details", {
        bookIds,
        details: {
          title: title.trim(),
          authors: parsedAuthors,
          pubdate: pubdate.trim(),
          description: description.trim(),
          publisher: publisher.trim(),
          seriesName: seriesName.trim(),
          seriesIndex: seriesIndex.trim() ? Number(seriesIndex) : null,
          rating: rating.trim() ? Number.parseInt(rating, 10) : null,
          tags: parsedTags,
          identifiers: parsedIdentifiers,
        },
      })
      await onSaved()
      onClose()
    } catch (err) {
      setError(String(err))
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="w-full max-w-3xl rounded-2xl border border-gray-200 bg-white p-6 shadow-2xl dark:border-gray-700 dark:bg-gray-900">
        <div className="flex items-start justify-between gap-4">
          <div>
            <p className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400">
              Metadata editor
            </p>
            <h2 className="mt-1 text-xl font-semibold text-gray-900 dark:text-white">
              {bookIds.length === 1 ? "Edit one book" : `Edit ${bookIds.length} books`}
            </h2>
          </div>
          <button
            onClick={onClose}
            className="rounded-lg border border-gray-300 px-3 py-1.5 text-sm text-gray-600 hover:bg-gray-100 dark:border-gray-700 dark:text-gray-300 dark:hover:bg-gray-800"
          >
            Close
          </button>
        </div>

        {loading ? (
          <div className="py-10 text-sm text-gray-500 dark:text-gray-400">Loading metadata…</div>
        ) : (
          <div className="mt-6 grid gap-4 md:grid-cols-2">
            <label className="md:col-span-2 flex flex-col gap-2">
              <span className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Title
              </span>
              <input
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                className="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 outline-none ring-0 focus:border-sky-500 dark:border-gray-700 dark:bg-gray-950 dark:text-white"
              />
            </label>

            <label className="md:col-span-2 flex flex-col gap-2">
              <span className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Authors
              </span>
              <textarea
                value={authors}
                onChange={(e) => setAuthors(e.target.value)}
                rows={2}
                className="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 outline-none focus:border-sky-500 dark:border-gray-700 dark:bg-gray-950 dark:text-white"
                placeholder="Comma or newline separated"
              />
            </label>

            <label className="flex flex-col gap-2">
              <span className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Publisher
              </span>
              <input
                value={publisher}
                onChange={(e) => setPublisher(e.target.value)}
                className="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 outline-none focus:border-sky-500 dark:border-gray-700 dark:bg-gray-950 dark:text-white"
              />
            </label>

            <label className="flex flex-col gap-2">
              <span className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Pub date
              </span>
              <input
                value={pubdate}
                onChange={(e) => setPubdate(e.target.value)}
                className="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 outline-none focus:border-sky-500 dark:border-gray-700 dark:bg-gray-950 dark:text-white"
                placeholder="YYYY-MM-DD"
              />
            </label>

            <label className="flex flex-col gap-2">
              <span className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Series
              </span>
              <input
                value={seriesName}
                onChange={(e) => setSeriesName(e.target.value)}
                className="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 outline-none focus:border-sky-500 dark:border-gray-700 dark:bg-gray-950 dark:text-white"
              />
            </label>

            <label className="flex flex-col gap-2">
              <span className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Series index
              </span>
              <input
                value={seriesIndex}
                onChange={(e) => setSeriesIndex(e.target.value)}
                type="number"
                step="0.1"
                className="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 outline-none focus:border-sky-500 dark:border-gray-700 dark:bg-gray-950 dark:text-white"
              />
            </label>

            <label className="flex flex-col gap-2">
              <span className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Rating
              </span>
              <input
                value={rating}
                onChange={(e) => setRating(e.target.value)}
                type="number"
                min={0}
                max={5}
                step="1"
                className="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 outline-none focus:border-sky-500 dark:border-gray-700 dark:bg-gray-950 dark:text-white"
              />
            </label>

            <label className="md:col-span-2 flex flex-col gap-2">
              <span className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Tags
              </span>
              <input
                value={tags}
                onChange={(e) => setTags(e.target.value)}
                className="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 outline-none focus:border-sky-500 dark:border-gray-700 dark:bg-gray-950 dark:text-white"
                placeholder="Comma or newline separated"
              />
            </label>

            <label className="md:col-span-2 flex flex-col gap-2">
              <span className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Description
              </span>
              <textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                rows={4}
                className="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 outline-none focus:border-sky-500 dark:border-gray-700 dark:bg-gray-950 dark:text-white"
              />
            </label>

            <label className="md:col-span-2 flex flex-col gap-2">
              <span className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Identifiers
              </span>
              <textarea
                value={identifiers}
                onChange={(e) => setIdentifiers(e.target.value)}
                rows={3}
                className="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 outline-none focus:border-sky-500 dark:border-gray-700 dark:bg-gray-950 dark:text-white"
                placeholder="isbn=9780000000000"
              />
            </label>
          </div>
        )}

        {error && (
          <p className="mt-4 rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300">
            {error}
          </p>
        )}

        <div className="mt-6 flex items-center justify-end gap-3">
          <button
            onClick={onClose}
            className="rounded-lg border border-gray-300 px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:border-gray-700 dark:text-gray-300 dark:hover:bg-gray-800"
          >
            Cancel
          </button>
          <button
            onClick={() => void save()}
            disabled={saving}
            className="rounded-lg bg-sky-600 px-4 py-2 text-sm font-medium text-white hover:bg-sky-500 disabled:opacity-60"
          >
            {saving ? "Saving…" : "Save changes"}
          </button>
        </div>
      </div>
    </div>
  )
}
