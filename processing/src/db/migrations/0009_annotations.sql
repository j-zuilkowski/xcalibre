-- Reader annotations: highlights, notes, bookmarks
CREATE TABLE IF NOT EXISTS annotations (
    id            TEXT    NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    book_id       TEXT    NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    type          TEXT    NOT NULL CHECK(type IN ('highlight','note','bookmark')),
    cfi           TEXT    NOT NULL,
    selected_text TEXT,
    note          TEXT,
    color         TEXT    NOT NULL DEFAULT 'yellow',
    synced        INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS annotations_book_id ON annotations(book_id);
