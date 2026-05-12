-- Virtual libraries: saved searches that appear as dynamic collections.
CREATE TABLE IF NOT EXISTS virtual_libraries (
    id          TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    library_id  TEXT,
    name        TEXT NOT NULL,
    search_expr TEXT NOT NULL,   -- serialized QueryNode expression string
    sort_field  TEXT NOT NULL DEFAULT 'title',
    sort_asc    INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS vlib_library_id ON virtual_libraries(library_id);
