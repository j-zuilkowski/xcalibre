-- Add library_id FK to local_books so each book belongs to a library.
-- NULL means the book was imported before multi-library support (legacy).
ALTER TABLE local_books ADD COLUMN library_id TEXT REFERENCES libraries(id) ON DELETE CASCADE;
CREATE INDEX IF NOT EXISTS local_books_library_id ON local_books(library_id);
