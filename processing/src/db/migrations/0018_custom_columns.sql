-- Custom columns: user-defined metadata fields per library.
CREATE TABLE IF NOT EXISTS custom_columns (
    id          TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    library_id  TEXT REFERENCES libraries(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    label       TEXT NOT NULL,
    col_type    TEXT NOT NULL DEFAULT 'text'
                CHECK (col_type IN ('text','integer','float','bool','date','list')),
    is_multiple INTEGER NOT NULL DEFAULT 0,
    display_in_grid INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX IF NOT EXISTS custom_columns_name_lib ON custom_columns(library_id, name);

-- Per-book values for each custom column.
CREATE TABLE IF NOT EXISTS book_custom_values (
    book_id   TEXT NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    column_id TEXT NOT NULL REFERENCES custom_columns(id) ON DELETE CASCADE,
    value     TEXT,
    PRIMARY KEY (book_id, column_id)
);
