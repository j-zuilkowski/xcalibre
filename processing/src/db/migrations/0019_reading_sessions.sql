ALTER TABLE local_books ADD COLUMN page_count INTEGER;

CREATE TABLE IF NOT EXISTS reading_sessions (
    id          TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    book_id     TEXT NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    started_at  TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at    TEXT,
    duration_s  INTEGER NOT NULL DEFAULT 0,
    progress_start REAL NOT NULL DEFAULT 0.0,
    progress_end   REAL NOT NULL DEFAULT 0.0
);
CREATE INDEX IF NOT EXISTS rs_book_id ON reading_sessions(book_id);
