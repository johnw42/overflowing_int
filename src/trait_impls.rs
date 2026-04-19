use std::{
    cmp::Ordering,
    fmt::{Binary, Formatter, LowerHex, Octal, UpperHex},
    ops::{Neg, Not},
    panic::RefUnwindSafe,
    str::FromStr,
};

use duplicate::duplicate_item;
use num_bigint::{BigInt, BigUint, ParseBigIntError, Sign};
use num_integer::{Integer, Roots};
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedEuclid, CheckedMul, CheckedSub, Euclid, FromBytes,
    FromPrimitive, Num, One, Signed, ToBytes, ToPrimitive, Zero,
};
use paste::paste;
#[cfg(any(test, feature = "serde"))]
use serde::{Deserialize, Serialize};
#[cfg(any(test, feature = "rand"))]
use {
    num_bigint::{RandomBits, UniformBigInt, UniformBigUint},
    rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformSampler},
    rand::prelude::Distribution,
};

use crate::{
    duplicate_iprims, duplicate_prims, duplicate_uprims,
    encoding::{Decode, Decoded, Encode, Encoding},
    signed::Int,
    small_num::SmallNumber,
    unsigned::Uint,
};

#[allow(unused_imports)]
#[duplicate_item(
    mod_name   ImplType   GenericBigNumType;
    [signed]   [BigInt]   [Int];
    [unsigned] [BigUint]  [Uint];
)]
pub mod mod_name {
    use std::{
        fmt::{Debug, Display},
        marker::PhantomData,
    };

    use num_traits::ConstZero;

    use super::*;

    //
    // Arbitrary (quickcheck)
    //

    #[cfg(any(test, feature = "quickcheck"))]
    impl<E> quickcheck::Arbitrary for GenericBigNumType<'static, E>
    where
        E: Encoding<'static, Big = ImplType> + 'static,
    {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            match bool::arbitrary(g) {
                true => Self::from_small(E::Small::arbitrary(g)),
                false => Self::from_big(E::Big::arbitrary(g)),
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match self.decode() {
                Decoded::Small(small) => Box::new(small.shrink().map(Self::from_small)),
                Decoded::Big(big) => Box::new(big.shrink().map(Self::from_big)),
            }
        }
    }

    //
    // Arbitrary (arbitrary)
    //

