use std::hint::black_box;

use criterion::measurement::WallTime;
use criterion::{
    AxisScale, BatchSize, BenchmarkGroup, BenchmarkId, Criterion, PlotConfiguration,
    criterion_group, criterion_main,
};
use num_bigint::UniformBigInt;
use rand::distributions::uniform::UniformSampler;
use rand::thread_rng;

use compact_bigint::*;

const SAFE: bool = !cfg!(feature = "unsafe_encoding");

#[inline(always)]
fn generic_benchmark<T, F>(group: &mut BenchmarkGroup<WallTime>, name: &str, bit_size: usize, f: F)
where
    T: From<BigInt>,
    F: Fn(&T, &T) -> T,
{
    let rng = &mut thread_rng();
    let limit = BigInt::from(1) << (bit_size - 2);
    let sampler = UniformBigInt::new(-limit.clone(), limit.clone());

    group.bench_with_input(
        BenchmarkId::new(name, bit_size),
        &bit_size,
        |b, &_bit_size| {
            b.iter_batched_ref(
                || (T::from(sampler.sample(rng)), T::from(sampler.sample(rng))),
                |(r1, r2)| {
                    black_box(f(&*r1, &*r2));
                },
                BatchSize::SmallInput,
            );
        },
    );
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut sizes = Vec::new();

    let bits = std::mem::size_of::<compact_bigint::digits::Digit>() * 8;
    sizes.push(bits);
    sizes.push(bits + 1);
    sizes.push(bits + 2);
    // sizes.push(256);
    // sizes.push(512);

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let safety = if SAFE { "-Safe" } else { "-Unsafe" };
    {
        let mut group = c.benchmark_group("Add");
        group.plot_config(plot_config.clone());
        for &bit_size in &sizes {
            generic_benchmark::<BigInt, _>(
                &mut group,
                &("BigInt".to_string() + safety),
                bit_size,
                |x, y| x + y,
            );
            generic_benchmark::<CBigInt, _>(
                &mut group,
                &("CBigInt".to_string() + safety),
                bit_size,
                |x, y| x + y,
            );
        }
    }
    {
        let mut group = c.benchmark_group("Mul");
        group.plot_config(plot_config.clone());
        for &bit_size in &sizes {
            generic_benchmark::<BigInt, _>(
                &mut group,
                &("BigInt".to_string() + safety),
                bit_size,
                |x, y| x * y,
            );
            generic_benchmark::<CBigInt, _>(
                &mut group,
                &("CBigInt".to_string() + safety),
                bit_size,
                |x, y| x * y,
            );
        }
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
