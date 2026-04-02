use core::cmp::Ordering;
use core::ops::{Neg, Not};
use std::fmt::{Binary, Formatter, LowerHex, Octal, UpperHex};
use std::panic::RefUnwindSafe;
use std::str::FromStr;

use num_bigint::{
    BigInt, ParseBigIntError, RandomBits,
    Sign::{self, *},
    UniformBigInt,
};
use num_integer::{Integer, Roots};
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedEuclid, CheckedMul, CheckedSub, ConstZero, Euclid, FromBytes,
    FromPrimitive, Num, One, Signed, ToBytes, ToPrimitive, Zero,
};
use paste::paste;
use quickcheck_macros::quickcheck;
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformSampler};
use rand::prelude::Distribution;
use serde::{Deserialize, Serialize};

use crate::rc_bigint::RcBigInt;
use crate::rc_bigint::encoding::RefEncoding;

impl quickcheck::Arbitrary for RcBigInt {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        match bool::arbitrary(g) {
            true => RcBigInt::from(i128::arbitrary(g)),
            false => RcBigInt::from(BigInt::arbitrary(g)),
        }
    }
}

impl arbitrary::Arbitrary<'_> for RcBigInt {
    fn arbitrary(g: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        match bool::arbitrary(g)? {
            true => Ok(RcBigInt::from(i128::arbitrary(g)?)),
            false => Ok(RcBigInt::from(BigInt::arbitrary(g)?)),
        }
    }
}

impl Binary for RcBigInt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.decode_ref() {
            RefEncoding::Small(n) => Binary::fmt(&n, f),
            RefEncoding::Big(n) => Binary::fmt(n, f),
        }
    }
}

impl CheckedAdd for RcBigInt {
    fn checked_add(&self, v: &Self) -> Option<Self> {
        self.checked_add(v)
    }
}

impl CheckedDiv for RcBigInt {
    fn checked_div(&self, v: &Self) -> Option<Self> {
        self.checked_div(v)
    }
}

impl CheckedEuclid for RcBigInt {
    fn checked_rem_euclid(&self, v: &Self) -> Option<Self> {
        self.to_cow()
            .checked_rem_euclid(&v.to_cow())
            .map(Into::into)
    }

    fn checked_div_euclid(&self, v: &Self) -> Option<Self> {
        self.to_cow()
            .checked_div_euclid(&v.to_cow())
            .map(Into::into)
    }
}

impl CheckedMul for RcBigInt {
    fn checked_mul(&self, v: &Self) -> Option<Self> {
        self.checked_mul(v)
    }
}

impl CheckedSub for RcBigInt {
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        self.checked_sub(v)
    }
}

impl ConstZero for RcBigInt {
    const ZERO: Self = Self(Encoded::from_small(0));
}

impl<'a, 'de> Deserialize<'de> for RcBigInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        BigInt::deserialize(deserializer).map(RcBigInt::from)
    }
}

impl<'a> Distribution<RcBigInt> for RandomBits {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> RcBigInt {
        <RandomBits as Distribution<BigInt>>::sample(self, rng).into()
    }
}

impl Euclid for RcBigInt {
    fn rem_euclid(&self, v: &Self) -> Self {
        self.to_cow().rem_euclid(&v.to_cow()).into()
    }

    fn div_euclid(&self, v: &Self) -> Self {
        self.to_cow().div_euclid(&v.to_cow()).into()
    }
}

impl FromBytes for RcBigInt {
    type Bytes = [u8];

    fn from_be_bytes(bytes: &[u8]) -> Self {
        Self::from_signed_bytes_be(bytes)
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        Self::from_signed_bytes_le(bytes)
    }
}

impl FromPrimitive for RcBigInt {
    fn from_i64(n: i64) -> Option<Self> {
        Some(RcBigInt::from(n))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(RcBigInt::from(n))
    }
}

impl FromStr for RcBigInt {
    type Err = num_bigint::ParseBigIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BigInt::from_str(s).map(Self::from)
    }
}