    #[cfg(feature = "arbitrary")]
    impl<'enc, E> arbitrary::Arbitrary<'_> for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn arbitrary(g: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
            Ok(match bool::arbitrary(g)? {
                true => Self::from_small(E::Small::arbitrary(g)?),
                false => Self::from_big(E::Big::arbitrary(g)?),
            })
        }
    }

    //
    // Binary
    //

    impl<'enc, E> Binary for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            Binary::fmt(self.big_cow().as_ref(), f)
        }
    }

    //
    // CheckedAdd
    //

    impl<'enc, E> CheckedAdd for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn checked_add(&self, v: &Self) -> Option<Self> {
            <Self as Encoding>::checked_add(self, v)
        }
    }

    //
    // CheckedDiv
    //

    impl<'enc, E> CheckedDiv for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn checked_div(&self, v: &Self) -> Option<Self> {
            <Self as Encoding>::checked_div(self, v)
        }
    }

    //
    // CheckedMul
    //

    impl<'enc, E> CheckedMul for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn checked_mul(&self, v: &Self) -> Option<Self> {
            <Self as Encoding>::checked_mul(self, v)
        }
    }

    //
    // CheckedSub
    //

    impl<'enc, E> CheckedSub for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn checked_sub(&self, v: &Self) -> Option<Self> {
            <Self as Encoding>::checked_sub(self, v)
        }
    }

    //
    // ConstZero
    //

    impl<'enc, E> ConstZero for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType, Owned = E>,
    {
        const ZERO: Self = Self::ZERO;
    }

    //
    // Debug
    //

    impl<'enc, E> Debug for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            Debug::fmt(&self.decode(), f)
        }
    }

    //
    // Deserialize
    //

    #[cfg(any(test, feature = "serde"))]
    impl<'enc, 'de, E> Deserialize<'de> for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            ImplType::deserialize(deserializer).map(Into::into)
        }
    }

    //
    // Display
    //

    impl<'enc, E> Display for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            Display::fmt(self.big_cow().as_ref(), f)
        }
    }

    //
    // Distribution
    //

    #[cfg(any(test, feature = "rand"))]
    impl<'enc, E> Distribution<GenericBigNumType<'enc, E>> for RandomBits
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> GenericBigNumType<'enc, E> {
            <RandomBits as Distribution<ImplType>>::sample(self, rng).into()
        }
    }

    //
    // FromStr
    //

    impl<'enc, E> FromStr for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        type Err = num_bigint::ParseBigIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            ImplType::from_str(s).map(Self::from)
        }
    }

    //
    // Integer
    //

    impl<'enc, E> Integer for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn div_floor(&self, other: &Self) -> Self {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
                && (lhs, rhs) != (E::Small::MIN, E::Small::MINUS_ONE)
            {
                return Self::from_small(Integer::div_floor(&lhs, &rhs));
            }
            let (lhs, rhs) = Self::big_cows(self, other);
            Self::from_big(lhs.div_floor(rhs.as_ref()))
        }

        fn mod_floor(&self, other: &Self) -> Self {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
                && (lhs, rhs) != (E::Small::MIN, E::Small::MINUS_ONE)
            {
                return Self::from_small(Integer::mod_floor(&lhs, &rhs));
            }
            let (lhs, rhs) = Self::big_cows(self, other);
            Self::from_big(lhs.mod_floor(rhs.as_ref()))
        }

        fn gcd(&self, other: &Self) -> Self {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
            {
                return Self::from_small(lhs.gcd(&rhs));
            }
            let (lhs, rhs) = Self::big_cows(self, other);
            Self::from_big(lhs.gcd(rhs.as_ref()))
        }

        fn lcm(&self, other: &Self) -> Self {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
                // See note in gcd_lcm about this check.
                && lhs.checked_mul(&rhs).is_some()
            {
                return Self::from_small(lhs.lcm(&rhs));
            }
            let (lhs, rhs) = Self::big_cows(self, other);
            Self::from_big(lhs.lcm(rhs.as_ref()))
        }

        fn is_multiple_of(&self, other: &Self) -> bool {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
            {
                return lhs.is_multiple_of(&rhs);
            }
            let (lhs, rhs) = Self::big_cows(self, other);
            lhs.is_multiple_of(rhs.as_ref())
        }

        fn is_even(&self) -> bool {
            match self.decode() {
                Decoded::Small(n) => n.is_even(),
                Decoded::Big(n) => n.is_even(),
            }
        }

        fn is_odd(&self) -> bool {
            match self.decode() {
                Decoded::Small(n) => n.is_odd(),
                Decoded::Big(n) => n.is_odd(),
            }
        }

        fn div_rem(&self, other: &Self) -> (Self, Self) {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
                && (lhs, rhs) != (E::Small::MIN, E::Small::MINUS_ONE)
            {
                let (q, r) = lhs.div_rem(&rhs);
                return (Self::from_small(q), Self::from_small(r));
            }
            let (q, r) = E::big_cows(self, other);
            (
                Self::from_big(q.div_rem(r.as_ref()).0),
                Self::from_big(q.div_rem(r.as_ref()).1),
            )
        }

        fn gcd_lcm(&self, other: &Self) -> (Self, Self) {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
                // This check ensures computing the LCM won't overflow the small
                // type, which happens particularly when the arguments are
                // relatively prime. This would cause an incorrect result and a
                // potential panic in debug mode .
                && lhs.checked_mul(&rhs).is_some()
            {
                let (gcd, lcm) = lhs.gcd_lcm(&rhs);
                return (Self::from_small(gcd), Self::from_small(lcm));
            }
            let (lhs, rhs) = Self::big_cows(self, other);
            let (gcd, lcm) = lhs.gcd_lcm(rhs.as_ref());
            (Self::from_big(gcd), Self::from_big(lcm))
        }
    }

    //
    // Num
    //

    impl<'enc, E> Num for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        type FromStrRadixErr = ParseBigIntError;

        fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
            ImplType::from_str_radix(str, radix).map(Into::into)
        }
    }

    //
    // One
    //

    impl<'enc, E> One for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn one() -> Self {
            Self::from_small(E::Small::one())
        }

        fn is_one(&self) -> bool {
            match self.decode() {
                Decoded::Small(n) => n.is_one(),
                Decoded::Big(n) => n.is_one(),
            }
        }
    }

    //
    // PartialOrd
    //

    impl<'enc, E> PartialOrd for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    //
    // Roots
    //

    impl<'enc, E> Roots for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn nth_root(&self, n: u32) -> Self {
            match self.decode() {
                Decoded::Small(a) => Self::from_small(a.nth_root(n)),
                Decoded::Big(a) => Self::from_big(a.nth_root(n)),
            }
        }
    }

    //
    // SampleUniform
    //

    #[cfg(any(test, feature = "rand"))]
    impl<'enc, E> SampleUniform for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType> + 'enc,
    {
        type Sampler = UniformSamplerImpl<'enc, E>;
    }

    paste! {
        #[cfg(any(test, feature = "rand"))]
        pub struct UniformSamplerImpl<'enc, E>([<Uniform ImplType>], PhantomData<&'enc E>)
        where
            E: Encoding<'enc, Big = ImplType>;

        #[cfg(any(test, feature = "rand"))]
        impl<'enc, E> UniformSampler for UniformSamplerImpl<'enc, E>
        where
            E: Encoding<'enc, Big = ImplType>,
        {
            type X = GenericBigNumType<'enc, E>;

            fn new<B1, B2>(low: B1, high: B2) -> Self
            where
                B1: SampleBorrow<Self::X> + Sized,
                B2: SampleBorrow<Self::X> + Sized,
            {
                let (low, high) = Self::X::big_cows(low.borrow(), high.borrow());
                Self([<Uniform ImplType>]::new(
                    low.as_ref(),
                    high.as_ref(),
                ), PhantomData)
            }

            fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
            where
                B1: SampleBorrow<Self::X> + Sized,
                B2: SampleBorrow<Self::X> + Sized,
            {
                let (low, high) = Self::X::big_cows(low.borrow(), high.borrow());
                Self([<Uniform ImplType>]::new_inclusive(
                    low.as_ref(),
                    high.as_ref(),
                ), PhantomData)
            }

            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
                self.0.sample(rng).into()
            }
        }
    }

    //
    // ToBytes
    //

    impl<'enc, E> ToBytes for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        type Bytes = Vec<u8>;

        fn to_be_bytes(&self) -> Self::Bytes {
            self.big_cow().to_be_bytes()
        }

        fn to_le_bytes(&self) -> Self::Bytes {
            self.big_cow().to_le_bytes()
        }
    }

    //
    // ToPrimitive
    //

    impl<'enc, E> ToPrimitive for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        duplicate_prims! {
            paste! {
                fn [< to_ prim >](&self) -> Option<prim> {
                    match self.decode() {
                        Decoded::Small(value) => value.[< to_ prim >](),
                        Decoded::Big(value) => value.[< to_ prim >](),
                    }
                }
            }
        }
    }

    //
    // LowerHex
    //

    impl<'enc, E> LowerHex for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            LowerHex::fmt(self.big_cow().as_ref(), f)
        }
    }

    //
    // Octal
    //

    impl<'enc, E> Octal for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            Octal::fmt(self.big_cow().as_ref(), f)
        }
    }

    //
    // RefUnwindSafe
    //

    impl<'enc, E> RefUnwindSafe for GenericBigNumType<'enc, E> where E: Encoding<'enc, Big = ImplType> {}

    //
    // Serialize
    //

    #[cfg(any(test, feature = "serde"))]
    impl<'enc, E> Serialize for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.big_cow().serialize(serializer)
        }
    }

    //
    // UpperHex
    //

    impl<'enc, E> UpperHex for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            UpperHex::fmt(self.big_cow().as_ref(), f)
        }
    }

    //
    // Zero
    //

    impl<'enc, E> Zero for GenericBigNumType<'enc, E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn zero() -> Self {
            Self::from_small(E::Small::zero())
        }

        fn is_zero(&self) -> bool {
            match self.decode() {
                Decoded::Small(n) => n.is_zero(),
                Decoded::Big(n) => n.is_zero(),
            }
        }
    }
}

