use criterion::{
    AxisScale, BatchSize, BenchmarkId, Criterion, PlotConfiguration, criterion_group,
    criterion_main,
};
use num_integer::Integer;
use num_traits::{CheckedAdd, CheckedDiv, CheckedEuclid, CheckedMul, CheckedSub, Euclid, Pow};
use paste::paste;
use rand::distributions::uniform::SampleUniform;
use rand::distributions::uniform::UniformSampler;
use rand::thread_rng;
use std::hint::black_box;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};

use compact_bigint::bench::*;
use compact_bigint::*;

macro_rules! duplicate_bigint_types {
    ($($body:tt)*) => {
        duplicate::duplicate! {
            [
                label         Encoded;
                [Arc]         [ArcBigInt];
                [ArcIsize]    [ArcBigIsize];
                [Box]         [BoxBigInt];
                [Cow]         [CowBigInt::<'static>];
                [Identity]    [IdentityBigInt::<'static>];
                [Rc]          [RcBigInt];
                [RcIsize]     [RcBigIsize];
                [Control]     [BigInt];
            ]
            $($body)*
        }
    }
}

macro_rules! duplicate_binary_operators {
    ($($body:tt)*) => {
        duplicate::duplicate! {
            [
                OpTrait      op_fn;
                [Add]        [add];
                [Sub]        [sub];
                [Mul]        [mul];
                [Div]        [div];
                [Rem]        [rem];
                [BitAnd]     [bitand];
                [BitOr]      [bitor];
                [BitXor]     [bitxor];
                [CheckedAdd] [checked_add];
                [CheckedSub] [checked_sub];
                [CheckedMul] [checked_mul];
                [CheckedDiv] [checked_div];
                [Integer]    [div_floor];
                [Integer]    [mod_floor];
                [Integer]    [gcd];
                [Integer]    [lcm];
                [Integer]    [div_rem];
                [Euclid]     [div_euclid];
                [Euclid]     [rem_euclid];
                [Euclid]     [div_rem_euclid];
                [CheckedEuclid] [checked_div_euclid];
                [CheckedEuclid] [checked_rem_euclid];
                [CheckedEuclid] [checked_div_rem_euclid];
            ]
            $($body)*
        }
    }
}

macro_rules! duplicate_shift_operators {
    ($($body:tt)*) => {
        duplicate::duplicate! {
            [
                OpTrait      op_fn;
                [Shl]        [shl];
                [Shr]        [shr];
            ]
            $($body)*
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let rng = &mut thread_rng();

    for bit_size in [63, 64, 65, 127, 128, 129] {
        let limit: BigInt = BigInt::from(1) << (bit_size - 2);
        let big_sampler = <BigInt as SampleUniform>::Sampler::new(-limit.clone(), limit.clone());
        let shift_sampler = <u32 as SampleUniform>::Sampler::new(0, 64);
        duplicate_binary_operators! {
            {
                let mut group = c.benchmark_group(stringify!(op_fn));
                group.plot_config(plot_config.clone());

                duplicate_bigint_types! {
                    paste! {
                        group.bench_function(
                            BenchmarkId::new(stringify!(label), bit_size),
                            |b| {
                                b.iter_batched_ref(
                                    || (Encoded::from(big_sampler.sample(rng)), Encoded::from(big_sampler.sample(rng))),
                                    |(r1, r2)| {
                                        black_box(OpTrait::op_fn(&*r1, &*r2));
                                    },
                                    BatchSize::SmallInput,
                                );
                            },
                        );
                    }
                }
            }
        }

        duplicate_shift_operators! {
            {
                let mut group = c.benchmark_group(stringify!(op_fn));
                group.plot_config(plot_config.clone());

                duplicate_bigint_types! {
                    paste! {
                        group.bench_function(
                            BenchmarkId::new(stringify!(label), bit_size),
                            |b| {
                                b.iter_batched_ref(
                                    || (Encoded::from(big_sampler.sample(rng)), shift_sampler.sample(rng)),
                                    |(r1, r2)| {
                                        black_box(OpTrait::op_fn(&*r1, *r2));
                                    },
                                    BatchSize::SmallInput,
                                );
                            },
                        );
                    }
                }
            }
        }

        {
            let mut group = c.benchmark_group(stringify!(op_fn));
            group.plot_config(plot_config.clone());

            duplicate_bigint_types! {
                paste! {
                    group.bench_function(
                        BenchmarkId::new(stringify!(label), bit_size),
                        |b| {
                            b.iter_batched_ref(
                                || (Encoded::from(big_sampler.sample(rng)), shift_sampler.sample(rng)),
                                |(r1, r2)| {
                                    black_box(Pow::pow(&*r1, *r2));
                                },
                                BatchSize::SmallInput,
                            );
                        },
                    );
                }
            }
        }
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
