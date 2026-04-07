use crate::generic_bigint::GenericBigInt;
use crate::generic_bignum::encoding::{Decode, Decoded, Encoding};
use crate::generic_biguint::GenericBigUint;
use crate::small_num::SmallNumber;
use crate::{duplicate_iprims, duplicate_prims, duplicate_uprims};
use duplicate::duplicate_item;
use num_bigint::{
    BigInt, BigUint, ParseBigIntError, RandomBits, Sign, UniformBigInt, UniformBigUint,
};
use num_integer::{Integer, Roots};
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedEuclid, CheckedMul, CheckedSub, Euclid, FromBytes,
    FromPrimitive, Num, One, Signed, ToBytes, ToPrimitive, Zero,
};
use paste::paste;
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformSampler};
use rand::prelude::Distribution;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{Binary, Formatter, LowerHex, Octal, UpperHex};
use std::ops::{Neg, Not};
use std::panic::RefUnwindSafe;
use std::str::FromStr;

#[duplicate_item(
    mod_name  BigNumType GenericBigNumWrapper;
    [bigint]  [BigInt]   [GenericBigInt];
    [biguint] [BigUint]  [GenericBigUint];
)]
pub mod mod_name {
    use std::{
        fmt::{Debug, Display},
        marker::PhantomData,
    };

    use super::*;