//
// CheckedEuclid
//

impl<'enc, E> CheckedEuclid for Int<'enc, E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    fn checked_rem_euclid(&self, v: &Self) -> Option<Self> {
        let (lhs, rhs) = Self::big_cows(self, v);
        lhs.checked_rem_euclid(rhs.as_ref()).map(Self::from_big)
    }

    fn checked_div_euclid(&self, v: &Self) -> Option<Self> {
        let (lhs, rhs) = Self::big_cows(self, v);
        lhs.checked_div_euclid(rhs.as_ref()).map(Self::from_big)
    }
}

//
// Euclid
//

impl<'enc, E> Euclid for Int<'enc, E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    fn rem_euclid(&self, v: &Self) -> Self {
        let (lhs, rhs) = Self::big_cows(self, v);
        Self::from_big(lhs.rem_euclid(rhs.as_ref()))
    }

    fn div_euclid(&self, v: &Self) -> Self {
        let (lhs, rhs) = Self::big_cows(self, v);
        Self::from_big(lhs.div_euclid(rhs.as_ref()))
    }
}

//
// FromBytes
//

impl<'enc, E> FromBytes for Int<'enc, E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    type Bytes = [u8];

    fn from_be_bytes(bytes: &[u8]) -> Self {
        E::Big::from_signed_bytes_be(bytes).into()
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        E::Big::from_signed_bytes_le(bytes).into()
    }
}

