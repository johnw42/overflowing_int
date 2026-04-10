use criterion::{
    AxisScale, BenchmarkId, Criterion, PlotConfiguration, criterion_group, criterion_main,
};

use compact_bigint::{BoxBigInt, CowBigInt, RcBigInt, RcBigIsize, TrivialBigInt};
use duplicate::duplicate;
use num_bigint::BigInt;
use num_traits::Zero;
use paste::paste;

duplicate! {
    [
        label         BigInt;
        [bigint]      [BigInt];
        [rcbigint]    [RcBigInt];
        [rcbigisize]  [RcBigIsize];
        [cow]         [CowBigInt::<'static>];
        [trivial]     [TrivialBigInt];
        [box]         [BoxBigInt];
    ]
    paste! {
        fn [<calc_pi_atan_rc_ label>](digits: u32) -> BigInt {
            fn pi_atan_rc(k: BigInt, n: BigInt) -> BigInt {
                let mut a = BigInt::zero();
                let mut w = n * &k;
                let k2 = &k * &k;
                let mut i = -1;
                while !w.is_zero() {
                    w /= &k2;
                    i += 2;
                    a += &w / i;
                    w /= &k2;
                    i += 2;
                    a -= &w / i;
                }
                a
            }

            let n = digits;
            let m = n + 3;
            let tenpower = BigInt::from(10).pow(m);
            pi_atan_rc(BigInt::from(18), &tenpower * BigInt::from(48))
                + pi_atan_rc(BigInt::from(57), &tenpower * BigInt::from(32))
                - pi_atan_rc(BigInt::from(239), &tenpower * BigInt::from(20))
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
        group.bench_with_input(
            BenchmarkId::new("Control", digits),
            &digits,
            |b, &digits| b.iter(|| calc_pi_atan_rc_bigint(digits)),
        );
        group.bench_with_input(BenchmarkId::new("Rc", digits), &digits, |b, &digits| {
            b.iter(|| calc_pi_atan_rc_rcbigint(digits))
        });
        group.bench_with_input(
            BenchmarkId::new("RcIsize", digits),
            &digits,
            |b, &digits| b.iter(|| calc_pi_atan_rc_rcbigisize(digits)),
        );
        group.bench_with_input(BenchmarkId::new("Cow", digits), &digits, |b, &digits| {
            b.iter(|| calc_pi_atan_rc_cow(digits))
        });
        group.bench_with_input(
            BenchmarkId::new("Trivial", digits),
            &digits,
            |b, &digits| b.iter(|| calc_pi_atan_rc_trivial(digits)),
        );
        group.bench_with_input(BenchmarkId::new("Box", digits), &digits, |b, &digits| {
            b.iter(|| calc_pi_atan_rc_box(digits))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
