CREATE TRIGGER books_fts_insert AFTER INSERT ON local_books BEGIN
    INSERT INTO books_fts (book_id, title, authors, description, full_text)
    VALUES (new.id, new.title, new.authors_json, '', '');
END;

CREATE TRIGGER books_fts_update AFTER UPDATE ON local_books BEGIN
    DELETE FROM books_fts WHERE book_id = old.id;
    INSERT INTO books_fts (book_id, title, authors, description, full_text)
    VALUES (new.id, new.title, new.authors_json, '', '');
END;

CREATE TRIGGER books_fts_delete AFTER DELETE ON local_books BEGIN
    DELETE FROM books_fts WHERE book_id = old.id;
END;