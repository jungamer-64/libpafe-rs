use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use libpafe::protocol::commands::Command;
use libpafe::protocol::frame::Frame;
use libpafe::types::{AccessMode, BlockElement, Idm, ServiceCode};

fn bench_frame_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_roundtrip");
    for &size in &[8usize, 64usize, 240usize] {
        let payload: Vec<u8> = (0..size).map(|i| (i & 0xff) as u8).collect();
        group.bench_with_input(BenchmarkId::from_parameter(size), &payload, |b, payload| {
            b.iter(|| {
                let frame = Frame::encode(black_box(payload)).expect("encode");
                let out = Frame::decode(black_box(&frame)).expect("decode");
                black_box(out);
            });
        });
    }
    group.finish();
}

fn bench_command_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("command_encode");

    let polling = Command::Polling {
        system_code: libpafe::types::SystemCode::new(0x1234),
        request_code: 1,
        time_slot: 0,
    };
    group.bench_function("polling_encode", |b| {
        b.iter(|| {
            black_box(polling.encode());
        })
    });

    // Read command with multiple services/blocks
    let idm = Idm::from_bytes([0u8; 8]);
    let services: Vec<ServiceCode> = (0..4).map(|i| ServiceCode::new(0x090f + i)).collect();
    let blocks: Vec<BlockElement> = (0..8)
        .map(|i| BlockElement::new(0, AccessMode::DirectAccessOrRead, i as u16))
        .collect();
    let read_cmd = Command::ReadWithoutEncryption {
        idm,
        services,
        blocks,
    };
    group.bench_function("read_encode_8blocks", |b| {
        b.iter(|| {
            black_box(read_cmd.encode());
        })
    });

    group.finish();
}

criterion_group!(benches, bench_frame_roundtrip, bench_command_encode);
criterion_main!(benches);
