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

use crate::rc_bignum::GenericBigNum;
use crate::rc_bignum::encoding::RefEncoding;

impl quickcheck::Arbitrary for GenericBigNum {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        match bool::arbitrary(g) {
            true => GenericBigNum::from(i128::arbitrary(g)),
            false => GenericBigNum::from(BigInt::arbitrary(g)),
        }
    }
}

impl arbitrary::Arbitrary<'_> for GenericBigNum {
    fn arbitrary(g: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        match bool::arbitrary(g)? {
            true => Ok(GenericBigNum::from(i128::arbitrary(g)?)),
            false => Ok(GenericBigNum::from(BigInt::arbitrary(g)?)),
        }
    }
}

impl Binary for GenericBigNum {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.decode_ref() {
            RefEncoding::Small(n) => Binary::fmt(&n, f),
            RefEncoding::Big(n) => Binary::fmt(n, f),
        }
    }
}

impl CheckedAdd for GenericBigNum {
    fn checked_add(&self, v: &Self) -> Option<Self> {
        self.checked_add(v)
    }
}

impl CheckedDiv for GenericBigNum {
    fn checked_div(&self, v: &Self) -> Option<Self> {
        self.checked_div(v)
    }
}

impl CheckedEuclid for GenericBigNum {
    fn checked_rem_euclid(&self, v: &Self) -> Option<Self> {
        self.big_cow()
            .checked_rem_euclid(&v.big_cow())
            .map(Into::into)
    }

    fn checked_div_euclid(&self, v: &Self) -> Option<Self> {
        self.big_cow()
            .checked_div_euclid(&v.big_cow())
            .map(Into::into)
    }
}

impl CheckedMul for GenericBigNum {
    fn checked_mul(&self, v: &Self) -> Option<Self> {
        self.checked_mul(v)
    }
}

impl CheckedSub for GenericBigNum {
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        self.checked_sub(v)
    }
}

impl ConstZero for GenericBigNum {
    const ZERO: Self = Self(Encoded::from_small(0));
}

impl<'a, 'de> Deserialize<'de> for GenericBigNum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        BigInt::deserialize(deserializer).map(GenericBigNum::from)
    }
}

impl<'a> Distribution<GenericBigNum> for RandomBits {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> GenericBigNum {
        <RandomBits as Distribution<BigInt>>::sample(self, rng).into()
    }
}

impl Euclid for GenericBigNum {
    fn rem_euclid(&self, v: &Self) -> Self {
        self.big_cow().rem_euclid(&v.big_cow()).into()
    }

    fn div_euclid(&self, v: &Self) -> Self {
        self.big_cow().div_euclid(&v.big_cow()).into()
    }
}

impl FromBytes for GenericBigNum {
    type Bytes = [u8];

    fn from_be_bytes(bytes: &[u8]) -> Self {
        Self::from_signed_bytes_be(bytes)
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        Self::from_signed_bytes_le(bytes)
    }
}

impl FromPrimitive for GenericBigNum {
    fn from_i64(n: i64) -> Option<Self> {
        Some(GenericBigNum::from(n))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(GenericBigNum::from(n))
    }
}

impl FromStr for GenericBigNum {
    type Err = num_bigint::ParseBigIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BigInt::from_str(s).map(Self::from)
    }
}

impl Integer for GenericBigNum {
    fn div_floor(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other)
            && (lhs, rhs) != (SmallInt::MIN, -1)
        {
            return Integer::div_floor(&lhs, &rhs).into();
        }
        self.big_cow().div_floor(&other.big_cow()).into()
    }

    fn mod_floor(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other)
            && (lhs, rhs) != (SmallInt::MIN, -1)
        {
            return Integer::mod_floor(&lhs, &rhs).into();
        }
        self.big_cow().mod_floor(&other.big_cow()).into()
    }

    fn gcd(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.gcd(&rhs).into();
        }
        self.big_cow().gcd(&other.big_cow()).into()
    }

    fn lcm(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.lcm(&rhs).into();
        }
        self.big_cow().lcm(&other.big_cow()).into()
    }

    fn divides(&self, other: &Self) -> bool {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.is_multiple_of(&rhs);
        }
        self.big_cow().is_multiple_of(&other.big_cow())
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.is_multiple_of(&rhs);
        }
        self.big_cow().is_multiple_of(&other.big_cow())
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
        let (q, r) = self.big_cow().div_rem(&other.big_cow());
        (q.into(), r.into())
    }
}

impl LowerHex for GenericBigNum {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.decode_ref() {
            RefEncoding::Small(n) => LowerHex::fmt(&n, f),
            RefEncoding::Big(n) => LowerHex::fmt(n, f),
        }
    }
}

impl<'a> Neg for GenericBigNum {
    type Output = GenericBigNum;

    fn neg(self) -> Self::Output {
        if let Some(a) = self.to_small()
            && let (b, false) = a.overflowing_neg()
        {
            return b.into();
        }
        BigInt::from(self).neg().into()
    }
}

