use std::ops::Add;

use criterion::measurement::WallTime;
use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkGroup, BenchmarkId, Criterion,
};
use rand::distributions::Uniform;
use rand::prelude::*;

use compact_bigint::*;

fn generic_benchmark<T>(group: &mut BenchmarkGroup<WallTime>, name: &str, bit_size: u64)
where
    T: From<BigInt>,
    for<'a> &'a T: Add<&'a T>,
{
    let mut rng = thread_rng();
    let limit = BigInt::from(1) << (bit_size - 2);
    let sampler = Uniform::new(-limit.clone(), limit.clone());

    group.bench_with_input(
        BenchmarkId::new(name, bit_size),
        &bit_size,
        |b, &_bit_size| {
            b.iter_batched_ref(
                || (T::from(rng.sample(&sampler)), T::from(rng.sample(&sampler))),
                |(r1, r2)| {
                    black_box(&*r1 + &*r2);
                },
                BatchSize::SmallInput,
            );
        },
    );
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Add");
    for &bit_size in &[63, 64, 65, 128, 256, 512, 1024] {
        generic_benchmark::<BigInt>(&mut group, "BigInt", bit_size);
        generic_benchmark::<CBigInt>(&mut group, "CBigInt", bit_size);
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
