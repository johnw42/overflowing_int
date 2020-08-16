use std::convert::TryFrom;
use std::ops::{Add, Shr};

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use num_traits::Bounded;
use rand::distributions::Standard;
use rand::prelude::*;

use compact_bigint::*;

const INPUT_COUNT: usize = 20;

fn generic_benchmark<T, I>(name: &str, shift: I, c: &mut Criterion)
where
    I: Shr<Output = I>,
    I: Bounded,
    I: TryFrom<i128>,
    i128: From<I>,
    I: Copy,
    Standard: Distribution<I>,
    T: From<I>,
    for<'a> &'a T: Add,
{
    c.bench_function(name, |b| {
        b.iter_batched_ref(
            || {
                let rands: Vec<_> = (0..INPUT_COUNT)
                    .map(|_| T::from(random::<I>() >> shift))
                    .collect();
                rands
            },
            |rands| {
                let rands = &*rands;
                for r1 in rands {
                    for r2 in rands {
                        black_box(r1 + r2);
                    }
                }
            },
            BatchSize::SmallInput,
        )
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    generic_benchmark::<CBigInt, i128>("CBigInt i128~i32", 128 - 32, c);
    generic_benchmark::<CBigInt, i32>("CBigInt i32", 0, c);
    generic_benchmark::<CBigInt, i64>("CBigInt i63", 1, c);
    generic_benchmark::<CBigInt, i64>("CBigInt i64", 0, c);
    generic_benchmark::<CBigInt, i128>("CBigInt i65", 63, c);
    generic_benchmark::<CBigInt, i128>("CBigInt i128", 0, c);
    generic_benchmark::<BigInt, i32>("BigInt i32", 0, c);
    generic_benchmark::<BigInt, i64>("BigInt i63", 1, c);
    generic_benchmark::<BigInt, i64>("BigInt i64", 0, c);
    generic_benchmark::<BigInt, i128>("BigInt i65", 63, c);
    generic_benchmark::<BigInt, i128>("BigInt i128", 0, c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