impl<'a> Neg for &GenericBigNum {
    type Output = GenericBigNum;

    fn neg(self) -> Self::Output {
        if let Some(a) = self.to_small()
            && let (b, false) = a.overflowing_neg()
        {
            return b.into();
        }
        (&*self.big_cow()).neg().into()
    }
}

impl Num for GenericBigNum {
    type FromStrRadixErr = ParseBigIntError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        BigInt::from_str_radix(str, radix).map(GenericBigNum::from)
    }
}

impl<'a> Not for GenericBigNum {
    type Output = GenericBigNum;

    fn not(self) -> Self::Output {
        match self.into_encoding() {
            Encoding::Small(n) => Encoded::from_small(n.not()),
            Encoding::Big(n) => Encoded::from_big(n.into_owned().not()),
        }
        .into()
    }
}

impl<'a> Not for &GenericBigNum {
    type Output = GenericBigNum;

    fn not(self) -> Self::Output {
        match self.into_encoding() {
            Encoding::Small(n) => Encoded::from_small(n.not()),
            Encoding::Big(n) => Encoded::from_big(n.borrow().not()),
        }
        .into()
    }
}

impl Octal for GenericBigNum {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.decode_ref() {
            Encoding::Small(n) => Octal::fmt(n, f),
            Encoding::Big(n) => Octal::fmt(n.borrow(), f),
        }
    }
}

impl Ord for GenericBigNum {
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
fn test_round_trip1(a: GenericBigNum) -> bool {
    GenericBigNum::from(BigInt::from(a.clone())) == a
        && a.clone() == GenericBigNum::from(BigInt::from(a))
}

#[quickcheck]
fn test_round_trip2(a: BigInt) -> bool {
    BigInt::from(GenericBigNum::from(a.clone())) == a
        && a.clone() == BigInt::from(GenericBigNum::from(a))
}

#[quickcheck]
fn test_to_string(a: GenericBigNum) -> bool {
    a.to_string() == BigInt::from(a).to_string()
}

#[quickcheck]
fn test_ord(a: GenericBigNum, b: GenericBigNum) -> bool {
    a.cmp(&b) == BigInt::from(a).cmp(&BigInt::from(b))
}

impl<'a> One for GenericBigNum {
    fn one() -> Self {
        GenericBigNum(Encoded::from_small(1))
    }

    fn is_one(&self) -> bool {
        self.encoding() == Encoded::from_small(1).encoding()
    }
}

impl<'a> PartialOrd for GenericBigNum {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> RefUnwindSafe for GenericBigNum {}

impl<'a> Roots for GenericBigNum {
    fn nth_root(&self, n: u32) -> Self {
        match self.into_encoding() {
            Encoding::Small(a) => a.nth_root(n).into(),
            Encoding::Big(a) => a.nth_root(n).into(),
        }
    }
}

impl SampleUniform for GenericBigNum {
    type Sampler = UniformCBigInt;
}

pub struct UniformCBigInt(UniformBigInt);

impl UniformSampler for UniformCBigInt {
    type X = GenericBigNum;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self(UniformBigInt::new(
            low.borrow().big_cow().borrow(),
            high.borrow().big_cow().borrow(),
        ))
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self(UniformBigInt::new_inclusive(
            low.borrow().big_cow().borrow(),
            high.borrow().big_cow().borrow(),
        ))
    }

    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        self.0.sample(rng).into()
    }
}

impl<'a> Serialize for GenericBigNum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.big_cow().serialize(serializer)
    }
}

impl<'a> Signed for GenericBigNum {
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

impl<'a> ToBytes for GenericBigNum {
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

impl<'a> ToPrimitive for GenericBigNum {
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

impl<'a> UpperHex for GenericBigNum {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.decode_ref() {
            Encoding::Small(n) => UpperHex::fmt(n, f),
            Encoding::Big(n) => UpperHex::fmt(n.borrow(), f),
        }
    }
}

impl<'a> Zero for GenericBigNum {
    fn zero() -> Self {
        GenericBigNum(Encoded::from_small(0))
    }

    fn is_zero(&self) -> bool {
        self.encoding() == Encoded::from_small(0).encoding()
    }
}

#[test]
fn test_gcd() {
    let small = GenericBigNum::from(5);
    let huge = GenericBigNum::from(i128::MAX).pow(2);
    assert_eq!(huge.gcd(&small), GenericBigNum::from(1));
    assert_eq!(small.gcd(&huge), GenericBigNum::from(1));
}

#[test]
fn test_one() {
    assert!(GenericBigNum::one().is_one());
    assert_eq!(GenericBigNum::one(), GenericBigNum::from(1));
    assert!(!GenericBigNum::from(2).is_one());
}

#[test]
fn test_zero() {
    assert!(GenericBigNum::zero().is_zero());
    assert_eq!(GenericBigNum::zero(), GenericBigNum::from(0));
    assert!(!GenericBigNum::from(1).is_zero());
}
