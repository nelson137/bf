use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use sha1::{Digest, Sha1};

pub type Sha1Digest = [u8; 20];

fn hash_sha1(lines: &[&str]) -> Sha1Digest {
    let mut hash = Sha1::new();
    for l in lines {
        hash.update(l);
    }
    hash.finalize().into()
}

fn hash_blake3(lines: &[&str]) -> blake3::Hash {
    let mut hasher = blake3::Hasher::new();
    for l in lines {
        hasher.update(l.as_bytes());
    }
    hasher.finalize()
}

fn criterion_benchmark(c: &mut Criterion) {
    const INPUT_ID: &str = "HelloWorld";
    let data = include_str!("../examples/hello-world.bf");
    let data: Vec<&str> = data.lines().collect();
    let input = data.as_slice();

    let mut group = c.benchmark_group("FileHash");

    group.bench_with_input(
        BenchmarkId::new("Sha1", INPUT_ID),
        input,
        |b, i| b.iter(|| black_box(hash_sha1(black_box(i)))),
    );

    group.bench_with_input(
        BenchmarkId::new("Blake3", INPUT_ID),
        input,
        |b, i| b.iter(|| black_box(hash_blake3(black_box(i)))),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
