use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityFactors {
    pub same_author:   bool,
    pub same_series:   bool,
    pub shared_tags:   bool,
    pub same_language: bool,
}

impl Default for SimilarityFactors {
    fn default() -> Self {
        Self { same_author: true, same_series: true, shared_tags: true, same_language: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SimilarBook {
    pub id:      String,
    pub title:   String,
    pub authors: String,
    pub score:   i64,
}

#[derive(Debug, sqlx::FromRow)]
struct CandRow {
    id: String,
    title: String,
    authors_json: String,
    series_name: Option<String>,
    tags_csv: Option<String>,
}

/// Score weights:
///   same_series:   4
///   same_author:   3
///   per_shared_tag:1 (up to 3 total)
///   same_language: 1
pub async fn find_similar_books(
    pool: &SqlitePool,
    book_id: &str,
    factors: &SimilarityFactors,
    limit: i64,
) -> Result<Vec<SimilarBook>, sqlx::Error> {
    // Fetch source book metadata
    let src_row: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT authors_json, series_name FROM local_books WHERE id=?"
    )
    .bind(book_id)
    .fetch_optional(pool).await?;

    let (src_authors_json, src_series) = match src_row {
        Some(r) => r,
        None => return Ok(vec![]),
    };

    let src_authors: Vec<String> = serde_json::from_str(&src_authors_json).unwrap_or_default();

    // Get source tags
    let tag_rows: Vec<(String,)> = sqlx::query_as(
        "SELECT t.name FROM tags t JOIN book_tags bt ON bt.tag_id = t.id WHERE bt.book_id = ?"
    )
    .bind(book_id)
    .fetch_all(pool).await.unwrap_or_default();
    let src_tags: Vec<String> = tag_rows.into_iter().map(|(n,)| n).collect();

    // Fetch all other books with tags in a single GROUP_CONCAT JOIN
    let cand_rows: Vec<CandRow> = sqlx::query_as(
        "SELECT b.id, b.title, b.authors_json, b.series_name,
                GROUP_CONCAT(t.name, ',') AS tags_csv
         FROM local_books b
         LEFT JOIN book_tags bt ON bt.book_id = b.id
         LEFT JOIN tags t       ON t.id = bt.tag_id
         WHERE b.id != ?
         GROUP BY b.id"
    )
    .bind(book_id)
    .fetch_all(pool).await?;

    let mut scored: Vec<SimilarBook> = Vec::new();
    for row in &cand_rows {
        let cand_authors: Vec<String> = serde_json::from_str(&row.authors_json).unwrap_or_default();
        let cand_tags: Vec<&str> = row
            .tags_csv
            .as_deref()
            .unwrap_or("")
            .split(',')
            .filter(|s| !s.is_empty())
            .collect();

        let mut score: i64 = 0;

        if factors.same_series {
            if let (Some(ss), Some(cs)) = (&src_series, &row.series_name) {
                if ss == cs { score += 4; }
            }
        }
        if factors.same_author {
            let any_match = src_authors.iter().any(|a| cand_authors.contains(a));
            if any_match { score += 3; }
        }
        if factors.shared_tags {
            let shared = src_tags.iter().filter(|t| cand_tags.contains(&t.as_str())).count() as i64;
            score += shared.min(3);
        }

        if score == 0 { continue; }

        scored.push(SimilarBook {
            id: row.id.clone(),
            title: row.title.clone(),
            authors: cand_authors.join(", "),
            score,
        });
    }

    scored.sort_by(|a, b| b.score.cmp(&a.score).then(a.title.cmp(&b.title)));
    scored.truncate(limit as usize);
    Ok(scored)
}
