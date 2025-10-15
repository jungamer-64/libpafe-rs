use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use libpafe::protocol::checksum::{dcs, lcs};

fn bench_lcs(c: &mut Criterion) {
    let mut group = c.benchmark_group("lcs");
    for &v in &[0u8, 1u8, 128u8, 255u8] {
        group.bench_with_input(BenchmarkId::from_parameter(v), &v, |b, &v| {
            b.iter(|| {
                black_box(lcs(black_box(v)));
            });
        });
    }
    group.finish();
}

fn bench_dcs(c: &mut Criterion) {
    let mut group = c.benchmark_group("dcs");
    for &size in &[0usize, 16usize, 64usize, 256usize] {
        let payload: Vec<u8> = (0..size).map(|i| (i & 0xff) as u8).collect();
        group.bench_with_input(BenchmarkId::from_parameter(size), &payload, |b, p| {
            b.iter(|| {
                black_box(dcs(black_box(p)));
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_lcs, bench_dcs);
criterion_main!(benches);
