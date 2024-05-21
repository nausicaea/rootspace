use criterion::{black_box, criterion_group, criterion_main, Criterion};
use plyers::{load_ply, save_ply};

fn load_ply_jasmin6(c: &mut Criterion) {
    let path = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/valid/jasmin6.ply"));
    c.bench_function("load_ply_jasmin6", |b| b.iter(|| load_ply(black_box(path))));
}

fn load_ply_expected_feature_2_segment_0(c: &mut Criterion) {
    let path = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/valid/expected_feature_2_segment_0.ply"
    ));
    c.bench_function("load_ply_expected_feature_2_segment_0", |b| {
        b.iter(|| load_ply(black_box(path)))
    });
}

fn save_ply_jasmin6(c: &mut Criterion) {
    let path = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/valid/jasmin6.ply"));
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let jasmin6 = load_ply(path).unwrap();
    c.bench_function("save_ply_jasmin6", |b| {
        b.iter(|| save_ply(black_box(&jasmin6), black_box(tmp.path())))
    });
}

fn save_ply_expected_feature_2_segment_0(c: &mut Criterion) {
    let path = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/valid/expected_feature_2_segment_0.ply"
    ));
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let jasmin6 = load_ply(path).unwrap();
    c.bench_function("save_ply_expected_feature_2_segment_0", |b| {
        b.iter(|| save_ply(black_box(&jasmin6), black_box(tmp.path())))
    });
}

criterion_group!(
    benches,
    load_ply_jasmin6,
    load_ply_expected_feature_2_segment_0,
    save_ply_jasmin6,
    save_ply_expected_feature_2_segment_0
);
criterion_main!(benches);