    impl<E: Encoding<'static, Big = BigNumType> + 'static> quickcheck::Arbitrary
        for GenericBigNumWrapper<'static, E>
    {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            match bool::arbitrary(g) {
                true => Self::from_small(E::Small::arbitrary(g)),
                false => Self::from_big(E::Big::arbitrary(g)),
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            self.with_decoded(|decoded| -> Box<dyn Iterator<Item = Self>> {
                match decoded {
                    Decoded::Small(small) => Box::new(small.shrink().map(Self::from_small)),
                    Decoded::Big(big) => Box::new(big.shrink().map(Self::from_big)),
                }
            })
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> arbitrary::Arbitrary<'_>
        for GenericBigNumWrapper<'a, E>
    {
        fn arbitrary(g: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
            Ok(match bool::arbitrary(g)? {
                true => Self::from_small(E::Small::arbitrary(g)?),
                false => Self::from_big(E::Big::arbitrary(g)?),
            })
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> Binary for GenericBigNumWrapper<'a, E> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            self.with_decoded(|encoded| match encoded {
                Decoded::Small(n) => Binary::fmt(&n, f),
                Decoded::Big(n) => Binary::fmt(n.as_ref(), f),
            })
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> CheckedAdd for GenericBigNumWrapper<'a, E> {
        fn checked_add(&self, v: &Self) -> Option<Self> {
            self.0.checked_add(&v.0).map(Into::into)
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> CheckedDiv for GenericBigNumWrapper<'a, E> {
        fn checked_div(&self, v: &Self) -> Option<Self> {
            self.0.checked_div(&v.0).map(Into::into)
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> CheckedMul for GenericBigNumWrapper<'a, E> {
        fn checked_mul(&self, v: &Self) -> Option<Self> {
            self.0.checked_mul(&v.0).map(Into::into)
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> CheckedSub for GenericBigNumWrapper<'a, E> {
        fn checked_sub(&self, v: &Self) -> Option<Self> {
            self.0.checked_sub(&v.0).map(Into::into)
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> Debug for GenericBigNumWrapper<'a, E> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            self.with_decoded(|encoded| Debug::fmt(&encoded, f))
        }
    }

    impl<'a, 'de, E: Encoding<'a, Big = BigNumType>> Deserialize<'de> for GenericBigNumWrapper<'a, E> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            BigNumType::deserialize(deserializer).map(Into::into)
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> Display for GenericBigNumWrapper<'a, E> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            self.with_decoded(|encoded| match encoded {
                Decoded::Small(n) => Display::fmt(&n, f),
                Decoded::Big(n) => Display::fmt(n.as_ref(), f),
            })
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> Distribution<GenericBigNumWrapper<'a, E>>
        for RandomBits
    {
        fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> GenericBigNumWrapper<'a, E> {
            <RandomBits as Distribution<BigNumType>>::sample(self, rng).into()
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> FromStr for GenericBigNumWrapper<'a, E> {
        type Err = num_bigint::ParseBigIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            BigNumType::from_str(s).map(Self::from)
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> Integer for GenericBigNumWrapper<'a, E> {
        fn div_floor(&self, other: &Self) -> Self {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
                && (lhs, rhs) != (E::Small::MIN, E::Small::MINUS_ONE)
            {
                return Self::from_small(Integer::div_floor(&lhs, &rhs));
            }
            self.with_big_cows(other, |lhs, rhs| lhs.div_floor(rhs.as_ref()).into())
        }

        fn mod_floor(&self, other: &Self) -> Self {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
                && (lhs, rhs) != (E::Small::MIN, E::Small::MINUS_ONE)
            {
                return Self::from_small(Integer::mod_floor(&lhs, &rhs));
            }
            self.with_big_cows(other, |lhs, rhs| lhs.mod_floor(rhs.as_ref()).into())
        }

        fn gcd(&self, other: &Self) -> Self {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
            {
                return Self::from_small(lhs.gcd(&rhs));
            }
            self.with_big_cows(other, |lhs, rhs| lhs.gcd(rhs.as_ref()).into())
        }

        fn lcm(&self, other: &Self) -> Self {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
            {
                return Self::from_small(lhs.lcm(&rhs));
            }
            self.with_big_cows(other, |lhs, rhs| lhs.lcm(rhs.as_ref()).into())
        }

        fn divides(&self, other: &Self) -> bool {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
            {
                return lhs.is_multiple_of(&rhs);
            }
            self.with_big_cows(other, |lhs, rhs| lhs.is_multiple_of(rhs.as_ref()))
        }

        fn is_multiple_of(&self, other: &Self) -> bool {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
            {
                return lhs.is_multiple_of(&rhs);
            }
            self.with_big_cows(other, |lhs, rhs| lhs.is_multiple_of(rhs.as_ref()))
        }

        fn is_even(&self) -> bool {
            self.with_decoded(|decoded| match decoded {
                Decoded::Small(n) => n.is_even(),
                Decoded::Big(n) => n.is_even(),
            })
        }

        fn is_odd(&self) -> bool {
            self.with_decoded(|decoded| match decoded {
                Decoded::Small(n) => n.is_odd(),
                Decoded::Big(n) => n.is_odd(),
            })
        }

        fn div_rem(&self, other: &Self) -> (Self, Self) {
            if let Some(lhs) = self.small()
                && let Some(rhs) = other.small()
                && (lhs, rhs) != (E::Small::MIN, E::Small::MINUS_ONE)
            {
                let (q, r) = lhs.div_rem(&rhs);
                return (Self::from_small(q), Self::from_small(r));
            }
            let (q, r) = self.with_big_cows(other, |lhs, rhs| lhs.div_rem(rhs.as_ref()));
            (Self::from_big(q), Self::from_big(r))
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> Num for GenericBigNumWrapper<'a, E> {
        type FromStrRadixErr = ParseBigIntError;

        fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
            BigNumType::from_str_radix(str, radix).map(Into::into)
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> One for GenericBigNumWrapper<'a, E> {
        fn one() -> Self {
            Self::from_small(E::Small::one())
        }

        fn is_one(&self) -> bool {
            self.small() == Some(E::Small::one())
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> PartialOrd for GenericBigNumWrapper<'a, E> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> Roots for GenericBigNumWrapper<'a, E> {
        fn nth_root(&self, n: u32) -> Self {
            self.with_decoded(|encoded| match encoded {
                Decoded::Small(a) => Self::from_small(a.nth_root(n)),
                Decoded::Big(a) => Self::from_big(a.nth_root(n)),
            })
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType> + 'a> SampleUniform for GenericBigNumWrapper<'a, E> {
        type Sampler = UniformSamplerImpl<'a, E>;
    }

    paste! {
        pub struct UniformSamplerImpl<'a, E: Encoding<'a, Big = BigNumType>>([<Uniform BigNumType>], PhantomData<&'a E>);

        impl<'a, E: Encoding<'a, Big = BigNumType>> UniformSampler for UniformSamplerImpl<'a, E> {
            type X = GenericBigNumWrapper<'a, E>;

            fn new<B1, B2>(low: B1, high: B2) -> Self
            where
                B1: SampleBorrow<Self::X> + Sized,
                B2: SampleBorrow<Self::X> + Sized,
            {
                low.borrow().with_big_cows(high.borrow(), |low, high| {
                        Self([<Uniform BigNumType>]::new(
                            low.as_ref(),
                            high.as_ref(),
                        ), PhantomData)
                })
            }

            fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
            where
                B1: SampleBorrow<Self::X> + Sized,
                B2: SampleBorrow<Self::X> + Sized,
            {
                low.borrow().with_big_cows(high.borrow(), |low, high| {
                    Self([<Uniform BigNumType>]::new_inclusive(
                        low.as_ref(),
                        high.as_ref(),
                    ), PhantomData)
                })
            }

            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
                self.0.sample(rng).into()
            }
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> ToBytes for GenericBigNumWrapper<'a, E> {
        type Bytes = Vec<u8>;

        fn to_be_bytes(&self) -> Self::Bytes {
            self.with_decoded(|encoded| match encoded {
                Decoded::Small(n) => n.to_be_bytes().borrow().to_vec(),
                Decoded::Big(n) => n.to_be_bytes(),
            })
        }

        fn to_le_bytes(&self) -> Self::Bytes {
            self.with_decoded(|encoded| match encoded {
                Decoded::Small(n) => n.to_le_bytes().borrow().to_vec(),
                Decoded::Big(n) => n.to_le_bytes(),
            })
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> ToPrimitive for GenericBigNumWrapper<'a, E> {
        duplicate_prims! {
            paste! {
                fn [< to_ prim >](&self) -> Option<prim> {
                    self.with_decoded(|encoded| match encoded {
                        Decoded::Small(value) => value.[< to_ prim >](),
                        Decoded::Big(value) => value.[< to_ prim >](),
                    })
                }
            }
        }
    }

    impl<'a, E: Encoding<'a>> LowerHex for GenericBigNumWrapper<'a, E> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            self.with_decoded(|encoded| match encoded {
                Decoded::Small(n) => LowerHex::fmt(&n, f),
                Decoded::Big(n) => LowerHex::fmt(n.as_ref(), f),
            })
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> Octal for GenericBigNumWrapper<'a, E> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            self.with_decoded(|encoded| match encoded {
                Decoded::Small(n) => Octal::fmt(&n, f),
                Decoded::Big(n) => Octal::fmt(n.as_ref(), f),
            })
        }
    }

    impl<'a, E: Encoding<'a>> RefUnwindSafe for GenericBigNumWrapper<'a, E> {}

    impl<'a, E: Encoding<'a, Big = BigNumType>> Serialize for GenericBigNumWrapper<'a, E> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.with_big_cow(|big| big.serialize(serializer))
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> UpperHex for GenericBigNumWrapper<'a, E> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            self.with_decoded(|encoded| match encoded {
                Decoded::Small(n) => UpperHex::fmt(&n, f),
                Decoded::Big(n) => UpperHex::fmt(n.as_ref(), f),
            })
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> Zero for GenericBigNumWrapper<'a, E> {
        fn zero() -> Self {
            Self::from_small(E::Small::zero())
        }

