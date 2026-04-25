CREATE VIRTUAL TABLE books_fts USING fts5(
    book_id   UNINDEXED,
    title,
    authors,
    description,
    full_text
);
