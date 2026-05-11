-- Each row is a named local library with its own DB path and cover directory.
-- layout: 'in_place' | 'managed' (S6-C decision: configurable per library)
CREATE TABLE IF NOT EXISTS libraries (
    id          TEXT    NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name        TEXT    NOT NULL,
    db_path     TEXT    NOT NULL UNIQUE,
    cover_dir   TEXT    NOT NULL,
    layout      TEXT    NOT NULL DEFAULT 'in_place'
                        CHECK (layout IN ('in_place', 'managed')),
    xs_url      TEXT,
    is_active   INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS libraries_is_active ON libraries(is_active);
