-- Per-library AI configuration.
CREATE TABLE IF NOT EXISTS ai_config (
    id            TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    library_id    TEXT REFERENCES libraries(id) ON DELETE CASCADE,
    provider      TEXT NOT NULL DEFAULT 'ollama',
    model         TEXT NOT NULL DEFAULT 'llama3',
    embed_model   TEXT NOT NULL DEFAULT 'nomic-embed-text',
    base_url      TEXT NOT NULL DEFAULT 'http://localhost:11434',
    api_key       TEXT,
    reasoning_strategy TEXT NOT NULL DEFAULT 'auto'
                  CHECK (reasoning_strategy IN ('auto','none','low','medium','high')),
    include_fields TEXT NOT NULL DEFAULT '["title","authors","tags","series","description"]',
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
