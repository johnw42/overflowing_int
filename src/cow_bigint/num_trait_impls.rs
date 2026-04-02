use core::cmp::Ordering;
use core::ops::{Neg, Not};
use std::borrow::Cow;
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

use super::encoding::{Encoded, Encoding, IntoEncoding, IntoEncodingRef as _};
use crate::SmallInt;
use crate::cow_bigint::CowBigInt;

impl quickcheck::Arbitrary for CowBigInt<'static> {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        match bool::arbitrary(g) {
            true => CowBigInt::from(i128::arbitrary(g)),
            false => CowBigInt::from(BigInt::arbitrary(g)),
        }
    }
}

impl<'a, 'b> arbitrary::Arbitrary<'a> for CowBigInt<'b> {
    fn arbitrary(g: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        match bool::arbitrary(g)? {
            true => Ok(CowBigInt::from(i128::arbitrary(g)?)),
            false => Ok(CowBigInt::from(BigInt::arbitrary(g)?)),
        }
    }
}

impl Binary for CowBigInt<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.into_encoding_ref() {
            Encoding::Small(n) => Binary::fmt(n, f),
            Encoding::Big(n) => Binary::fmt(n.borrow(), f),
        }
    }
}

impl CheckedAdd for CowBigInt<'_> {
    fn checked_add(&self, v: &Self) -> Option<Self> {
        Some(self.clone() + v.clone())
    }
}

impl CheckedDiv for CowBigInt<'_> {
    fn checked_div(&self, v: &Self) -> Option<Self> {
        Some(self.clone() / v.clone())
    }
}

impl CheckedEuclid for CowBigInt<'_> {
    fn checked_rem_euclid(&self, v: &Self) -> Option<Self> {
        Cow::from(self)
            .checked_rem_euclid(&Cow::from(v))
            .map(Into::into)
    }

    fn checked_div_euclid(&self, v: &Self) -> Option<Self> {
        Cow::from(self)
            .checked_div_euclid(&Cow::from(v))
            .map(Into::into)
    }
}

impl CheckedMul for CowBigInt<'_> {
    fn checked_mul(&self, v: &Self) -> Option<Self> {
        Some(self.clone() * v.clone())
    }
}

impl CheckedSub for CowBigInt<'_> {
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        Some(self.clone() - v.clone())
    }
}

impl ConstZero for CowBigInt<'static> {
    const ZERO: Self = Self(Encoded::from_small(0));
}

impl<'a, 'de> Deserialize<'de> for CowBigInt<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        BigInt::deserialize(deserializer).map(CowBigInt::from)
    }
}

impl<'a> Distribution<CowBigInt<'a>> for RandomBits {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> CowBigInt<'a> {
        <RandomBits as Distribution<BigInt>>::sample(self, rng).into()
    }
}

impl Euclid for CowBigInt<'_> {
    fn rem_euclid(&self, v: &Self) -> Self {
        Cow::from(self).rem_euclid(&Cow::from(v)).into()
    }

    fn div_euclid(&self, v: &Self) -> Self {
        Cow::from(self).div_euclid(&Cow::from(v)).into()
    }
}

impl FromBytes for CowBigInt<'_> {
    type Bytes = [u8];

    fn from_be_bytes(bytes: &[u8]) -> Self {
        Self::from_signed_bytes_be(bytes)
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        Self::from_signed_bytes_le(bytes)
    }
}

impl FromPrimitive for CowBigInt<'_> {
    fn from_i64(n: i64) -> Option<Self> {
        Some(CowBigInt::from(n))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(CowBigInt::from(n))
    }
}

impl FromStr for CowBigInt<'_> {
    type Err = num_bigint::ParseBigIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BigInt::from_str(s).map(Self::from)
    }
}

impl Integer for CowBigInt<'_> {
    fn div_floor(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other)
            && (lhs, rhs) != (SmallInt::MIN, -1)
        {
            return Integer::div_floor(&lhs, &rhs).into();
        }
        Cow::from(self).div_floor(&*Cow::from(other)).into()
    }

    fn mod_floor(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other)
            && (lhs, rhs) != (SmallInt::MIN, -1)
        {
            return Integer::mod_floor(&lhs, &rhs).into();
        }
        Cow::from(self).mod_floor(&*Cow::from(other)).into()
    }

    fn gcd(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.gcd(&rhs).into();
        }
        Cow::from(self).gcd(&*Cow::from(other)).into()
    }

    fn lcm(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.lcm(&rhs).into();
        }
        Cow::from(self).lcm(&*Cow::from(other)).into()
    }

    fn divides(&self, other: &Self) -> bool {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.is_multiple_of(&rhs);
        }
        Cow::from(self).is_multiple_of(&*Cow::from(other))
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        if let Some((lhs, rhs)) = self.to_small_with(other) {
            return lhs.is_multiple_of(&rhs);
        }
        Cow::from(self).is_multiple_of(&*Cow::from(other))
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
        let (q, r) = Cow::<BigInt>::from(self).div_rem(&*Cow::from(other));
        (q.into(), r.into())
    }
}

impl LowerHex for CowBigInt<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.into_encoding_ref() {
            Encoding::Small(n) => LowerHex::fmt(n, f),
            Encoding::Big(n) => LowerHex::fmt(n.borrow(), f),
        }
    }
}