        fn is_zero(&self) -> bool {
            self.small() == Some(E::Small::zero())
        }
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> CheckedEuclid for GenericBigInt<'a, E> {
    fn checked_rem_euclid(&self, v: &Self) -> Option<Self> {
        self.with_big_cows(v, |lhs, rhs| {
            lhs.checked_rem_euclid(rhs.as_ref()).map(Into::into)
        })
    }

    fn checked_div_euclid(&self, v: &Self) -> Option<Self> {
        self.with_big_cows(v, |lhs, rhs| {
            lhs.checked_div_euclid(rhs.as_ref()).map(Into::into)
        })
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Euclid for GenericBigInt<'a, E> {
    fn rem_euclid(&self, v: &Self) -> Self {
        self.with_big_cows(v, |lhs, rhs| lhs.rem_euclid(rhs.as_ref()).into())
    }

    fn div_euclid(&self, v: &Self) -> Self {
        self.with_big_cows(v, |lhs, rhs| lhs.div_euclid(rhs.as_ref()).into())
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> FromBytes for GenericBigInt<'a, E> {
    type Bytes = [u8];

    fn from_be_bytes(bytes: &[u8]) -> Self {
        E::Big::from_signed_bytes_be(bytes).into()
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        E::Big::from_signed_bytes_le(bytes).into()
    }
}

impl<'a, E: Encoding<'a, Big = BigUint>> FromBytes for GenericBigUint<'a, E> {
    type Bytes = [u8];

    fn from_be_bytes(bytes: &[u8]) -> Self {
        E::Big::from_bytes_be(bytes).into()
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        E::Big::from_bytes_le(bytes).into()
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> FromPrimitive for GenericBigInt<'a, E> {
    duplicate_prims! { paste! {
        fn [<from_ prim>](n: prim) -> Option<Self> {
            Some(GenericBigInt::from(n))
        }
    } }
}

impl<'a, E: Encoding<'a, Big = BigUint>> FromPrimitive for GenericBigUint<'a, E> {
    duplicate_iprims! { paste! {
        fn [<from_ prim>](_: prim) -> Option<Self> {
            None
        }
    } }
    duplicate_uprims! { paste! {
        fn [<from_ prim>](n: prim) -> Option<Self> {
            Some(GenericBigUint::from(n))
        }
    } }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Neg for GenericBigInt<'a, E> {
    type Output = GenericBigInt<'a, E>;

    fn neg(self) -> Self::Output {
        if let Some(a) = self.small()
            && let (b, false) = a.overflowing_neg()
        {
            Self::Output::from_small(b)
        } else {
            self.with_big_cow(|big| big.as_ref().neg()).into()
        }
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Neg for &GenericBigInt<'a, E> {
    type Output = GenericBigInt<'a, E>;

    fn neg(self) -> Self::Output {
        if let Some(a) = self.small()
            && let (b, false) = a.overflowing_neg()
        {
            Self::Output::from_small(b)
        } else {
            self.with_big_cow(|big| big.as_ref().neg()).into()
        }
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Not for GenericBigInt<'a, E> {
    type Output = GenericBigInt<'a, E>;

    fn not(self) -> Self::Output {
        self.with_decoded(|encoded| match encoded {
            Decoded::Small(n) => Self::from_small(n.not()),
            Decoded::Big(n) => Self::from_big(n.as_ref().not()),
        })
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Not for &GenericBigInt<'a, E> {
    type Output = GenericBigInt<'a, E>;

    fn not(self) -> Self::Output {
        self.with_decoded(|encoded| match encoded {
            Decoded::Small(n) => GenericBigInt::from_small(n.not()),
            Decoded::Big(n) => GenericBigInt::from_big(n.as_ref().not()),
        })
    }
}
impl<'a, E: Encoding<'a, Big = BigInt>> Ord for GenericBigInt<'a, E> {
    fn cmp(&self, other: &Self) -> Ordering {
        use Decoded::*;
        use Ordering::*;
        use Sign::*;

        self.with_decoded(|lhs| {
            other.with_decoded(|rhs| match (lhs, rhs) {
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
            })
        })
    }
}

impl<'a, E: Encoding<'a, Big = BigUint>> Ord for GenericBigUint<'a, E> {
    fn cmp(&self, other: &Self) -> Ordering {
        use Decoded::*;
        use Ordering::*;

        self.with_decoded(|lhs| {
            other.with_decoded(|rhs| match (lhs, rhs) {
                (Small(a), Small(b)) => a.cmp(&b),
                (Small(_), Big(_)) => Less,
                (Big(_), Small(_)) => Greater,
                (Big(a), Big(b)) => a.as_ref().cmp(b.as_ref()),
            })
        })
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Signed for GenericBigInt<'a, E> {
    fn abs(&self) -> Self {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(a) => {
                if let (b, false) = a.overflowing_abs() {
                    Self::from_small(b)
                } else {
                    self.with_big_cow(|big| big.as_ref().abs()).into()
                }
            }
            Decoded::Big(a) => a.as_ref().abs().into(),
        })
    }

    fn abs_sub(&self, other: &Self) -> Self {
        (self - other).abs()
    }

    fn signum(&self) -> Self {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(n) => Self::from_small(n.signum()),
            Decoded::Big(n) => Self::from_big(n.signum()),
        })
    }

    fn is_positive(&self) -> bool {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(n) => n.signum() > E::Small::zero(),
            Decoded::Big(n) => n.is_positive(),
        })
    }

    fn is_negative(&self) -> bool {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(n) => n.signum() < E::Small::zero(),
            Decoded::Big(n) => n.is_negative(),
        })
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