impl Integer for RcBigInt {
    fn div_floor(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other)
            && (lhs, rhs) != (SmallInt::MIN, -1)
        {
            return Integer::div_floor(&lhs, &rhs).into();
        }
        self.to_cow().div_floor(&other.to_cow()).into()
    }

    fn mod_floor(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other)
            && (lhs, rhs) != (SmallInt::MIN, -1)
        {
            return Integer::mod_floor(&lhs, &rhs).into();
        }
        self.to_cow().mod_floor(&other.to_cow()).into()
    }

    fn gcd(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.gcd(&rhs).into();
        }
        self.to_cow().gcd(&other.to_cow()).into()
    }

    fn lcm(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.lcm(&rhs).into();
        }
        self.to_cow().lcm(&other.to_cow()).into()
    }

    fn divides(&self, other: &Self) -> bool {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.is_multiple_of(&rhs);
        }
        self.to_cow().is_multiple_of(&other.to_cow())
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.is_multiple_of(&rhs);
        }
        self.to_cow().is_multiple_of(&other.to_cow())
    }

    fn is_even(&self) -> bool {
        match self.decode_ref() {
            RefEncoding::Small(n) => n.is_even(),
            RefEncoding::Big(n) => n.is_even(),
        }
    }

    fn is_odd(&self) -> bool {
        match self.decode_ref() {
            RefEncoding::Small(n) => n.is_odd(),
            RefEncoding::Big(n) => n.is_odd(),
        }
    }

    fn div_rem(&self, other: &Self) -> (Self, Self) {
        if let Some((lhs, rhs)) = self.to_small_with(other)
            && (lhs, rhs) != (SmallInt::MIN, -1)
        {
            let (q, r) = lhs.div_rem(&rhs);
            return (q.into(), r.into());
        }
        let (q, r) = self.to_cow().div_rem(&other.to_cow());
        (q.into(), r.into())
    }
}

impl LowerHex for RcBigInt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.decode_ref() {
            RefEncoding::Small(n) => LowerHex::fmt(&n, f),
            RefEncoding::Big(n) => LowerHex::fmt(n, f),
        }
    }
}

impl<'a> Neg for RcBigInt {
    type Output = RcBigInt;

    fn neg(self) -> Self::Output {
        if let Some(a) = self.to_small()
            && let (b, false) = a.overflowing_neg()
        {
            return b.into();
        }
        BigInt::from(self).neg().into()
    }
}

impl<'a> Neg for &RcBigInt {
    type Output = RcBigInt;

    fn neg(self) -> Self::Output {
        if let Some(a) = self.to_small()
            && let (b, false) = a.overflowing_neg()
        {
            return b.into();
        }
        (&*self.to_cow()).neg().into()
    }
}

impl Num for RcBigInt {
    type FromStrRadixErr = ParseBigIntError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        BigInt::from_str_radix(str, radix).map(RcBigInt::from)
    }
}

impl<'a> Not for RcBigInt {
    type Output = RcBigInt;

    fn not(self) -> Self::Output {
        match self.into_encoding() {
            Encoding::Small(n) => Encoded::from_small(n.not()),
            Encoding::Big(n) => Encoded::from_big(n.into_owned().not()),
        }
        .into()
    }
}

impl<'a> Not for &RcBigInt {
    type Output = RcBigInt;

    fn not(self) -> Self::Output {
        match self.into_encoding() {
            Encoding::Small(n) => Encoded::from_small(n.not()),
            Encoding::Big(n) => Encoded::from_big(n.borrow().not()),
        }
        .into()
    }
}

impl Octal for RcBigInt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.decode_ref() {
            Encoding::Small(n) => Octal::fmt(n, f),
            Encoding::Big(n) => Octal::fmt(n.borrow(), f),
        }
    }
}

impl Ord for RcBigInt {
    fn cmp(&self, other: &Self) -> Ordering {
        use Encoding::*;
        use Ordering::*;
        use Sign::*;

        match (&self.decode_ref(), other.decode_ref()) {
            (Small(a), Small(b)) => a.cmp(b),
            (Small(a), Big(b)) => match (a.cmp(&0), b.sign()) {
                (_, Minus) => Greater,
                (_, Plus) => Less,
                (Equal, NoSign) => Equal,
                (Less, NoSign) => Less,
                (Greater, NoSign) => Greater,
            },
            (Big(a), Small(b)) => match (a.sign(), b.cmp(&0)) {
                (Plus, _) => Greater,
                (Minus, _) => Less,
                (NoSign, Less) => Greater,
                (NoSign, Equal) => Equal,
                (NoSign, Greater) => Less,
            },
            (Big(a), Big(b)) => a.cmp(b),
        }
    }
}

#[quickcheck]
fn test_round_trip1(a: RcBigInt) -> bool {
    RcBigInt::from(BigInt::from(a.clone())) == a && a.clone() == RcBigInt::from(BigInt::from(a))
}

