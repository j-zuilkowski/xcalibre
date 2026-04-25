ALTER TABLE local_books ADD COLUMN title_sort   TEXT;
ALTER TABLE local_books ADD COLUMN author_sort  TEXT;
ALTER TABLE local_books ADD COLUMN pubdate      TEXT;
ALTER TABLE local_books ADD COLUMN description  TEXT;
ALTER TABLE local_books ADD COLUMN publisher    TEXT;
ALTER TABLE local_books ADD COLUMN series_name  TEXT;
ALTER TABLE local_books ADD COLUMN series_index REAL DEFAULT 1.0;
ALTER TABLE local_books ADD COLUMN rating       INTEGER DEFAULT 0;

CREATE TABLE IF NOT EXISTS tags (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE
);

CREATE TABLE IF NOT EXISTS book_tags (
    book_id TEXT    NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    tag_id  INTEGER NOT NULL REFERENCES tags(id)        ON DELETE CASCADE,
    PRIMARY KEY (book_id, tag_id)
);

CREATE TABLE IF NOT EXISTS identifiers (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id TEXT    NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    type    TEXT    NOT NULL,
    value   TEXT    NOT NULL,
    UNIQUE  (book_id, type)
);

CREATE TABLE IF NOT EXISTS book_formats (
    id          TEXT    NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    book_id     TEXT    NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    format      TEXT    NOT NULL,
    file_path   TEXT    NOT NULL UNIQUE,
    file_sha256 TEXT    NOT NULL UNIQUE,
    file_size   INTEGER NOT NULL DEFAULT 0,
    added_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS book_formats_book_id ON book_formats(book_id);
