use sqlx::sqlite::SqlitePool;

pub async fn update_book_page_count(
    pool: &SqlitePool,
    book_id: &str,
    page_count: u32,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE local_books SET page_count=? WHERE id=?")
        .bind(page_count as i64)
        .bind(book_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_book_page_count(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Option<u32>, sqlx::Error> {
    let row: Option<(Option<i64>,)> = sqlx::query_as(
        "SELECT page_count FROM local_books WHERE id=?"
    )
    .bind(book_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|(v,)| v).map(|v| v as u32))
}
