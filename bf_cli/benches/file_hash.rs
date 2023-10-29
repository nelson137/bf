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

#[cfg(feature = "bench-alternative-hash-crates")]
fn hash_sha1_smol(lines: &[&str]) -> sha1_smol::Digest {
    let mut hash = sha1_smol::Sha1::new();
    for l in lines {
        hash.update(l.as_bytes());
    }
    hash.digest()
}

#[cfg(feature = "bench-alternative-hash-crates")]
fn hash_blake3(lines: &[&str]) -> blake3::Hash {
    let mut hasher = blake3::Hasher::new();
    for l in lines {
        hasher.update(l.as_bytes());
    }
    hasher.finalize()
}

#[cfg(feature = "bench-alternative-hash-crates")]
fn hash_metro(lines: &[&str]) -> metrohash::MetroHash128 {
    use std::hash::Hasher;
    let mut hasher = metrohash::MetroHash128::new();
    for l in lines {
        hasher.write(l.as_bytes());
    }
    hasher
}

fn criterion_benchmark(c: &mut Criterion) {
    const INPUT_ID: &str = "HelloWorld";
    let data = include_str!("../../examples/hello-world.bf");
    let data: Vec<&str> = data.lines().collect();
    let input = data.as_slice();

    let mut group = c.benchmark_group("FileHash");

    group.bench_with_input(
        BenchmarkId::new("Sha1", INPUT_ID),
        input,
        |b, i| b.iter(|| black_box(hash_sha1(black_box(i)))),
    );

    #[cfg(feature = "bench-alternative-hash-crates")]
    group.bench_with_input(
        BenchmarkId::new("Sha1_Smol", INPUT_ID),
        input,
        |b, i| b.iter(|| black_box(hash_sha1_smol(black_box(i)))),
    );

    #[cfg(feature = "bench-alternative-hash-crates")]
    group.bench_with_input(
        BenchmarkId::new("Blake3", INPUT_ID),
        input,
        |b, i| b.iter(|| black_box(hash_blake3(black_box(i)))),
    );

    #[cfg(feature = "bench-alternative-hash-crates")]
    group.bench_with_input(
        BenchmarkId::new("MetroHash", INPUT_ID),
        input,
        |b, i| b.iter(|| black_box(hash_metro(black_box(i)))),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
