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

    // Fetch all other books
    let cand_rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, title, authors_json, series_name FROM local_books WHERE id != ?"
    )
    .bind(book_id)
    .fetch_all(pool).await?;

    let mut scored: Vec<SimilarBook> = Vec::new();
    for (cid, ctitle, cauthors_json, cseries) in cand_rows {
        let cand_authors: Vec<String> = serde_json::from_str(&cauthors_json).unwrap_or_default();
        
        // Get candidate tags
        let ctag_rows: Vec<(String,)> = sqlx::query_as(
            "SELECT t.name FROM tags t JOIN book_tags bt ON bt.tag_id = t.id WHERE bt.book_id = ?"
        )
        .bind(&cid)
        .fetch_all(pool).await.unwrap_or_default();
        let cand_tags: Vec<String> = ctag_rows.into_iter().map(|(n,)| n).collect();

        let mut score: i64 = 0;

        if factors.same_series {
            if let (Some(ss), Some(cs)) = (&src_series, &cseries) {
                if ss == cs { score += 4; }
            }
        }
        if factors.same_author {
            let any_match = src_authors.iter().any(|a| cand_authors.contains(a));
            if any_match { score += 3; }
        }
        if factors.shared_tags {
            let shared = src_tags.iter().filter(|t| cand_tags.contains(t)).count() as i64;
            score += shared.min(3);
        }
        if factors.same_language { /* skipped - no language column */ } else if false {
            // language column not available
        }

        if score == 0 { continue; }

        scored.push(SimilarBook {
            id: cid,
            title: ctitle,
            authors: serde_json::from_str::<Vec<String>>(&cauthors_json)
                .unwrap_or_default().join(", "),
            score,
        });
    }

    scored.sort_by(|a, b| b.score.cmp(&a.score).then(a.title.cmp(&b.title)));
    scored.truncate(limit as usize);
    Ok(scored)
}
