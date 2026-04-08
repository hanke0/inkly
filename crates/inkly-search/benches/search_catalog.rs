use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use inkly_search::{DocumentRow, IndexManager};
use tempfile::{TempDir, tempdir};

const TENANT: &str = "bench_tenant";
const DOC_COUNT: usize = 20_000;

fn build_doc(i: usize) -> DocumentRow {
    let path = format!("/root/section-{}/", i % 200);
    let tags = vec![
        format!("tag{}", i % 30),
        if i.is_multiple_of(5) {
            "hot".to_string()
        } else {
            "cold".to_string()
        },
    ];
    DocumentRow {
        doc_id: (i + 1) as u64,
        title: format!("Document {i}"),
        content: format!(
            "This is a benchmark content body for document {i}. rust axum tantivy performance search catalog"
        ),
        doc_url: format!("https://example.test/doc/{i}"),
        summary: if i.is_multiple_of(3) {
            format!("Summary for document {i} with keyword benchmark")
        } else {
            String::new()
        },
        tags,
        path,
        note: if i.is_multiple_of(7) {
            "Contains extra note text for ranking behavior".to_string()
        } else {
            String::new()
        },
    }
}

struct BenchIndex {
    _dir: TempDir,
    index: IndexManager,
}

fn setup_index() -> BenchIndex {
    let dir = tempdir().expect("create bench temp dir");
    let index = IndexManager::open_or_create(dir.path()).expect("open index manager");
    let docs = (0..DOC_COUNT).map(build_doc);
    index.index_documents(TENANT, docs).expect("bulk index");
    BenchIndex { _dir: dir, index }
}

fn bench_search(c: &mut Criterion) {
    let bench_index = setup_index();
    let mut group = c.benchmark_group("search");
    group.sample_size(30);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(DOC_COUNT as u64));

    group.bench_function("query_only", |b| {
        b.iter(|| {
            let (hits, rows) = bench_index
                .index
                .search(
                    black_box(TENANT),
                    black_box("benchmark rust"),
                    black_box(20),
                    black_box(None),
                    black_box(&[]),
                )
                .expect("search query_only");
            black_box((hits, rows.len()));
        });
    });

    let required_tags = vec!["hot".to_string(), "tag3".to_string()];
    group.bench_function("query_with_path_and_tags", |b| {
        b.iter(|| {
            let (hits, rows) = bench_index
                .index
                .search(
                    black_box(TENANT),
                    black_box("benchmark"),
                    black_box(20),
                    black_box(Some("/root/section-3/")),
                    black_box(required_tags.as_slice()),
                )
                .expect("search query_with_path_and_tags");
            black_box((hits, rows.len()));
        });
    });

    group.finish();
}

fn bench_catalog(c: &mut Criterion) {
    let bench_index = setup_index();
    let mut group = c.benchmark_group("catalog");
    group.sample_size(30);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(DOC_COUNT as u64));

    for path in ["/", "/root/", "/root/section-3/"] {
        group.bench_with_input(BenchmarkId::from_parameter(path), &path, |b, &p| {
            b.iter(|| {
                let listing = bench_index
                    .index
                    .catalog_list(black_box(TENANT), black_box(p))
                    .expect("catalog_list");
                black_box((listing.subdirs.len(), listing.files.len()));
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_search, bench_catalog);
criterion_main!(benches);
