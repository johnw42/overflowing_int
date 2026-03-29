use core::cmp::Ordering;
use core::ops::{Neg, Not};
use std::fmt::{Binary, Formatter, LowerHex, Octal, UpperHex};
use std::hash::Hash;
use std::panic::RefUnwindSafe;
use std::str::FromStr;

use num_bigint::{BigInt, ParseBigIntError, RandomBits, Sign::*, UniformBigInt};
use num_integer::{Integer, Roots};
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedEuclid, CheckedMul, CheckedSub, ConstZero, Euclid, FromBytes,
    FromPrimitive, Num, One, Signed, ToBytes, ToPrimitive, Zero,
};
use paste::paste;
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformSampler};
use rand::prelude::Distribution;
use serde::{Deserialize, Serialize};

use crate::Digit;
use crate::big_integer::BigInteger as _;
use crate::cbigint::CBigInt;
use crate::encoding::Encoded;

impl quickcheck::Arbitrary for CBigInt {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        CBigInt::from(BigInt::arbitrary(g))
    }
}

impl arbitrary::Arbitrary<'_> for CBigInt {
    fn arbitrary(g: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        Ok(CBigInt::from(BigInt::arbitrary(g)?))
    }
}

impl Binary for CBigInt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match &self.0 {
            Encoded::Digit(n) => Binary::fmt(n, f),
            Encoded::Big(n) => Binary::fmt(n, f),
        }
    }
}

impl CheckedAdd for CBigInt {
    fn checked_add(&self, v: &CBigInt) -> Option<CBigInt> {
        Some(self + v)
    }
}

impl CheckedDiv for CBigInt {
    fn checked_div(&self, v: &CBigInt) -> Option<CBigInt> {
        Some(self / v)
    }
}

impl CheckedEuclid for CBigInt {
    fn checked_rem_euclid(&self, v: &CBigInt) -> Option<CBigInt> {
        self.to_bigint()
            .checked_rem_euclid(&v.to_bigint())
            .map(Into::into)
    }

    fn checked_div_euclid(&self, v: &Self) -> Option<Self> {
        self.to_bigint()
            .checked_div_euclid(&v.to_bigint())
            .map(Into::into)
    }
}

impl CheckedMul for CBigInt {
    fn checked_mul(&self, v: &CBigInt) -> Option<CBigInt> {
        Some(self * v)
    }
}

impl CheckedSub for CBigInt {
    fn checked_sub(&self, v: &CBigInt) -> Option<CBigInt> {
        Some(self - v)
    }
}

impl ConstZero for CBigInt {
    const ZERO: Self = CBigInt(Encoded::zero());
}

impl<'de> Deserialize<'de> for CBigInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        BigInt::deserialize(deserializer).map(CBigInt::from)
    }
}

impl Distribution<CBigInt> for RandomBits {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> CBigInt {
        <RandomBits as Distribution<BigInt>>::sample(self, rng).into()
    }
}

impl Euclid for CBigInt {
    fn rem_euclid(&self, v: &CBigInt) -> CBigInt {
        self.to_bigint().rem_euclid(&v.to_bigint()).into()
    }

    fn div_euclid(&self, v: &CBigInt) -> CBigInt {
        self.to_bigint().div_euclid(&v.to_bigint()).into()
    }
}

impl FromBytes for CBigInt {
    type Bytes = [u8];

    fn from_be_bytes(bytes: &[u8]) -> Self {
        Self::from_signed_bytes_be(bytes)
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        Self::from_signed_bytes_le(bytes)
    }
}

impl FromPrimitive for CBigInt {
    fn from_i64(n: i64) -> Option<Self> {
        Some(CBigInt::from(n))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(CBigInt::from(n))
    }
}

impl FromStr for CBigInt {
    type Err = num_bigint::ParseBigIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BigInt::from_str(s).map(Self::from)
    }
}

impl Hash for CBigInt {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.normalized().hash(state)
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

impl LowerHex for CBigInt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match &self.0 {
            Encoded::Digit(n) => LowerHex::fmt(n, f),
            Encoded::Big(n) => LowerHex::fmt(n, f),
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

impl Num for CBigInt {
    type FromStrRadixErr = ParseBigIntError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        BigInt::from_str_radix(str, radix).map(CBigInt::from)
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

impl Octal for CBigInt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match &self.0 {
            Encoded::Digit(n) => Octal::fmt(n, f),
            Encoded::Big(n) => Octal::fmt(n, f),
        }
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

impl One for CBigInt {
    fn one() -> Self {
        CBigInt(Encoded::one())
    }

    fn is_one(&self) -> bool {
        self.0.is_one()
    }
}

impl PartialEq for CBigInt {
    fn eq(&self, other: &Self) -> bool {
        self.normalized().0 == other.normalized().0
    }
}

impl PartialOrd for CBigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl RefUnwindSafe for CBigInt {}

impl Roots for CBigInt {
    fn nth_root(&self, n: u32) -> Self {
        match &self.0 {
            Encoded::Digit(a) => a.nth_root(n).into(),
            Encoded::Big(a) => a.nth_root(n).into(),
        }
    }
}

impl SampleUniform for CBigInt {
    type Sampler = UniformCBigInt;
}

pub struct UniformCBigInt(UniformBigInt);

impl UniformSampler for UniformCBigInt {
    type X = CBigInt;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self(UniformBigInt::new(
            low.borrow().to_bigint().borrow(),
            high.borrow().to_bigint().borrow(),
        ))
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self(UniformBigInt::new_inclusive(
            low.borrow().to_bigint().borrow(),
            high.borrow().to_bigint().borrow(),
        ))
    }

    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        self.0.sample(rng).into()
    }
}

impl Serialize for CBigInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_bigint().serialize(serializer)
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

impl ToBytes for CBigInt {
    type Bytes = Vec<u8>;

    fn to_be_bytes(&self) -> Self::Bytes {
        match &self.0 {
            Encoded::Digit(n) => n.to_be_bytes().to_vec(),
            Encoded::Big(n) => n.to_be_bytes(),
        }
    }

    fn to_le_bytes(&self) -> Self::Bytes {
        match &self.0 {
            Encoded::Digit(n) => n.to_le_bytes().to_vec(),
            Encoded::Big(n) => n.to_le_bytes(),
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

impl UpperHex for CBigInt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match &self.0 {
            Encoded::Digit(n) => UpperHex::fmt(n, f),
            Encoded::Big(n) => UpperHex::fmt(n, f),
        }
    }
}

impl Zero for CBigInt {
    fn zero() -> Self {
        CBigInt(Encoded::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

#[test]
fn test_gcd() {
    let small = CBigInt::from(5);
    let huge = CBigInt::from(i128::MAX).pow(2);
    assert_eq!(huge.gcd(&small), CBigInt::from(1));
    assert_eq!(small.gcd(&huge), CBigInt::from(1));
}

#[test]
fn test_one() {
    assert!(CBigInt::one().is_one());
    assert_eq!(CBigInt::one(), CBigInt::from(1));
    assert!(!CBigInt::from(2).is_one());
}

#[test]
fn test_zero() {
    assert!(CBigInt::zero().is_zero());
    assert_eq!(CBigInt::zero(), CBigInt::from(0));
    assert!(!CBigInt::from(1).is_zero());
}
