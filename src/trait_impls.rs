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
    impl<E: Encoding<'static, Big = ImplType> + 'static> quickcheck::Arbitrary
        for GenericBigNumType<'static, E>
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
    impl<'a, E: Encoding<'a, Big = ImplType>> arbitrary::Arbitrary<'_> for GenericBigNumType<'a, E> {
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

    impl<'a, E: Encoding<'a, Big = ImplType>> Binary for GenericBigNumType<'a, E> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            Binary::fmt(self.big_cow().as_ref(), f)
        }
    }

    //
    // CheckedAdd
    //

    impl<'a, E: Encoding<'a, Big = ImplType>> CheckedAdd for GenericBigNumType<'a, E> {
        fn checked_add(&self, v: &Self) -> Option<Self> {
            <Self as Encoding>::checked_add(self, v)
        }
    }

    //
    // CheckedDiv
    //

    impl<'a, E: Encoding<'a, Big = ImplType>> CheckedDiv for GenericBigNumType<'a, E> {
        fn checked_div(&self, v: &Self) -> Option<Self> {
            <Self as Encoding>::checked_div(self, v)
        }
    }

    //
    // CheckedMul
    //

    impl<'a, E: Encoding<'a, Big = ImplType>> CheckedMul for GenericBigNumType<'a, E> {
        fn checked_mul(&self, v: &Self) -> Option<Self> {
            <Self as Encoding>::checked_mul(self, v)
        }
    }

    //
    // CheckedSub
    //

    impl<'a, E: Encoding<'a, Big = ImplType>> CheckedSub for GenericBigNumType<'a, E> {
        fn checked_sub(&self, v: &Self) -> Option<Self> {
            <Self as Encoding>::checked_sub(self, v)
        }
    }

    //
    // CheckedSub
    //

    impl<'a, E: Encoding<'a, Big = ImplType>> ConstZero for GenericBigNumType<'a, E> {
        const ZERO: Self = Self::ZERO;
    }

    //
    // Debug
    //

    impl<'a, E: Encoding<'a, Big = ImplType>> Debug for GenericBigNumType<'a, E> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            Debug::fmt(&self.decode(), f)
        }
    }

    //
    // Deserialize
    //

    #[cfg(any(test, feature = "serde"))]
    impl<'a, 'de, E: Encoding<'a, Big = ImplType>> Deserialize<'de> for GenericBigNumType<'a, E> {
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

    impl<'a, E: Encoding<'a, Big = ImplType>> Display for GenericBigNumType<'a, E> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            Display::fmt(self.big_cow().as_ref(), f)
        }
    }

    //
    // Distribution
    //

    #[cfg(any(test, feature = "rand"))]
    impl<'a, E: Encoding<'a, Big = ImplType>> Distribution<GenericBigNumType<'a, E>> for RandomBits {
        fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> GenericBigNumType<'a, E> {
            <RandomBits as Distribution<ImplType>>::sample(self, rng).into()
        }
    }

    //
    // FromStr
    //

    impl<'a, E: Encoding<'a, Big = ImplType>> FromStr for GenericBigNumType<'a, E> {
        type Err = num_bigint::ParseBigIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            ImplType::from_str(s).map(Self::from)
        }
    }

    //
    // Integer
    //

    impl<'a, E: Encoding<'a, Big = ImplType>> Integer for GenericBigNumType<'a, E> {
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
            // We don't bother doing the LCM computation in the small case, since it can easily overflow.
            let (lhs, rhs) = Self::big_cows(self, other);
            Self::from_big(lhs.lcm(rhs.as_ref()))
        }

        fn divides(&self, other: &Self) -> bool {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
            {
                return lhs.is_multiple_of(&rhs);
            }
            let (lhs, rhs) = Self::big_cows(self, other);
            lhs.is_multiple_of(rhs.as_ref())
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
    }

    //
    // Num
    //

    impl<'a, E: Encoding<'a, Big = ImplType>> Num for GenericBigNumType<'a, E> {
        type FromStrRadixErr = ParseBigIntError;

        fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
            ImplType::from_str_radix(str, radix).map(Into::into)
        }
    }

    //
    // One
    //

    impl<'a, E: Encoding<'a, Big = ImplType>> One for GenericBigNumType<'a, E> {
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

    impl<'a, E: Encoding<'a, Big = ImplType>> PartialOrd for GenericBigNumType<'a, E> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    //
    // Roots
    //

    impl<'a, E: Encoding<'a, Big = ImplType>> Roots for GenericBigNumType<'a, E> {
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
    impl<'a, E: Encoding<'a, Big = ImplType> + 'a> SampleUniform for GenericBigNumType<'a, E> {
        type Sampler = UniformSamplerImpl<'a, E>;
    }

    paste! {
        #[cfg(any(test, feature = "rand"))]
        pub struct UniformSamplerImpl<'a, E: Encoding<'a, Big = ImplType>>([<Uniform ImplType>], PhantomData<&'a E>);

        #[cfg(any(test, feature = "rand"))]
        impl<'a, E: Encoding<'a, Big = ImplType>> UniformSampler for UniformSamplerImpl<'a, E> {
            type X = GenericBigNumType<'a, E>;

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

    impl<'a, E: Encoding<'a, Big = ImplType>> ToBytes for GenericBigNumType<'a, E> {
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

    impl<'a, E: Encoding<'a, Big = ImplType>> ToPrimitive for GenericBigNumType<'a, E> {
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

    impl<'a, E: Encoding<'a, Big = ImplType>> LowerHex for GenericBigNumType<'a, E> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            LowerHex::fmt(self.big_cow().as_ref(), f)
        }
    }

    //
    // Octal
    //

    impl<'a, E: Encoding<'a, Big = ImplType>> Octal for GenericBigNumType<'a, E> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            Octal::fmt(self.big_cow().as_ref(), f)
        }
    }

    //
    // RefUnwindSafe
    //

    impl<'a, E: Encoding<'a, Big = ImplType>> RefUnwindSafe for GenericBigNumType<'a, E> {}

    //
    // Serialize
    //

    #[cfg(any(test, feature = "serde"))]
    impl<'a, E: Encoding<'a, Big = ImplType>> Serialize for GenericBigNumType<'a, E> {
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

    impl<'a, E: Encoding<'a, Big = ImplType>> UpperHex for GenericBigNumType<'a, E> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            UpperHex::fmt(self.big_cow().as_ref(), f)
        }
    }

    //
    // Zero
    //

    impl<'a, E: Encoding<'a, Big = ImplType>> Zero for GenericBigNumType<'a, E> {
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

