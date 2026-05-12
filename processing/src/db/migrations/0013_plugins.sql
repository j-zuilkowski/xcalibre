CREATE TABLE IF NOT EXISTS installed_plugins (
    id           TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name         TEXT NOT NULL UNIQUE,
    version      TEXT NOT NULL,
    api_version  INTEGER NOT NULL,
    plugin_type  TEXT NOT NULL CHECK (plugin_type IN ('metadata_source','conversion_output','store')),
    dylib_path   TEXT NOT NULL,
    enabled      INTEGER NOT NULL DEFAULT 1,
    installed_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
