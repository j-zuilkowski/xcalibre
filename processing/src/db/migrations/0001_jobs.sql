CREATE TABLE jobs (
    id                TEXT PRIMARY KEY,
    file_path         TEXT NOT NULL,
    file_sha256       TEXT NOT NULL,
    format            TEXT NOT NULL CHECK (format IN ('EPUB','PDF','MOBI','AZW3','CBZ','CBR','TXT')),
    status            TEXT NOT NULL DEFAULT 'PENDING'
                          CHECK (status IN ('PENDING','READY_TO_PUSH','PUSHING','RETRYING','COMPLETED','FAILED')),
    retry_count       INTEGER NOT NULL DEFAULT 0,
    next_retry_at     TEXT,
    xs_book_id TEXT,
    push_step         INTEGER NOT NULL DEFAULT 0,
    error_message     TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);
CREATE INDEX idx_jobs_status  ON jobs(status);
CREATE INDEX idx_jobs_sha256  ON jobs(file_sha256);

CREATE TABLE job_metadata (
    job_id        TEXT PRIMARY KEY REFERENCES jobs(id) ON DELETE CASCADE,
    metadata_json TEXT NOT NULL,
    extracted_at  TEXT NOT NULL
);

CREATE TABLE job_text (
    job_id       TEXT PRIMARY KEY REFERENCES jobs(id) ON DELETE CASCADE,
    full_text    TEXT NOT NULL,
    word_count   INTEGER NOT NULL,
    user_edited  INTEGER NOT NULL DEFAULT 0,
    updated_at   TEXT NOT NULL
);

CREATE TABLE local_books (
    id               TEXT PRIMARY KEY,
    title            TEXT NOT NULL,
    authors_json     TEXT NOT NULL,
    format           TEXT NOT NULL,
    local_path       TEXT,
    cover_path       TEXT,
    reading_position TEXT,
    progress_percent REAL NOT NULL DEFAULT 0,
    last_opened_at   TEXT,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);

CREATE TABLE push_queue (
    job_id     TEXT PRIMARY KEY REFERENCES jobs(id) ON DELETE CASCADE,
    queued_at  TEXT NOT NULL,
    last_error TEXT
);
