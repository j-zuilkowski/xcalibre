CREATE TABLE collections (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);
CREATE TABLE collection_books (
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    book_id       TEXT NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    added_at      TEXT NOT NULL,
    PRIMARY KEY (collection_id, book_id)
);