impl<'a> Neg for CowBigInt<'a> {
    type Output = CowBigInt<'a>;

    fn neg(self) -> Self::Output {
        if let Some(a) = self.to_small()
            && let (b, false) = a.overflowing_neg()
        {
            return b.into();
        }
        BigInt::from(self).neg().into()
    }
}

impl<'a> Neg for &CowBigInt<'a> {
    type Output = CowBigInt<'a>;

    fn neg(self) -> Self::Output {
        if let Some(a) = self.to_small()
            && let (b, false) = a.overflowing_neg()
        {
            return b.into();
        }
        (&*Cow::from(self)).neg().into()
    }
}

impl Num for CowBigInt<'_> {
    type FromStrRadixErr = ParseBigIntError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        BigInt::from_str_radix(str, radix).map(CowBigInt::from)
    }
}

impl<'a> Not for CowBigInt<'a> {
    type Output = CowBigInt<'a>;

    fn not(self) -> Self::Output {
        match self.into_encoding() {
            Encoding::Small(n) => Encoded::from_small(n.not()),
            Encoding::Big(n) => Encoded::from_big(n.into_owned().not()),
        }
        .into()
    }
}

impl<'a> Not for &CowBigInt<'a> {
    type Output = CowBigInt<'a>;

    fn not(self) -> Self::Output {
        match self.into_encoding() {
            Encoding::Small(n) => Encoded::from_small(n.not()),
            Encoding::Big(n) => Encoded::from_big(n.borrow().not()),
        }
        .into()
    }
}

impl Octal for CowBigInt<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.into_encoding_ref() {
            Encoding::Small(n) => Octal::fmt(n, f),
            Encoding::Big(n) => Octal::fmt(n.borrow(), f),
        }
    }
}

impl Ord for CowBigInt<'_> {
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
fn test_round_trip1(a: CowBigInt<'static>) -> bool {
    CowBigInt::from(BigInt::from(a.clone())) == a && a.clone() == CowBigInt::from(BigInt::from(a))
}

#[quickcheck]
fn test_round_trip2(a: BigInt) -> bool {
    BigInt::from(CowBigInt::from(a.clone())) == a && a.clone() == BigInt::from(CowBigInt::from(a))
}

#[quickcheck]
fn test_to_string(a: CowBigInt<'static>) -> bool {
    a.to_string() == BigInt::from(a).to_string()
}

#[quickcheck]
fn test_ord(a: CowBigInt<'static>, b: CowBigInt<'static>) -> bool {
    a.cmp(&b) == BigInt::from(a).cmp(&BigInt::from(b))
}

impl<'a> One for CowBigInt<'a> {
    fn one() -> Self {
        CowBigInt(Encoded::from_small(1))
    }

    fn is_one(&self) -> bool {
        self.encoding() == Encoded::from_small(1).encoding()
    }
}

impl<'a> PartialOrd for CowBigInt<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> RefUnwindSafe for CowBigInt<'a> {}

impl<'a> Roots for CowBigInt<'a> {
    fn nth_root(&self, n: u32) -> Self {
        match self.into_encoding() {
            Encoding::Small(a) => a.nth_root(n).into(),
            Encoding::Big(a) => a.nth_root(n).into(),
        }
    }
}

impl SampleUniform for CowBigInt<'static> {
    type Sampler = UniformCBigInt;
}

pub struct UniformCBigInt(UniformBigInt);

impl UniformSampler for UniformCBigInt {
    type X = CowBigInt<'static>;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self(UniformBigInt::new(
            Cow::from(low.borrow()).borrow(),
            Cow::from(high.borrow()).borrow(),
        ))
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self(UniformBigInt::new_inclusive(
            Cow::from(low.borrow()).borrow(),
            Cow::from(high.borrow()).borrow(),
        ))
    }

    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        self.0.sample(rng).into()
    }
}

impl<'a> Serialize for CowBigInt<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Cow::from(self).serialize(serializer)
    }
}

impl<'a> Signed for CowBigInt<'a> {
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

impl<'a> ToBytes for CowBigInt<'a> {
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

impl<'a> ToPrimitive for CowBigInt<'a> {
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

impl<'a> UpperHex for CowBigInt<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.into_encoding_ref() {
            Encoding::Small(n) => UpperHex::fmt(n, f),
            Encoding::Big(n) => UpperHex::fmt(n.borrow(), f),
        }
    }
}

impl<'a> Zero for CowBigInt<'a> {
    fn zero() -> Self {
        CowBigInt(Encoded::from_small(0))
    }

    fn is_zero(&self) -> bool {
        self.encoding() == Encoded::from_small(0).encoding()
    }
}

#[test]
fn test_gcd() {
    let small = CowBigInt::from(5);
    let huge = CowBigInt::from(i128::MAX).pow(2);
    assert_eq!(huge.gcd(&small), CowBigInt::from(1));
    assert_eq!(small.gcd(&huge), CowBigInt::from(1));
}

#[test]
fn test_one() {
    assert!(CowBigInt::one().is_one());
    assert_eq!(CowBigInt::one(), CowBigInt::from(1));
    assert!(!CowBigInt::from(2).is_one());
}

#[test]
fn test_zero() {
    assert!(CowBigInt::zero().is_zero());
    assert_eq!(CowBigInt::zero(), CowBigInt::from(0));
    assert!(!CowBigInt::from(1).is_zero());
}
