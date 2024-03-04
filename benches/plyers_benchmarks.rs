use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rootspace::plyers::load_ply;

pub fn load_ply_benchmark(c: &mut Criterion) {
    let path = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/valid/jasmin6.ply"));
    c.bench_function("load_ply", |b| b.iter(|| {
        load_ply(black_box(path))
    }));
}

criterion_group!(benches, load_ply_benchmark);
criterion_main!(benches);