impl<'enc, E> FromBytes for Uint<'enc, E>
where
    E: Encoding<'enc, Big = BigUint>,
{
    type Bytes = [u8];

    fn from_be_bytes(bytes: &[u8]) -> Self {
        E::Big::from_bytes_be(bytes).into()
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        E::Big::from_bytes_le(bytes).into()
    }
}

//
// FromPrimitive
//

impl<'enc, E> FromPrimitive for Int<'enc, E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    duplicate_prims! { paste! {
        fn [<from_ prim>](n: prim) -> Option<Self> {
            Some(Int::from(n))
        }
    } }
}

//
// FromPrimitive
//

impl<'enc, E> FromPrimitive for Uint<'enc, E>
where
    E: Encoding<'enc, Big = BigUint>,
{
    duplicate_iprims! { paste! {
        fn [<from_ prim>](n: prim) -> Option<Self> {
            match uprim::try_from(n) {
                Ok(small) => Some(Uint::from(small)),
                Err(_) => None,
            }
        }
    } }
    duplicate_uprims! { paste! {
        fn [<from_ prim>](n: prim) -> Option<Self> {
            Some(Uint::from(n))
        }
    } }
}

//
// Neg
//

impl<'enc, E> Neg for Int<'enc, E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    type Output = Int<'enc, E>;

    fn neg(self) -> Self::Output {
        match self.into_decoded() {
            Decoded::Small(s) => {
                if let Some(neg) = s.try_neg() {
                    Self::Output::from_small(neg)
                } else {
                    s.to_big().neg().into()
                }
            }
            Decoded::Big(b) => b.into_owned().neg().into(),
        }
    }
}

impl<'enc, E> Neg for &Int<'enc, E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    type Output = Int<'enc, E>;

    fn neg(self) -> Self::Output {
        match self.decode() {
            Decoded::Small(s) => {
                if let Some(neg) = s.try_neg() {
                    Self::Output::from_small(neg)
                } else {
                    s.to_big().neg().into()
                }
            }
            Decoded::Big(b) => b.into_owned().neg().into(),
        }
    }
}

//
// Not
//

impl<'enc, E> Not for Int<'enc, E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    type Output = Int<'enc, E>;

    fn not(self) -> Self::Output {
        match self.into_decoded() {
            Decoded::Small(n) => Self::from_small(n.not()),
            Decoded::Big(n) => Self::from_big(n.into_owned().not()),
        }
    }
}

