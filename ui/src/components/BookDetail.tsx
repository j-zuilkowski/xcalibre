import { Book } from "../store/libraryStore"

interface Props {
  book: Book
  onClose: () => void
  onDiscuss?: () => void
}

export function BookDetail({ book, onClose, onDiscuss }: Props) {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-md p-6">
        <button
          onClick={onClose}
          className="float-right text-gray-400 hover:text-gray-600 text-xl leading-none"
        >
          ×
        </button>
        <h2 className="text-lg font-semibold dark:text-white mb-1">{book.title}</h2>
        <p className="text-sm text-gray-500 mb-4">{book.authors.join(", ")}</p>
        <p className="text-xs text-gray-400 uppercase tracking-wide mb-1">Format</p>
        <p className="text-sm dark:text-gray-300 mb-4">{book.format}</p>
        {book.progress_percent > 0 && (
          <>
            <p className="text-xs text-gray-400 uppercase tracking-wide mb-1">Progress</p>
            <p className="text-sm dark:text-gray-300">{Math.round(book.progress_percent)}%</p>
          </>
        )}
        {onDiscuss && (
          <button
            data-testid="discuss-book-btn"
            onClick={onDiscuss}
            className="mt-4 w-full py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg text-sm font-medium"
          >
            Discuss with AI
          </button>
        )}
      </div>
    </div>
  )
}
