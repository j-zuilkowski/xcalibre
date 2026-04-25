use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sqlx::sqlite::SqlitePoolOptions;
use tokio::runtime::Runtime;
use xcalibre_processing::db::queries;

fn bench_list_jobs(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(async {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
        // Seed 1000 rows
        for i in 0..1000 {
            let mut job = queries::NewJob::new(&format!("/tmp/book_{}.epub", i), "EPUB");
            job.file_sha256 = format!("sha256_{}", i);
            queries::create_job(&pool, &job).await.unwrap();
        }
        pool
    });

    c.bench_function("list_jobs_by_status_1000", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(queries::list_jobs_by_status(&pool, "PENDING").await.unwrap())
            })
        })
    });
}

criterion_group!(benches, bench_list_jobs);
criterion_main!(benches);
