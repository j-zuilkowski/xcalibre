CREATE TABLE enrichment_cache (
    isbn       TEXT PRIMARY KEY,
    ol_json    TEXT,
    gb_json    TEXT,
    fetched_at TEXT NOT NULL
);