CREATE TABLE bookmarks (
    id         TEXT PRIMARY KEY,
    book_id    TEXT NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    cfi        TEXT NOT NULL,
    label      TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_bookmarks_book ON bookmarks(book_id);