use criterion::{criterion_group, criterion_main, Criterion};
use sqlx::sqlite::SqlitePoolOptions;
use tokio::runtime::Runtime;
use xcalibre_processing::pipeline::ingest::run_ingest;
use std::path::PathBuf;

fn bench_ingest(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("run_ingest_epub", |b| {
        b.iter(|| {
            rt.block_on(async {
                let pool = SqlitePoolOptions::new()
                    .connect("sqlite::memory:")
                    .await
                    .unwrap();
                sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
                let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
                run_ingest(&pool, &path).await.unwrap()
            })
        })
    });
}

criterion_group!(benches, bench_ingest);
criterion_main!(benches);