#[quickcheck]
fn test_round_trip2(a: BigInt) -> bool {
    BigInt::from(RcBigInt::from(a.clone())) == a && a.clone() == BigInt::from(RcBigInt::from(a))
}

#[quickcheck]
fn test_to_string(a: RcBigInt) -> bool {
    a.to_string() == BigInt::from(a).to_string()
}

#[quickcheck]
fn test_ord(a: RcBigInt, b: RcBigInt) -> bool {
    a.cmp(&b) == BigInt::from(a).cmp(&BigInt::from(b))
}

impl<'a> One for RcBigInt {
    fn one() -> Self {
        RcBigInt(Encoded::from_small(1))
    }

    fn is_one(&self) -> bool {
        self.encoding() == Encoded::from_small(1).encoding()
    }
}

impl<'a> PartialOrd for RcBigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> RefUnwindSafe for RcBigInt {}

impl<'a> Roots for RcBigInt {
    fn nth_root(&self, n: u32) -> Self {
        match self.into_encoding() {
            Encoding::Small(a) => a.nth_root(n).into(),
            Encoding::Big(a) => a.nth_root(n).into(),
        }
    }
}

impl SampleUniform for RcBigInt {
    type Sampler = UniformCBigInt;
}

pub struct UniformCBigInt(UniformBigInt);

impl UniformSampler for UniformCBigInt {
    type X = RcBigInt;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self(UniformBigInt::new(
            low.borrow().to_cow().borrow(),
            high.borrow().to_cow().borrow(),
        ))
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self(UniformBigInt::new_inclusive(
            low.borrow().to_cow().borrow(),
            high.borrow().to_cow().borrow(),
        ))
    }

    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        self.0.sample(rng).into()
    }
}

impl<'a> Serialize for RcBigInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_cow().serialize(serializer)
    }
}

impl<'a> Signed for RcBigInt {
    fn abs(&self) -> Self {
        match self.decode_ref() {
            Encoding::Small(a) => {
                if let (b, false) = a.overflowing_abs() {
                    b.into()
                } else {
                    BigInt::from(*a).abs().into()
                }
            }
            Encoding::Big(a) => a.abs().into(),
        }
    }

    fn abs_sub(&self, other: &Self) -> Self {
        (self.clone() - other.clone()).abs()
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

impl<'a> ToBytes for RcBigInt {
    type Bytes = Vec<u8>;

    fn to_be_bytes(&self) -> Self::Bytes {
        match self.decode_ref() {
            Encoding::Small(n) => n.to_be_bytes().to_vec(),
            Encoding::Big(n) => n.to_be_bytes(),
        }
    }

    fn to_le_bytes(&self) -> Self::Bytes {
        match self.decode_ref() {
            Encoding::Small(n) => n.to_le_bytes().to_vec(),
            Encoding::Big(n) => n.to_le_bytes(),
        }
    }
}

impl<'a> ToPrimitive for RcBigInt {
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
            match self.decode_ref() {
                Encoding::Small(value) => value.[< to_ prim >](),
                Encoding::Big(value) => value.[< to_ prim >](),
            }
        }
    }
}

impl<'a> UpperHex for RcBigInt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.decode_ref() {
            Encoding::Small(n) => UpperHex::fmt(n, f),
            Encoding::Big(n) => UpperHex::fmt(n.borrow(), f),
        }
    }
}

impl<'a> Zero for RcBigInt {
    fn zero() -> Self {
        RcBigInt(Encoded::from_small(0))
    }

    fn is_zero(&self) -> bool {
        self.encoding() == Encoded::from_small(0).encoding()
    }
}

#[test]
fn test_gcd() {
    let small = RcBigInt::from(5);
    let huge = RcBigInt::from(i128::MAX).pow(2);
    assert_eq!(huge.gcd(&small), RcBigInt::from(1));
    assert_eq!(small.gcd(&huge), RcBigInt::from(1));
}

#[test]
fn test_one() {
    assert!(RcBigInt::one().is_one());
    assert_eq!(RcBigInt::one(), RcBigInt::from(1));
    assert!(!RcBigInt::from(2).is_one());
}

#[test]
fn test_zero() {
    assert!(RcBigInt::zero().is_zero());
    assert_eq!(RcBigInt::zero(), RcBigInt::from(0));
    assert!(!RcBigInt::from(1).is_zero());
}
