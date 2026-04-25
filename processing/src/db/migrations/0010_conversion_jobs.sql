CREATE TABLE IF NOT EXISTS conversion_jobs (
    id            TEXT PRIMARY KEY,
    book_id       TEXT NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    source_format TEXT NOT NULL,
    target_format TEXT NOT NULL,
    mode          TEXT NOT NULL CHECK (mode IN ('CONVERT', 'TWEAK')),
    output_path   TEXT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'PENDING'
                     CHECK (status IN ('PENDING', 'RUNNING', 'COMPLETED', 'FAILED')),
    error_message TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS conversion_jobs_book_id ON conversion_jobs(book_id);
