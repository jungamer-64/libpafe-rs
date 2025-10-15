use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use libpafe::protocol::commands::Command;
use libpafe::types::{AccessMode, BlockData, BlockElement, Idm, ServiceCode};

fn bench_encode_write_multi(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_write_multi");
    for &blocks in &[1usize, 8usize, 32usize] {
        let idm = Idm::from_bytes([0x01; 8]);
        let services = vec![ServiceCode::new(0x118b)];
        let blocks_vec: Vec<BlockElement> = (0..blocks)
            .map(|i| BlockElement::new(0, AccessMode::DirectAccessOrRead, i as u16))
            .collect();
        let data: Vec<BlockData> = (0..blocks)
            .map(|_| BlockData::from_bytes([0u8; 16]))
            .collect();
        let cmd = Command::WriteWithoutEncryptionMulti {
            idm,
            services: services.clone(),
            blocks: blocks_vec,
            data,
        };

        group.bench_with_input(BenchmarkId::from_parameter(blocks), &cmd, |b, cmd| {
            b.iter(|| {
                black_box(cmd.encode());
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_encode_write_multi);
criterion_main!(benches);
