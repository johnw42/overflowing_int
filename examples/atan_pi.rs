// Adapted from https://github.com/scymtym/sb-benchmarks/blob/master/cl-bench.bignum.benchmark.lisp
// Original code from Bruno Haible <haible@ilog.fr>

use std::env::args;

use overflowing_int::CowBigInt;
use num_traits::Zero;

fn pi_atan<'a>(k: CowBigInt<'a>, n: CowBigInt<'a>) -> CowBigInt<'a> {
    let mut a = CowBigInt::zero();
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

fn calc_pi_atan(digits: u32) -> CowBigInt<'static> {
    let n = digits;
    let m = n + 3;
    let tenpower = CowBigInt::from(10).pow(m);
    pi_atan(18.into(), &tenpower * CowBigInt::from(48))
        + pi_atan(57.into(), &tenpower * CowBigInt::from(32))
        - pi_atan(239.into(), &tenpower * CowBigInt::from(20))
}

fn main() {
    let num_digits = args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(1_000);
    let digits = calc_pi_atan(num_digits).to_string();
    println!("π ≈ {}.{}", &digits[0..1], &digits[1..]);
}
