// Adapted from https://github.com/scymtym/sb-benchmarks/blob/master/cl-bench.bignum.benchmark.lisp
// Original code from Bruno Haible <haible@ilog.fr>

use std::time::Duration;

use criterion::{
    AxisScale, BenchmarkId, Criterion, PlotConfiguration, criterion_group, criterion_main,
};

use num_bigint::BigUint;
use num_traits::Zero;
use overflowing_int::{ArcUint64, ArcUint128, CowUint128, EnumUint128, bench::IdentityBigUint};
use paste::paste;

macro_rules! duplicate_bigint_types {
    ($($body:tt)*) => {
        duplicate::duplicate! {
            [
                label         BigUint(lifetime);
                [Arc]         [ArcUint128];
                [ArcSize]     [ArcUint64];
                [Cow]         [CowUint128::<lifetime>];
                [Identity]    [IdentityBigUint];
                [Enum]        [EnumUint128];
                [Control]     [BigUint];
            ]
            $($body)*
        }
    }
}

duplicate_bigint_types! {
    paste! {
        fn [<calc_pi_atan_ label:lower>](digits: u32) -> BigUint(['static]) {
            fn pi_atan_rc<'a>(k: BigUint(['a]), n: BigUint(['a])) -> BigUint(['static]) {
                let mut a = BigUint(['static])::zero();
                let mut w = n * &k;
                let k2 = k.pow(2u32);
                let mut i = 1u32;
                while !w.is_zero() {
                    w /= &k2;
                    a += &w / i;
                    i += 2;
                    w /= &k2;
                    a -= &w / i;
                    i += 2;
                }
                a
            }

            let n = digits;
            let m = n + 3;
            let tenpower = BigUint(['static])::from(10u32).pow(m);
            pi_atan_rc(BigUint(['static])::from(18u32), &tenpower * BigUint(['static])::from(48u32))
                + pi_atan_rc(BigUint(['static])::from(57u32), &tenpower * BigUint(['static])::from(32u32))
                - pi_atan_rc(BigUint(['static])::from(239u32), &tenpower * BigUint(['static])::from(20u32))
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default();
    let plot_config = plot_config.summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("PiUnsigned");
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
