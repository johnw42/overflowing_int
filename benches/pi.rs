// Adapted from https://github.com/scymtym/sb-benchmarks/blob/master/cl-bench.bignum.benchmark.lisp
// Original code from Bruno Haible <haible@ilog.fr>

use std::{borrow::Borrow, time::Duration};

use criterion::{
    AxisScale, BenchmarkId, Criterion, PlotConfiguration, criterion_group, criterion_main,
};

use compact_bigint::{ArcBigInt, ArcBigIsize, CowBigInt, EnumBigInt, bench::IdentityBigInt};
use num_bigint::BigInt;
use num_traits::Zero;
use paste::paste;

macro_rules! duplicate_bigint_types {
    ($($body:tt)*) => {
        duplicate::duplicate! {
            [
                label         BigInt(lifetime);
                [Arc]         [ArcBigInt];
                [ArcSize]     [ArcBigIsize];
                [Cow]         [CowBigInt::<lifetime>];
                [Identity]    [IdentityBigInt];
                [Enum]        [EnumBigInt];
                [Control]     [BigInt];
            ]
            $($body)*
        }
    }
}

duplicate_bigint_types! {
    paste! {
        fn [<calc_pi_atan_ label:lower>](digits: u32) -> BigInt(['static]) {
            fn pi_atan_rc<'a>(k: BigInt(['a]), n: BigInt(['a])) -> BigInt(['static]) {
                let mut a = BigInt(['static])::zero();
                let mut w = n * k.borrow();
                let k2 = k.pow(2);
                let mut i = -1;
                while !w.is_zero() {
                    w /= k2.borrow();
                    i += 2;
                    a += w.borrow() / i;
                    w /= k2.borrow();
                    i += 2;
                    a -= w.borrow() / i;
                }
                a
            }

            let n = digits;
            let m = n + 3;
            let tenpower = BigInt(['static])::from(10).pow(m);
            pi_atan_rc(BigInt(['static])::from(18), &tenpower * BigInt(['static])::from(48))
                + pi_atan_rc(BigInt(['static])::from(57), &tenpower * BigInt(['static])::from(32))
                - pi_atan_rc(BigInt(['static])::from(239), &tenpower * BigInt(['static])::from(20))
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default();
    let plot_config = plot_config.summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("Pi");
    group.plot_config(plot_config.clone());
    for digits in [10, 15, 20, 30, 40, 50, 100] {
        group.throughput(criterion::Throughput::Elements(digits as u64));
        duplicate_bigint_types! {
            paste! {
                group.bench_with_input(
                    BenchmarkId::new(stringify!(label), digits),
                    &digits,
                    |b, &digits| b.iter(|| [<calc_pi_atan_ label:lower>](digits)),
                );
            }
        }
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_secs(1));
    targets = criterion_benchmark
);
criterion_main!(benches);
