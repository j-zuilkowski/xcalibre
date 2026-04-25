/**
 * Builds the init script injected into every Playwright page before the app
 * loads. Replaces window.__TAURI_INTERNALS__ so invoke() resolves with mock
 * data instead of throwing in a plain browser context.
 */

export const MOCK_BOOKS = [
  {
    id: "1",
    title: "The Great Gatsby",
    authors: ["F. Scott Fitzgerald"],
    format: "epub",
    cover_path: null,
    local_path: "/books/gatsby.epub",
    progress_percent: 23,
    last_opened_at: "2024-03-01T10:00:00Z",
    reading_cfi: null,
  },
  {
    id: "2",
    title: "1984",
    authors: ["George Orwell"],
    format: "pdf",
    cover_path: null,
    local_path: "/books/1984.pdf",
    progress_percent: 0,
    last_opened_at: null,
    reading_cfi: null,
  },
  {
    id: "3",
    title: "Dune",
    authors: ["Frank Herbert"],
    format: "mobi",
    cover_path: null,
    local_path: "/books/dune.mobi",
    progress_percent: 67,
    last_opened_at: "2024-03-05T12:00:00Z",
    reading_cfi: null,
  },
  {
    id: "4",
    title: "Foundation",
    authors: ["Isaac Asimov"],
    format: "epub",
    cover_path: null,
    local_path: "/books/foundation.epub",
    progress_percent: 0,
    last_opened_at: null,
    reading_cfi: null,
  },
  {
    id: "5",
    title: "Neuromancer",
    authors: ["William Gibson"],
    format: "epub",
    cover_path: null,
    local_path: "/books/neuromancer.epub",
    progress_percent: 100,
    last_opened_at: "2024-03-08T18:00:00Z",
    reading_cfi: null,
  },
]

export const MOCK_COLLECTIONS = [
  { id: "col1", name: "Sci-Fi", created_at: "2024-01-01" },
  { id: "col2", name: "Classics", created_at: "2024-01-02" },
]

export function buildMockScript(
  books = MOCK_BOOKS,
  options: { hasToken?: boolean; xsUrl?: string | null } = {},
): string {
  const { hasToken = false, xsUrl = null } = options
  return `(function () {
  const _books = ${JSON.stringify(books)};
  const _collections = ${JSON.stringify(MOCK_COLLECTIONS)};

  function handle(cmd, args) {
    args = args || {};
    switch (cmd) {
      case "list_books":                return _books;
      case "list_collections":          return _collections;
      case "get_books_in_collection":   return _books.map(b => b.id);
      case "filter_books":              return _books;
      case "list_authors":              return [...new Set(_books.flatMap(b => b.authors))];
      case "list_series":               return [];
      case "list_tags":                 return ["fiction", "sci-fi", "classic"];
      case "search_books": {
        const q = (args.query || "").toLowerCase();
        return _books.filter(b => b.title.toLowerCase().includes(q));
      }
      case "get_xs_url":                return ${JSON.stringify(xsUrl)};
      case "has_token":                 return ${hasToken};
      case "save_config":               return null;
      case "get_book_details": {
        const b = _books.find(b => b.id === args.bookId) || _books[0];
        return { ...b, description: "A great book.", publisher: "Publisher", pubdate: "1950-01-01", series_name: "", series_index: 0, rating: 4, tags: ["fiction"], identifiers: {} };
      }
      case "update_book_details":       return null;
      case "bulk_reingest_books":       return null;
      case "bulk_delete_books":         return null;
      case "bulk_export_metadata":      return "id,title\\n1,The Great Gatsby";
      case "ingest_file":               return null;
      case "import_calibre":            return null;
      case "repair_books":              return [];
      case "convert_book_to_epub":      return null;
      case "open_in_os":                return null;
      case "create_annotation":         return { id: "ann1" };
      case "get_annotations":           return [];
      case "delete_annotation":         return null;
      case "update_annotation_note":    return null;
      default:
        if (cmd.startsWith("plugin:event|"))   return null;
        if (cmd.startsWith("plugin:updater|")) return null;
        if (cmd.startsWith("plugin:"))         return null;
        console.warn("[tauri-mock] unhandled invoke:", cmd, args);
        return null;
    }
  }

  let _nextId = 0;
  const _cbs = {};

  window.__TAURI_INTERNALS__ = {
    invoke:   (cmd, args) => Promise.resolve(handle(cmd, args)),
    transformCallback: (cb, once) => {
      const id = ++_nextId;
      _cbs[id] = once ? (...a) => { delete _cbs[id]; cb(...a); } : cb;
      return id;
    },
    unregisterCallback: (id) => { delete _cbs[id]; },
    metadata: { currentWindow: { label: "main" } },
    convertFileSrc: (src, protocol) => (protocol || "asset") + "://" + src,
  };
})();`
}
