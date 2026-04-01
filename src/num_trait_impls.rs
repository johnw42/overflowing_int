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

use crate::SmallInt;
use crate::cbigint::CBigInt;
use crate::encoding::{Encoded, Encoding, IntoEncoding, IntoEncodingRef as _};

impl quickcheck::Arbitrary for CBigInt<'static> {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        match bool::arbitrary(g) {
            true => CBigInt::from(i128::arbitrary(g)),
            false => CBigInt::from(BigInt::arbitrary(g)),
        }
    }
}

impl arbitrary::Arbitrary<'_> for CBigInt<'static> {
    fn arbitrary(g: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        match bool::arbitrary(g)? {
            true => Ok(CBigInt::from(i128::arbitrary(g)?)),
            false => Ok(CBigInt::from(BigInt::arbitrary(g)?)),
        }
    }
}

impl Binary for CBigInt<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.into_encoding_ref() {
            Encoding::Small(n) => Binary::fmt(n, f),
            Encoding::Big(n) => Binary::fmt(n.borrow(), f),
        }
    }
}

impl CheckedAdd for CBigInt<'_> {
    fn checked_add(&self, v: &Self) -> Option<Self> {
        Some(self.clone() + v.clone())
    }
}

impl CheckedDiv for CBigInt<'_> {
    fn checked_div(&self, v: &Self) -> Option<Self> {
        Some(self.clone() / v.clone())
    }
}

impl CheckedEuclid for CBigInt<'_> {
    fn checked_rem_euclid(&self, v: &Self) -> Option<Self> {
        self.to_bigint_cow()
            .checked_rem_euclid(&v.to_bigint_cow())
            .map(Into::into)
    }

    fn checked_div_euclid(&self, v: &Self) -> Option<Self> {
        self.to_bigint_cow()
            .checked_div_euclid(&v.to_bigint_cow())
            .map(Into::into)
    }
}

impl CheckedMul for CBigInt<'_> {
    fn checked_mul(&self, v: &Self) -> Option<Self> {
        Some(self.clone() * v.clone())
    }
}

impl CheckedSub for CBigInt<'_> {
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        Some(self.clone() - v.clone())
    }
}

impl ConstZero for CBigInt<'static> {
    const ZERO: Self = CBigInt(Encoded::ZERO);
}

impl<'a, 'de> Deserialize<'de> for CBigInt<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        BigInt::deserialize(deserializer).map(CBigInt::from)
    }
}

impl<'a> Distribution<CBigInt<'a>> for RandomBits {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> CBigInt<'a> {
        <RandomBits as Distribution<BigInt>>::sample(self, rng).into()
    }
}

impl Euclid for CBigInt<'_> {
    fn rem_euclid(&self, v: &Self) -> Self {
        self.to_bigint_cow().rem_euclid(&v.to_bigint_cow()).into()
    }

    fn div_euclid(&self, v: &Self) -> Self {
        self.to_bigint_cow().div_euclid(&v.to_bigint_cow()).into()
    }
}

impl FromBytes for CBigInt<'_> {
    type Bytes = [u8];

    fn from_be_bytes(bytes: &[u8]) -> Self {
        Self::from_signed_bytes_be(bytes)
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        Self::from_signed_bytes_le(bytes)
    }
}

impl FromPrimitive for CBigInt<'_> {
    fn from_i64(n: i64) -> Option<Self> {
        Some(CBigInt::from(n))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(CBigInt::from(n))
    }
}

impl FromStr for CBigInt<'_> {
    type Err = num_bigint::ParseBigIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BigInt::from_str(s).map(Self::from)
    }
}

impl Integer for CBigInt<'_> {
    fn div_floor(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other)
            && (lhs, rhs) != (SmallInt::MIN, -1)
        {
            return Integer::div_floor(&lhs, &rhs).into();
        }
        self.to_bigint_cow()
            .div_floor(&*other.to_bigint_cow())
            .into()
    }

    fn mod_floor(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other)
            && (lhs, rhs) != (SmallInt::MIN, -1)
        {
            return Integer::mod_floor(&lhs, &rhs).into();
        }
        self.to_bigint_cow()
            .mod_floor(&*other.to_bigint_cow())
            .into()
    }

    fn gcd(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.gcd(&rhs).into();
        }
        self.to_bigint_cow().gcd(&*other.to_bigint_cow()).into()
    }

    fn lcm(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.lcm(&rhs).into();
        }
        self.to_bigint_cow().lcm(&*other.to_bigint_cow()).into()
    }

    fn divides(&self, other: &Self) -> bool {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.is_multiple_of(&rhs);
        }
        self.to_bigint_cow().is_multiple_of(&*other.to_bigint_cow())
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.is_multiple_of(&rhs);
        }
        self.to_bigint_cow().is_multiple_of(&*other.to_bigint_cow())
    }

    fn is_even(&self) -> bool {
        match self.into_encoding_ref() {
            Encoding::Small(n) => n.is_even(),
            Encoding::Big(n) => n.is_even(),
        }
    }

    fn is_odd(&self) -> bool {
        match self.into_encoding_ref() {
            Encoding::Small(n) => n.is_odd(),
            Encoding::Big(n) => n.is_odd(),
        }
    }

    fn div_rem(&self, other: &Self) -> (Self, Self) {
        if let Some((lhs, rhs)) = self.to_small_with(other)
            && (lhs, rhs) != (SmallInt::MIN, -1)
        {
            let (q, r) = lhs.div_rem(&rhs);
            return (q.into(), r.into());
        }
        let (q, r) = self.to_bigint_cow().div_rem(&*other.to_bigint_cow());
        (q.into(), r.into())
    }
}

