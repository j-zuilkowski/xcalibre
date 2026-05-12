-- Recreate jobs table without the format CHECK constraint.
-- The original constraint only allowed 7 formats; the app now supports 20+.
PRAGMA foreign_keys = OFF;

CREATE TABLE jobs_new (
    id                TEXT PRIMARY KEY,
    file_path         TEXT NOT NULL,
    file_sha256       TEXT NOT NULL,
    format            TEXT NOT NULL,
    status            TEXT NOT NULL DEFAULT 'PENDING'
                          CHECK (status IN ('PENDING','READY_TO_PUSH','PUSHING','RETRYING','COMPLETED','FAILED')),
    retry_count       INTEGER NOT NULL DEFAULT 0,
    next_retry_at     TEXT,
    xs_book_id        TEXT,
    push_step         INTEGER NOT NULL DEFAULT 0,
    error_message     TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);

INSERT INTO jobs_new SELECT * FROM jobs;
DROP TABLE jobs;
ALTER TABLE jobs_new RENAME TO jobs;

CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_sha256 ON jobs(file_sha256);

PRAGMA foreign_keys = ON;