impl<'enc, E> Not for &Int<'enc, E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    type Output = Int<'enc, E>;

    fn not(self) -> Self::Output {
        match self.decode() {
            Decoded::Small(n) => Int::from_small(n.not()),
            Decoded::Big(n) => Int::from_big(n.into_owned().not()),
        }
    }
}

//
// Ord
//

impl<'enc, E> Ord for Int<'enc, E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        use Decoded::*;
        use Ordering::*;
        use Sign::*;

        match (self.decode(), other.decode()) {
            (Small(a), Small(b)) => a.cmp(&b),
            (Small(a), Big(b)) => match (a.cmp(&E::Small::zero()), b.sign()) {
                (_, Minus) => Greater,
                (_, Plus) => Less,
                (Equal, NoSign) => Equal,
                (Less, NoSign) => Less,
                (Greater, NoSign) => Greater,
            },
            (Big(a), Small(b)) => match (a.sign(), b.cmp(&E::Small::zero())) {
                (Plus, _) => Greater,
                (Minus, _) => Less,
                (NoSign, Less) => Greater,
                (NoSign, Equal) => Equal,
                (NoSign, Greater) => Less,
            },
            (Big(a), Big(b)) => a.as_ref().cmp(b.as_ref()),
        }
    }
}

impl<'enc, E> Ord for Uint<'enc, E>
where
    E: Encoding<'enc, Big = BigUint>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        use Decoded::*;
        use Ordering::*;

        match (self.decode(), other.decode()) {
            (Small(a), Small(b)) => a.cmp(&b),
            (Small(_), Big(_)) => Less,
            (Big(_), Small(_)) => Greater,
            (Big(a), Big(b)) => a.as_ref().cmp(b.as_ref()),
        }
    }
}

//
// Signed
//

impl<'enc, E> Signed for Int<'enc, E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    fn abs(&self) -> Self {
        match self.decode() {
            Decoded::Small(s) => {
                if let Some(s) = s.try_abs() {
                    Self::from_small(s)
                } else {
                    s.to_big().abs().into()
                }
            }
            Decoded::Big(b) => b.abs().into(),
        }
    }

    fn abs_sub(&self, other: &Self) -> Self {
        (self - other).abs()
    }

    fn signum(&self) -> Self {
        match self.decode() {
            Decoded::Small(n) => Self::from_small(n.signum()),
            Decoded::Big(n) => Self::from_big(n.signum()),
        }
    }

    fn is_positive(&self) -> bool {
        match self.decode() {
            Decoded::Small(n) => n.signum() > E::Small::zero(),
            Decoded::Big(n) => n.is_positive(),
        }
    }

    fn is_negative(&self) -> bool {
        match self.decode() {
            Decoded::Small(n) => n.signum() < E::Small::zero(),
            Decoded::Big(n) => n.is_negative(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::CowBigInt;
    use num_bigint::BigInt;
    use num_integer::Integer;
    use num_traits::{One, Zero};
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

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

    #[quickcheck]
    fn test_round_trip1(a: CowBigInt<'static>) -> TestResult {
        let b = CowBigInt::from(BigInt::from(a.clone()));
        if a != b {
            return TestResult::error(format!("a != b: a = {:?}, b = {:?}", a, b));
        }
        if b != a {
            return TestResult::error(format!("b != a: b = {:?}, a = {:?}", b, a));
        }
        TestResult::passed()
    }

    #[quickcheck]
    fn test_round_trip2(a: BigInt) -> TestResult {
        let b = BigInt::from(CowBigInt::from(a.clone()));
        if a != b {
            return TestResult::error(format!("a != b: a = {:?}, b = {:?}", a, b));
        }
        if b != a {
            return TestResult::error(format!("b != a: b = {:?}, a = {:?}", b, a));
        }
        TestResult::passed()
    }

    #[quickcheck]
    fn test_to_string(a: CowBigInt<'static>) -> bool {
        a.to_string() == BigInt::from(a).to_string()
    }

    #[quickcheck]
    fn test_ord(a: CowBigInt<'static>, b: CowBigInt<'static>) -> bool {
        a.cmp(&b) == BigInt::from(a).cmp(&BigInt::from(b))
    }
}
