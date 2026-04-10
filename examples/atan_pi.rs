use compact_bigint::RcBigInt;
use num_traits::Zero;

fn pi_atan(k: RcBigInt, n: RcBigInt) -> RcBigInt {
    let mut a = RcBigInt::from(0);
    let mut w = n * k.clone();
    let k2 = k.clone() * k;
    let mut i = -1;
    while !w.is_zero() {
        w /= k2.clone();
        i += 2;
        a += w.clone() / i;
        w /= k2.clone();
        i += 2;
        a -= w.clone() / i;
    }
    a
}

fn calc_pi_atan(digits: u32) -> RcBigInt {
    let n = digits;
    let m = n + 3;
    let tenpower = RcBigInt::from(10).pow(m);
    pi_atan(18.into(), tenpower.clone() * RcBigInt::from(48))
        + pi_atan(57.into(), tenpower.clone() * RcBigInt::from(32))
        - pi_atan(239.into(), tenpower.clone() * RcBigInt::from(20))
}

fn main() {
    println!("{}", calc_pi_atan(1_000))
}
