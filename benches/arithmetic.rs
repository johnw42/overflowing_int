use criterion::{
    BatchSize, BenchmarkId, Criterion, PlotConfiguration, criterion_group, criterion_main,
};
use num_integer::Integer;
use num_traits::{CheckedAdd, CheckedDiv, CheckedEuclid, CheckedMul, CheckedSub, Euclid, Pow};
use paste::paste;
use rand::rngs::ThreadRng;
use rand::{Rng, thread_rng};
use std::hint::black_box;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};
use std::time::Duration;

use compact_bigint::bench::*;
use compact_bigint::*;

macro_rules! duplicate_bigint_types {
    ($($body:tt)*) => {
        duplicate::duplicate! {
            [
                label         Encoded;
                [Arc]         [ArcBigInt];
                [ArcSize]     [ArcBigIsize];
                [Box]         [BoxBigInt];
                [Cow]         [CowBigInt::<'static>];
                [Identity]    [IdentityBigInt::<'static>];
                [Rc]          [RcBigInt];
                [RcSize]      [RcBigIsize];
                [Enum]        [EnumBigInt];
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
                [Integer]    [gcd];
                [Integer]    [lcm];
                [Integer]    [is_multiple_of];
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
    let plot_config = PlotConfiguration::default();
    let rng = &mut thread_rng();

    fn sample_big(rng: &mut ThreadRng, bit_size: u32) -> BigInt {
        let limit: BigInt = BigInt::from(1) << (bit_size - 1);
        rng.gen_range(-limit.clone()..=limit.clone())
    }

    fn sample_small(rng: &mut ThreadRng) -> u32 {
        rng.gen_range(0u32..=64u32)
    }

    macro_rules! group {
        ($op_fn:ident, $bit_size:ident, $args:pat_param, $arg_values:expr, $to_bench:expr) => {{
            let mut group = c.benchmark_group(stringify!($op_fn));
            group.plot_config(plot_config.clone());

            for $bit_size in [126, 127, 128, 129, 130] {
                duplicate_bigint_types! { paste! {
                    group.bench_function(
                        BenchmarkId::new(stringify!(label), $bit_size),
                        |b| {
                            b.iter_batched_ref(
                                || $arg_values,
                                |$args| {
                                    black_box($to_bench);
                                },
                                BatchSize::SmallInput,
                            );
                        },
                    );
                } }
            }
        }};
    }

    duplicate_binary_operators! {
        group!(
            op_fn,
            bit_size,
            (r1, r2),
            (
                Encoded::from(sample_big(rng, bit_size)),
                Encoded::from(sample_big(rng, bit_size))
            ),
            OpTrait::op_fn(&*r1, &*r2)
        );
    }

    duplicate_shift_operators! {
        group!(
            op_fn,
            bit_size,
            (r1, r2),
            (
                Encoded::from(sample_big(rng, bit_size)),
                sample_small(rng)
            ),
            OpTrait::op_fn(&*r1, *r2)
        );
    }

    group!(
        pow,
        bit_size,
        (r1, r2),
        (Encoded::from(sample_big(rng, bit_size)), sample_small(rng)),
        Pow::pow(&*r1, *r2)
    );
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_secs(1));
    targets = criterion_benchmark
);
criterion_main!(benches);
