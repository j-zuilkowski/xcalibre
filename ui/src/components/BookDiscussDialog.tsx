import { AIChatPanel } from "./AIChatPanel"

interface Book {
  id: string; title: string; authors: string[]
  format: string; cover_path: string | null
  progress_percent: number; last_opened_at: string | null
}
interface Props { book: Book; onClose: () => void }

export function BookDiscussDialog({ book, onClose }: Props) {
  return (
    <div data-testid="book-discuss-dialog"
         className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-900 rounded-xl shadow-2xl w-full max-w-lg h-[600px] flex flex-col overflow-hidden">
        <AIChatPanel book={book} onClose={onClose} />
      </div>
    </div>
  )
}