impl LowerHex for CBigInt<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.into_encoding_ref() {
            Encoding::Small(n) => LowerHex::fmt(n, f),
            Encoding::Big(n) => LowerHex::fmt(n.borrow(), f),
        }
    }
}

impl<'a> Neg for CBigInt<'a> {
    type Output = CBigInt<'a>;

    fn neg(self) -> Self::Output {
        if let Some(a) = self.to_small()
            && let (b, false) = a.overflowing_neg()
        {
            return b.into();
        }
        BigInt::from(self).neg().into()
    }
}

impl<'a> Neg for &CBigInt<'a> {
    type Output = CBigInt<'a>;

    fn neg(self) -> Self::Output {
        if let Some(a) = self.to_small()
            && let (b, false) = a.overflowing_neg()
        {
            return b.into();
        }
        (&*self.to_bigint_cow()).neg().into()
    }
}

impl Num for CBigInt<'_> {
    type FromStrRadixErr = ParseBigIntError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        BigInt::from_str_radix(str, radix).map(CBigInt::from)
    }
}

impl<'a> Not for CBigInt<'a> {
    type Output = CBigInt<'a>;

    fn not(self) -> Self::Output {
        CBigInt(match self.into_encoding() {
            Encoding::Small(n) => Encoded::from_small(n.not()),
            Encoding::Big(n) => Encoded::from_big(n.into_owned().not()),
        })
    }
}

impl<'a> Not for &CBigInt<'a> {
    type Output = CBigInt<'a>;

    fn not(self) -> Self::Output {
        CBigInt(match self.into_encoding() {
            Encoding::Small(n) => Encoded::from_small(n.not()),
            Encoding::Big(n) => Encoded::from_big(n.borrow().not()),
        })
    }
}

impl Octal for CBigInt<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.into_encoding_ref() {
            Encoding::Small(n) => Octal::fmt(n, f),
            Encoding::Big(n) => Octal::fmt(n.borrow(), f),
        }
    }
}

impl Ord for CBigInt<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        use Encoding::*;
        use Ordering::*;
        use Sign::*;

        match (&self.into_encoding_ref(), other.into_encoding_ref()) {
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
fn test_round_trip1(a: CBigInt<'static>) -> bool {
    CBigInt::from(BigInt::from(a.clone())) == a && a.clone() == CBigInt::from(BigInt::from(a))
}

#[quickcheck]
fn test_round_trip2(a: BigInt) -> bool {
    BigInt::from(CBigInt::from(a.clone())) == a && a.clone() == BigInt::from(CBigInt::from(a))
}

#[quickcheck]
fn test_to_string(a: CBigInt<'static>) -> bool {
    a.to_string() == BigInt::from(a).to_string()
}

#[quickcheck]
fn test_ord(a: CBigInt<'static>, b: CBigInt<'static>) -> bool {
    a.cmp(&b) == BigInt::from(a).cmp(&BigInt::from(b))
}

impl<'a> One for CBigInt<'a> {
    fn one() -> Self {
        CBigInt(Encoded::ONE)
    }

    fn is_one(&self) -> bool {
        self.0 == Encoded::ONE
    }
}

impl<'a> PartialOrd for CBigInt<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> RefUnwindSafe for CBigInt<'a> {}

impl<'a> Roots for CBigInt<'a> {
    fn nth_root(&self, n: u32) -> Self {
        match self.into_encoding() {
            Encoding::Small(a) => a.nth_root(n).into(),
            Encoding::Big(a) => a.nth_root(n).into(),
        }
    }
}

impl SampleUniform for CBigInt<'static> {
    type Sampler = UniformCBigInt;
}

pub struct UniformCBigInt(UniformBigInt);

impl UniformSampler for UniformCBigInt {
    type X = CBigInt<'static>;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self(UniformBigInt::new(
            low.borrow().to_bigint_cow().borrow(),
            high.borrow().to_bigint_cow().borrow(),
        ))
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self(UniformBigInt::new_inclusive(
            low.borrow().to_bigint_cow().borrow(),
            high.borrow().to_bigint_cow().borrow(),
        ))
    }

    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        self.0.sample(rng).into()
    }
}

impl<'a> Serialize for CBigInt<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_bigint_cow().serialize(serializer)
    }
}

impl<'a> Signed for CBigInt<'a> {
    fn abs(&self) -> Self {
        match self.into_encoding_ref() {
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

impl<'a> ToBytes for CBigInt<'a> {
    type Bytes = Vec<u8>;

    fn to_be_bytes(&self) -> Self::Bytes {
        match self.into_encoding_ref() {
            Encoding::Small(n) => n.to_be_bytes().to_vec(),
            Encoding::Big(n) => n.to_be_bytes(),
        }
    }

    fn to_le_bytes(&self) -> Self::Bytes {
        match self.into_encoding_ref() {
            Encoding::Small(n) => n.to_le_bytes().to_vec(),
            Encoding::Big(n) => n.to_le_bytes(),
        }
    }
}

impl<'a> ToPrimitive for CBigInt<'a> {
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
            match self.into_encoding_ref() {
                Encoding::Small(value) => value.[< to_ prim >](),
                Encoding::Big(value) => value.[< to_ prim >](),
            }
        }
    }
}

impl<'a> UpperHex for CBigInt<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.into_encoding_ref() {
            Encoding::Small(n) => UpperHex::fmt(n, f),
            Encoding::Big(n) => UpperHex::fmt(n.borrow(), f),
        }
    }
}

impl<'a> Zero for CBigInt<'a> {
    fn zero() -> Self {
        CBigInt(Encoded::ZERO)
    }

    fn is_zero(&self) -> bool {
        self.0 == Encoded::ZERO
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
