-- RAG pipeline: chunked embeddings for AI book discussion (S7-D).
-- sqlite-vec stores float32 vectors as BLOBs.
CREATE TABLE IF NOT EXISTS book_chunks (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id     TEXT    NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    chunk_text  TEXT    NOT NULL,
    embedding   BLOB,             -- float32[N] stored by sqlite-vec
    embed_model TEXT,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS book_chunks_book_id ON book_chunks(book_id);
CREATE UNIQUE INDEX IF NOT EXISTS book_chunks_book_chunk ON book_chunks(book_id, chunk_index);
