-- Rich notes attached to a book.
CREATE TABLE IF NOT EXISTS notes (
    id          TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    book_id     TEXT NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    title       TEXT NOT NULL DEFAULT 'Untitled Note',
    body_html   TEXT NOT NULL DEFAULT '',
    body_text   TEXT NOT NULL DEFAULT '',  -- plain-text version for FTS
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS notes_book_id ON notes(book_id);
CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
    body_text, title,
    content='notes',
    content_rowid='rowid'
);
CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
    INSERT INTO notes_fts(rowid, body_text, title) VALUES (new.rowid, new.body_text, new.title);
END;
CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, body_text, title) VALUES ('delete', old.rowid, old.body_text, old.title);
    INSERT INTO notes_fts(rowid, body_text, title) VALUES (new.rowid, new.body_text, new.title);
END;
CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, body_text, title) VALUES ('delete', old.rowid, old.body_text, old.title);
END;