impl<'a, E: Encoding<'a, Big = BigInt>> CheckedEuclid for Int<'a, E> {
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

impl<'a, E: Encoding<'a, Big = BigInt>> Euclid for Int<'a, E> {
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

impl<'a, E: Encoding<'a, Big = BigInt>> FromBytes for Int<'a, E> {
    type Bytes = [u8];

    fn from_be_bytes(bytes: &[u8]) -> Self {
        E::Big::from_signed_bytes_be(bytes).into()
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        E::Big::from_signed_bytes_le(bytes).into()
    }
}

impl<'a, E: Encoding<'a, Big = BigUint>> FromBytes for Uint<'a, E> {
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

impl<'a, E: Encoding<'a, Big = BigInt>> FromPrimitive for Int<'a, E> {
    duplicate_prims! { paste! {
        fn [<from_ prim>](n: prim) -> Option<Self> {
            Some(Int::from(n))
        }
    } }
}

//
// FromPrimitive
//

impl<'a, E: Encoding<'a, Big = BigUint>> FromPrimitive for Uint<'a, E> {
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

impl<'a, E: Encoding<'a, Big = BigInt>> Neg for Int<'a, E> {
    type Output = Int<'a, E>;

    fn neg(self) -> Self::Output {
        match self.into_decoded() {
            Decoded::Small(s) => {
                if let (neg, false) = s.overflowing_neg() {
                    Self::Output::from_small(neg)
                } else {
                    Self::from_small(s).neg().into()
                }
            }
            Decoded::Big(b) => b.into_owned().neg().into(),
        }
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Neg for &Int<'a, E> {
    type Output = Int<'a, E>;

    fn neg(self) -> Self::Output {
        match self.decode() {
            Decoded::Small(s) => {
                if let (neg, false) = s.overflowing_neg() {
                    Self::Output::from_small(neg)
                } else {
                    Int::from_small(s).neg().into()
                }
            }
            Decoded::Big(b) => b.into_owned().neg().into(),
        }
    }
}

//
// Not
//

impl<'a, E: Encoding<'a, Big = BigInt>> Not for Int<'a, E> {
    type Output = Int<'a, E>;

    fn not(self) -> Self::Output {
        match self.into_decoded() {
            Decoded::Small(n) => Self::from_small(n.not()),
            Decoded::Big(n) => Self::from_big(n.into_owned().not()),
        }
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Not for &Int<'a, E> {
    type Output = Int<'a, E>;

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

impl<'a, E: Encoding<'a, Big = BigInt>> Ord for Int<'a, E> {
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

impl<'a, E: Encoding<'a, Big = BigUint>> Ord for Uint<'a, E> {
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

impl<'a, E: Encoding<'a, Big = BigInt>> Signed for Int<'a, E> {
    fn abs(&self) -> Self {
        match self.decode() {
            Decoded::Small(a) => {
                if let (b, false) = a.overflowing_abs() {
                    Self::from_small(b)
                } else {
                    self.big_cow().abs().into()
                }
            }
            Decoded::Big(a) => a.abs().into(),
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
