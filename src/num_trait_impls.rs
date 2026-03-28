use core::cmp::Ordering;
use core::ops::{Neg, Not};

use num_bigint::{BigInt, ParseBigIntError, Sign::*};
use num_integer::{Integer, Roots};
use num_traits::{Num, One, Signed, ToPrimitive, Zero};
use paste::paste;

use crate::Digit;
use crate::cbigint::CBigInt;
use crate::encoding::Encoded;

impl Zero for CBigInt {
    fn zero() -> Self {
        CBigInt(Encoded::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

#[test]
fn zero() {
    assert!(CBigInt::zero().is_zero());
    assert_eq!(CBigInt::zero(), CBigInt::from(0));
    assert!(!CBigInt::from(1).is_zero());
}

impl One for CBigInt {
    fn one() -> Self {
        CBigInt(Encoded::one())
    }

    fn is_one(&self) -> bool {
        self.0.is_one()
    }
}

#[test]
fn one() {
    assert!(CBigInt::one().is_one());
    assert_eq!(CBigInt::one(), CBigInt::from(1));
    assert!(!CBigInt::from(2).is_one());
}

impl Num for CBigInt {
    type FromStrRadixErr = ParseBigIntError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        BigInt::from_str_radix(str, radix).map(CBigInt::from)
    }
}

impl Signed for CBigInt {
    fn abs(&self) -> Self {
        match &self.0 {
            Encoded::Digit(a) => {
                if let (b, false) = a.overflowing_abs() {
                    b.into()
                } else {
                    BigInt::from(*a).abs().into()
                }
            }
            Encoded::Big(a) => a.abs().into(),
        }
    }

    fn abs_sub(&self, other: &Self) -> Self {
        (self - other).abs()
    }

    fn signum(&self) -> Self {
        match self.sign() {
            NoSign => 0,
            Plus => 1,
            Minus => -1,
        }
        .into()
    }

    fn is_positive(&self) -> bool {
        self.sign() == Plus
    }

    fn is_negative(&self) -> bool {
        self.sign() == Minus
    }
}

impl PartialOrd for CBigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CBigInt {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.0, &other.0) {
            (Encoded::Digit(a), Encoded::Digit(b)) => a.cmp(b),
            (Encoded::Big(a), Encoded::Big(b)) => a.cmp(b),
            _ => self
                .sign()
                .cmp(&other.sign())
                .then_with(|| self.to_bigint().cmp(&other.to_bigint())),
        }
    }
}

impl Integer for CBigInt {
    fn div_floor(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_digit_with(other)
            && (lhs, rhs) != (Digit::MIN, -1)
        {
            return Integer::div_floor(&lhs, &rhs).into();
        }
        self.to_bigint().div_floor(&*other.to_bigint()).into()
    }

    fn mod_floor(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_digit_with(other)
            && (lhs, rhs) != (Digit::MIN, -1)
        {
            return Integer::mod_floor(&lhs, &rhs).into();
        }
        self.to_bigint().mod_floor(&*other.to_bigint()).into()
    }

    fn gcd(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_digit_with(other) {
            return lhs.gcd(&rhs).into();
        }
        self.to_bigint().gcd(&*other.to_bigint()).into()
    }

    fn lcm(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_digit_with(other) {
            return lhs.lcm(&rhs).into();
        }
        self.to_bigint().lcm(&*other.to_bigint()).into()
    }

    fn divides(&self, other: &Self) -> bool {
        if let Some((lhs, rhs)) = self.to_digit_with(other) {
            return lhs.is_multiple_of(&rhs);
        }
        self.to_bigint().is_multiple_of(&*other.to_bigint())
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        if let Some((lhs, rhs)) = self.to_digit_with(other) {
            return lhs.is_multiple_of(&rhs);
        }
        self.to_bigint().is_multiple_of(&*other.to_bigint())
    }

    fn is_even(&self) -> bool {
        match &self.0 {
            Encoded::Digit(n) => n.is_even(),
            Encoded::Big(n) => n.is_even(),
        }
    }

    fn is_odd(&self) -> bool {
        match &self.0 {
            Encoded::Digit(n) => n.is_odd(),
            Encoded::Big(n) => n.is_odd(),
        }
    }

    fn div_rem(&self, other: &Self) -> (Self, Self) {
        if let Some((lhs, rhs)) = self.to_digit_with(other)
            && (lhs, rhs) != (Digit::MIN, -1)
        {
            let (q, r) = lhs.div_rem(&rhs);
            return (q.into(), r.into());
        }
        let (q, r) = self.to_bigint().div_rem(&*other.to_bigint());
        (q.into(), r.into())
    }
}

#[test]
fn gcd_test() {
    let small = CBigInt::from(5);
    let huge = CBigInt::from(i128::MAX).pow(2);
    assert_eq!(huge.gcd(&small), CBigInt::from(1));
    assert_eq!(small.gcd(&huge), CBigInt::from(1));
}

impl Roots for CBigInt {
    fn nth_root(&self, n: u32) -> Self {
        match &self.0 {
            Encoded::Digit(a) => a.nth_root(n).into(),
            Encoded::Big(a) => a.nth_root(n).into(),
        }
    }
}

impl Neg for CBigInt {
    type Output = CBigInt;

    fn neg(self) -> Self::Output {
        if let Some(a) = self.to_digit()
            && let (b, false) = a.overflowing_neg()
        {
            return b.into();
        }
        BigInt::from(self).neg().into()
    }
}

impl Neg for &CBigInt {
    type Output = CBigInt;

    fn neg(self) -> Self::Output {
        if let Some(a) = self.to_digit()
            && let (b, false) = a.overflowing_neg()
        {
            return b.into();
        }
        (&*self.to_bigint()).neg().into()
    }
}

impl Not for CBigInt {
    type Output = CBigInt;

    fn not(self) -> Self::Output {
        match self.0 {
            Encoded::Digit(n) => n.not().into(),
            Encoded::Big(n) => n.not().into(),
        }
    }
}

impl Not for &CBigInt {
    type Output = CBigInt;

    fn not(self) -> Self::Output {
        match &self.0 {
            Encoded::Digit(n) => n.not().into(),
            Encoded::Big(n) => n.not().into(),
        }
    }
}

impl ToPrimitive for CBigInt {
    #[duplicate::duplicate_item(
        prim;
        [i8];
        [i16];
        [i32];
        [i64];
        [i128];
        [isize];
        [u8];
        [u16];
        [u32];
        [u64];
        [u128];
        [usize];
        [f32];
        [f64];
    )]
    paste! {
        fn [< to_ prim >](&self) -> Option<prim> {
            match &self.0 {
                Encoded::Digit(value) => value.[< to_ prim >](),
                Encoded::Big(value) => value.[< to_ prim >](),
            }
        }
    }
}
