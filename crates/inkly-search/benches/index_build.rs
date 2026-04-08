use std::fs;
use std::path::Path;
use std::sync::Once;
use std::time::Duration;

use criterion::{BatchSize, Criterion, Throughput, black_box, criterion_group, criterion_main};
use inkly_search::{DocumentRow, IndexManager};
use tempfile::tempdir;

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

fn dir_size_bytes(path: &Path) -> u64 {
    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(p) = stack.pop() {
        let Ok(meta) = fs::metadata(&p) else {
            continue;
        };
        if meta.is_file() {
            total = total.saturating_add(meta.len());
            continue;
        }
        if !meta.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&p) else {
            continue;
        };
        for entry in entries.flatten() {
            stack.push(entry.path());
        }
    }
    total
}

fn log_index_footprint() {
    let dir = tempdir().expect("create footprint temp dir");
    let index = IndexManager::open_or_create(dir.path()).expect("open index manager");
    let docs = (0..DOC_COUNT).map(build_doc);
    index.index_documents(TENANT, docs).expect("bulk index");
    let bytes = dir_size_bytes(dir.path());
    eprintln!(
        "[bench] index footprint: docs={} bytes={} (~{:.2} MiB)",
        DOC_COUNT,
        bytes,
        bytes as f64 / (1024.0 * 1024.0)
    );
}

fn bench_index_build(c: &mut Criterion) {
    static ONCE: Once = Once::new();
    ONCE.call_once(log_index_footprint);

    let mut group = c.benchmark_group("index_build");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(15));
    group.throughput(Throughput::Elements(DOC_COUNT as u64));

    group.bench_function("index_documents_20k", |b| {
        b.iter_batched(
            || tempdir().expect("create build temp dir"),
            |dir| {
                let index = IndexManager::open_or_create(dir.path()).expect("open index manager");
                let docs = (0..DOC_COUNT).map(build_doc);
                index
                    .index_documents(black_box(TENANT), docs)
                    .expect("bulk index");
            },
            BatchSize::PerIteration,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_index_build);
criterion_main!(benches);
