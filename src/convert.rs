#![allow(unused_imports)]
use crate::encoding::{Decode, Decode as _, Decoded, Encode, Encoding};
use crate::signed::GenericSignedBigNum;
use crate::small_num::SmallNumber;
use crate::unsigned::GenericUnsignedBigNum;
use crate::{
    CowBigInt, CowBigUint, RcBigInt, RcBigUint, duplicate_iprims, duplicate_iprims_if_unsigned,
    duplicate_prims, duplicate_uprims, duplicate_uprims_and_iprims_if_signed,
};
use duplicate::{duplicate, duplicate_item};
use num_bigint::{BigInt, BigUint, Sign, ToBigInt, ToBigUint};
use num_traits::{One, ToPrimitive, Zero};
use paste::paste;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TryFromBigIntError<T>(T);

impl<T> TryFromBigIntError<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_original(self) -> T {
        self.0
    }
}

impl<T: Debug> Display for TryFromBigIntError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to convert big integer to narrower type: {:?}",
            self.0
        )
    }
}

impl<T> Error for TryFromBigIntError<T> where T: Debug {}

pub trait ToCowBigInt<'a> {
    fn to_cow_bigint(&'a self) -> Option<CowBigInt<'a>>;
}

pub trait ToCowBigUint<'a> {
    fn to_cow_biguint(&'a self) -> Option<CowBigUint<'a>>;
}

pub trait ToRcBigInt {
    fn to_rc_bigint(&self) -> Option<RcBigInt>;
}

pub trait ToRcBigUint {
    fn to_rc_biguint(&self) -> Option<RcBigUint>;
}

// =============================================================================
// Idiosyncratic Conversions to Encoded Types
// =============================================================================

#[duplicate_item(
    ImplType   RcType       CowType       signedness;
    [BigInt]   [RcBigInt]   [CowBigInt]   [signed];
    [BigUint]  [RcBigUint]  [CowBigUint]  [unsigned];
)]
pub mod signedness {
    use super::*;

    // from references to the impl type

    impl<'a> From<&'a ImplType> for CowType<'a> {
        fn from(value: &'a ImplType) -> Self {
            Self::from_big_cow(Cow::Borrowed(value))
        }
    }

    impl<'a> From<&'a ImplType> for RcType {
        fn from(value: &'a ImplType) -> Self {
            Self::from_big_cow(Cow::Owned(value.clone()))
        }
    }

    // from cows of the impl type

    impl<'a> From<Cow<'a, ImplType>> for CowType<'a> {
        fn from(value: Cow<'a, ImplType>) -> Self {
            Self::from_big_cow(value)
        }
    }

    impl<'a> From<Cow<'a, ImplType>> for RcType {
        fn from(value: Cow<'a, ImplType>) -> Self {
            Self::from_big(value.into_owned())
        }
    }

    // from cow type to rc type

    impl<'a> From<CowType<'a>> for RcType {
        fn from(value: CowType<'a>) -> Self {
            match value.0.decode() {
                Decoded::Small(s) => Self::from(s),
                Decoded::Big(big) => Self::from(big.into_owned()),
            }
        }
    }

    impl<'a> From<&CowType<'a>> for RcType {
        fn from(value: &CowType<'a>) -> Self {
            value.0.with_decoded(|decoded| match decoded {
                Decoded::Small(s) => Self::from(s),
                Decoded::Big(big) => Self::from(big.into_owned()),
            })
        }
    }

    // from rc type to cow type

    impl<'a> From<RcType> for CowType<'a> {
        fn from(value: RcType) -> Self {
            match value.0.decode() {
                Decoded::Small(s) => Self::from(s),
                Decoded::Big(big) => Self::from_big_cow(big),
            }
        }
    }

    impl<'a> From<&RcType> for CowType<'a> {
        fn from(value: &RcType) -> Self {
            value.0.with_decoded(|decoded| match decoded {
                Decoded::Small(s) => Self::from(s),
                Decoded::Big(big) => Self::from(big.into_owned()),
            })
        }
    }

    // from references to a type to the type itself

    impl<'a, 'b> From<&'b CowType<'a>> for CowType<'a>
    where
        'b: 'a,
    {
        fn from(value: &'b CowType<'a>) -> Self {
            value.borrow()
        }
    }

    impl From<&RcType> for RcType {
        fn from(value: &RcType) -> Self {
            value.clone()
        }
    }

    // To* traits
    paste! {
        impl<'a> [<To ImplType>] for CowType<'a> {
            fn [<to_ ImplType:lower>](&self) -> Option<ImplType> {
                Some(self.clone().into_big())
            }
        }

        impl [<To ImplType>] for RcType {
            fn [<to_ ImplType:lower>](&self) -> Option<ImplType> {
                Some(self.clone().into_big())
            }
        }

        impl<'a> [<To CowType>]<'a> for ImplType {
            fn [<to_cow_ ImplType:lower>](&'a self) -> Option<CowType<'a>> {
                Some(CowType::from_big_cow(Cow::Borrowed(self)))
            }
        }

        impl<'a> [<To CowType>]<'a> for CowType<'a> {
            fn [<to_cow_ ImplType:lower>](&'a self) -> Option<CowType<'a>> {
                Some(self.clone())
            }
        }

        impl<'a> [<To CowType>]<'a> for RcType {
            fn [<to_cow_ ImplType:lower>](&'a self) -> Option<CowType<'a>> {
                self.with_decoded(|decoded| match decoded {
                    Decoded::Small(s) => Some(CowType::from(s)),
                    Decoded::Big(big) => Some(CowType::from_big(big.into_owned())),
                })
            }
        }

        impl [<To RcType>] for ImplType {
            fn [<to_rc_ ImplType:lower>](&self) -> Option<RcType> {
                Some(RcType::from_big(self.clone()))
            }
        }

        impl<'a> [<To RcType>] for CowType<'a> {
            fn [<to_rc_ ImplType:lower>](&self) -> Option<RcType> {
                self.0.with_decoded(|decoded| match decoded {
                    Decoded::Small(s) => Some(RcType::from(s)),
                    Decoded::Big(big) => Some(RcType::from(big.into_owned())),
                })
            }
        }

        impl [<To RcType>] for RcType {
            fn [<to_rc_ ImplType:lower>](&self) -> Option<RcType> {
                Some(self.clone())
            }
        }
    }
}

// =============================================================================
// To* Signed to Unsigned
// =============================================================================

impl<'a> ToBigUint for CowBigInt<'a> {
    fn to_biguint(&self) -> Option<BigUint> {
        self.clone().into_big().try_into().ok()
    }
}

impl ToBigUint for RcBigInt {
    fn to_biguint(&self) -> Option<BigUint> {
        self.clone().into_big().try_into().ok()
    }
}

impl<'a> ToCowBigUint<'a> for BigInt {
    fn to_cow_biguint(&'a self) -> Option<CowBigUint<'a>> {
        self.to_biguint().map(CowBigUint::from_big)
    }
}

impl ToRcBigUint for BigInt {
    fn to_rc_biguint(&self) -> Option<RcBigUint> {
        self.to_biguint().map(RcBigUint::from_big)
    }
}

impl<'a> ToCowBigUint<'a> for CowBigInt<'a> {
    fn to_cow_biguint(&'a self) -> Option<CowBigUint<'a>> {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => CowBigUint::try_from(s).ok(),
            Decoded::Big(big) => big.to_biguint().map(CowBigUint::from_big),
        })
    }
}

impl<'a> ToCowBigUint<'a> for RcBigInt {
    fn to_cow_biguint(&'a self) -> Option<CowBigUint<'a>> {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => CowBigUint::try_from(s).ok(),
            Decoded::Big(big) => big.to_biguint().map(CowBigUint::from_big),
        })
    }
}

impl<'a> ToRcBigUint for CowBigInt<'a> {
    fn to_rc_biguint(&self) -> Option<RcBigUint> {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => RcBigUint::try_from(s).ok(),
            Decoded::Big(big) => big.to_biguint().map(RcBigUint::from_big),
        })
    }
}

impl ToRcBigUint for RcBigInt {
    fn to_rc_biguint(&self) -> Option<RcBigUint> {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => RcBigUint::try_from(s).ok(),
            Decoded::Big(big) => big.to_biguint().map(RcBigUint::from_big),
        })
    }
}

duplicate_iprims! {
    impl<'a> ToCowBigUint<'a> for prim {
        fn to_cow_biguint(&'a self) -> Option<CowBigUint<'a>> {
            CowBigUint::try_from(*self).ok()
        }
    }


    impl ToRcBigUint for prim {
        fn to_rc_biguint(&self) -> Option<RcBigUint> {
            RcBigUint::try_from(*self).ok()
        }
    }
}

// =============================================================================
// To* Unsigned to Signed
// =============================================================================

impl<'a> ToBigInt for CowBigUint<'a> {
    fn to_bigint(&self) -> Option<BigInt> {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => Some(s.to_bigint()),
            Decoded::Big(b) => b.to_bigint(),
        })
    }
}

impl ToBigInt for RcBigUint {
    fn to_bigint(&self) -> Option<BigInt> {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => Some(s.to_bigint()),
            Decoded::Big(b) => b.to_bigint(),
        })
    }
}

impl<'a> ToCowBigInt<'a> for BigUint {
    fn to_cow_bigint(&'a self) -> Option<CowBigInt<'a>> {
        self.to_bigint().map(CowBigInt::from_big)
    }
}

impl ToRcBigInt for BigUint {
    fn to_rc_bigint(&self) -> Option<RcBigInt> {
        self.to_bigint().map(RcBigInt::from_big)
    }
}

impl<'a> ToCowBigInt<'a> for CowBigUint<'a> {
    fn to_cow_bigint(&'a self) -> Option<CowBigInt<'a>> {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => Some(CowBigInt::from(s)),
            Decoded::Big(b) => b.to_bigint().map(CowBigInt::from_big),
        })
    }
}

impl<'a> ToRcBigInt for CowBigUint<'a> {
    fn to_rc_bigint(&self) -> Option<RcBigInt> {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => Some(RcBigInt::from(s)),
            Decoded::Big(b) => b.to_bigint().map(RcBigInt::from_big),
        })
    }
}

impl<'a> ToCowBigInt<'a> for RcBigUint {
    fn to_cow_bigint(&'a self) -> Option<CowBigInt<'a>> {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => Some(CowBigInt::from(s)),
            Decoded::Big(b) => b.to_bigint().map(CowBigInt::from_big),
        })
    }
}

impl ToRcBigInt for RcBigUint {
    fn to_rc_bigint(&self) -> Option<RcBigInt> {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => Some(RcBigInt::from(s)),
            Decoded::Big(b) => b.to_bigint().map(RcBigInt::from_big),
        })
    }
}

duplicate_prims! {
    impl<'a> ToCowBigInt<'a> for prim {
        fn to_cow_bigint(&'a self) -> Option<CowBigInt<'a>> {
            Some(self.into())
        }
    }


    impl ToRcBigInt for prim {
        fn to_rc_bigint(&self) -> Option<RcBigInt> {
            Some(self.into())
        }
    }
}

// =============================================================================
// TryFrom Signed to Unsigned
// =============================================================================

impl<'a, E: Encoding<'a, Big = BigUint>> TryFrom<BigInt> for GenericUnsignedBigNum<'a, E> {
    type Error = TryFromBigIntError<BigInt>;

    fn try_from(value: BigInt) -> Result<Self, Self::Error> {
        value
            .to_biguint()
            .map(Self::from_big)
            .ok_or_else(|| TryFromBigIntError::new(value))
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> TryFrom<GenericSignedBigNum<'a, E>> for BigUint {
    type Error = TryFromBigIntError<GenericSignedBigNum<'a, E>>;

    fn try_from(value: GenericSignedBigNum<'a, E>) -> Result<Self, Self::Error> {
        value
            .to_biguint()
            .ok_or_else(|| TryFromBigIntError::new(value))
    }
}

impl<'a, 'b, E1, E2> TryFrom<GenericSignedBigNum<'a, E1>> for GenericUnsignedBigNum<'b, E2>
where
    E1: Encoding<'a, Big = BigInt>,
    E2: Encoding<'b, Big = BigUint>,
    E2::Small: TryFrom<E1::Small>,
{
    type Error = TryFromBigIntError<GenericSignedBigNum<'a, E1>>;

    fn try_from(value: GenericSignedBigNum<'a, E1>) -> Result<Self, Self::Error> {
        value
            .with_decoded(|decoded| match decoded {
                Decoded::Small(s) => match E2::Small::try_from(s) {
                    Ok(u) => Some(Self::from_small(u)),
                    Err(_) => s.to_biguint().map(Self::from_big),
                },
                Decoded::Big(b) => b.to_biguint().map(Self::from_big),
            })
            .ok_or_else(|| TryFromBigIntError::new(value))
    }
}

duplicate_uprims! {
    impl<'a> ToCowBigUint<'a> for prim {
        fn to_cow_biguint(&'a self) -> Option<CowBigUint<'a>> {
            Some(self.into())
        }
    }


    impl ToRcBigUint for prim {
        fn to_rc_biguint(&self) -> Option<RcBigUint> {
            Some(self.into())
        }
    }
}

// =============================================================================
// From Unsigned to Signed
// =============================================================================

impl<'a, E: Encoding<'a, Big = BigInt>> From<BigUint> for GenericSignedBigNum<'a, E> {
    fn from(value: BigUint) -> Self {
        Self::from_big(value.into())
    }
}

impl<'a, E: Encoding<'a, Big = BigUint>> From<GenericUnsignedBigNum<'a, E>> for BigInt {
    fn from(value: GenericUnsignedBigNum<'a, E>) -> Self {
        BigInt::from(value.clone().into_big())
    }
}

impl<'a, 'b, E1, E2> From<GenericUnsignedBigNum<'a, E1>> for GenericSignedBigNum<'b, E2>
where
    E1: Encoding<'a, Big = BigUint>,
    E2: Encoding<'b, Big = BigInt>,
    E2::Small: TryFrom<E1::Small>,
{
    fn from(value: GenericUnsignedBigNum<'a, E1>) -> Self {
        value.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => match E2::Small::try_from(s) {
                Ok(small) => Self::from_small(small),
                Err(_) => Self::from_big(s.to_bigint()),
            },
            Decoded::Big(b) => Self::from_big(b.into_owned().into()),
        })
    }
}

#[duplicate_item(
    tag       signedness  ImplType    EncodedType;
    [bigint]  [signed]    [BigInt]   [GenericSignedBigNum];
    [biguint] [unsigned]  [BigUint]  [GenericUnsignedBigNum];
)]
pub mod tag {
    use super::*;

    impl<'a, E: Encoding<'a, Big = ImplType>> From<ImplType> for EncodedType<'a, E> {
        fn from(value: ImplType) -> Self {
            Self::from_big(value)
        }
    }

    impl<'a, E: Encoding<'a, Big = ImplType>> From<EncodedType<'a, E>> for ImplType {
        fn from(value: EncodedType<'a, E>) -> Self {
            value.0.into_big()
        }
    }

    impl<'a, E: Encoding<'a, Big = ImplType>> From<&EncodedType<'a, E>> for ImplType {
        fn from(value: &EncodedType<'a, E>) -> Self {
            value.clone().0.into_big()
        }
    }

    impl<'a, E: Encoding<'a, Big = ImplType>> From<bool> for EncodedType<'a, E> {
        fn from(value: bool) -> Self {
            u8::from(value).into()
        }
    }

    duplicate_uprims_and_iprims_if_signed! { signedness;
        paste! {
            impl<'a, E: Encoding<'a, Big=ImplType>> From<prim> for EncodedType<'a, E> {
                fn from(value: prim) -> Self {
                    #[allow(irrefutable_let_patterns)]
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    if let Ok(n) = E::Small::try_from(value) {
                        Self::from_small(n)
                    } else {
                        Self::from_big(E::Big::from(value))
                    }
                }
            }

            impl<'a, E: Encoding<'a, Big=ImplType>> From<&prim> for EncodedType<'a, E> {
                fn from(value: &prim) -> Self {
                    Self::from(*value)
                }
            }
        }
    }

    duplicate_iprims_if_unsigned! { signedness;
        paste! {
            impl<'a, E: Encoding<'a, Big=ImplType>> TryFrom<prim> for EncodedType<'a, E> {
                type Error = TryFromBigIntError<()>;

                fn try_from(value: prim) -> Result<Self, Self::Error> {
                    #[allow(irrefutable_let_patterns)]
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    if let Ok(n) = E::Small::try_from(value) {
                        Ok(Self::from_small(n))
                    } else {
                        E::Big::try_from(value).map(Self::from_big).map_err(|_| TryFromBigIntError::new(()))
                    }
                }
            }

            impl<'a, E: Encoding<'a, Big=ImplType>> TryFrom<&prim> for EncodedType<'a, E> {
                type Error = TryFromBigIntError<()>;

                fn try_from(value: &prim) -> Result<Self, Self::Error> {
                    Self::try_from(*value)
                }
            }
        }
    }

    duplicate_prims! {
    paste! {
        impl<'a, E: Encoding<'a, Big=ImplType>> TryFrom<EncodedType<'a, E>> for prim {
                type Error = TryFromBigIntError<EncodedType<'a, E>>;

                fn try_from(value: EncodedType<'a, E>) -> Result<Self, Self::Error> {
                    value.0.with_decoded(|decoded| match decoded {
                        Decoded::Small(n) => n.[<to_ prim>](),
                        Decoded::Big(n) => n.[<to_ prim>](),
                    }).ok_or_else(|| TryFromBigIntError::new(value))
                }
            }

            impl<'a, 'b, E: Encoding<'a, Big=ImplType>> TryFrom<&'b EncodedType<'a, E>> for prim {
                type Error = TryFromBigIntError<()>;

                fn try_from(value: &'b EncodedType<'a, E>) -> Result<Self, Self::Error> {
                    value.0.with_decoded(|decoded| match decoded {
                        Decoded::Small(n) => n.[<to_ prim>](),
                        Decoded::Big(n) => n.[<to_ prim>](),
                    }).ok_or(TryFromBigIntError::new(()))
                }
            }
        }
    }
}

// =============================================================================
// Unit Tests for Conversions
// =============================================================================

#[cfg(test)]
mod test {
    use if_tokens::if_tokens;
    use quickcheck_macros::quickcheck;

    use super::*;

    type StaticCowBigInt = CowBigInt<'static>;
    type StaticCowBigUint = CowBigUint<'static>;

    // Test idiosyncratic conversions.
    duplicate! {
        [
            ImplType   RcType       CowType       signedness;
            [BigInt]   [RcBigInt]   [CowBigInt]   [signed];
            [BigUint]  [RcBigUint]  [CowBigUint]  [unsigned];
        ]

        paste ! {
            #[quickcheck]
            fn [<test_from_ref_ ImplType:lower>](value: ImplType) {
                let rc = RcType::from(&value);
                let cow = CowType::from(&value);
                assert_eq!(rc.clone().into_big(), value);
                assert_eq!(cow.clone().into_big(), value);
            }

            #[quickcheck]
            fn [<test_from_cow_ ImplType:lower>](value: ImplType) {
                let rc = RcType::from(Cow::Owned(value.clone()));
                let cow = CowType::from(Cow::Owned(value.clone()));
                assert_eq!(rc.clone().into_big(), value);
                assert_eq!(cow.clone().into_big(), value);
            }

            #[quickcheck]
            fn [<test_from_ref_ CowType:lower>](value: CowType<'static>) {
                let rc = RcType::from(&value);
                let cow = CowType::from(&value);
                assert_eq!(rc.clone().into_big(), value.clone().into_big());
                assert_eq!(cow.clone().into_big(), value.clone().into_big());
            }

            #[quickcheck]
            fn [<test_from_ref_ RcType:lower>](value: RcType) {
                let rc = RcType::from(&value);
                let cow = CowType::from(&value);
                assert_eq!(rc.clone().into_big(), value.clone().into_big());
                assert_eq!(cow.clone().into_big(), value.clone().into_big());
            }
        }
    }

    // Test infallible signedess conversions.
    duplicate! {
        [
            SourceType;
            [BigInt];
            [BigUint];
            [StaticCowBigInt];
            [StaticCowBigUint];
            [RcBigInt];
            [RcBigUint];
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
        ]
        duplicate! {
            [
                TargetType;
                [BigInt];
                [StaticCowBigInt];
                [RcBigInt];
            ]
            if_tokens! {
                if [SourceType] != [TargetType] {
                    paste! {
                        #[quickcheck]
                        fn [<test_to_signed_ SourceType:lower _to_ TargetType:lower>](value: SourceType) {
                            #[allow(clippy::clone_on_copy)]
                            let converted: TargetType = TargetType::from(value.clone());
                            #[allow(clippy::unnecessary_fallible_conversions)]
                            let round_trip: Option<SourceType> = SourceType::try_from(converted).ok();
                            assert_eq!(Some(value), round_trip);
                        }
                    }
                }
            }
        }
    }

    // Test infallible signed non-narrowing conversions.
    duplicate! {
        [
            SourceType;
            [BigInt];
            [StaticCowBigInt];
            [RcBigInt];
            [i8];
            [i16];
            [i32];
            [i64];
            [i128];
            [isize];
        ]
        duplicate! {
            [
                TargetType;
                [BigInt];
                [StaticCowBigInt];
                [RcBigInt];
            ]
            if_tokens! {
                if [SourceType] != [TargetType] {
                    paste! {
                        #[quickcheck]
                        fn [<test_widening_ SourceType:lower _to_ TargetType:lower>](value: SourceType) {
                            #[allow(clippy::clone_on_copy)]
                            let converted: TargetType = TargetType::from(value.clone());
                            #[allow(clippy::unnecessary_fallible_conversions)]
                            let round_trip: Option<SourceType> = SourceType::try_from(converted).ok();
                            assert_eq!(Some(value), round_trip);
                        }
                    }
                }
            }
        }
    }

    // Test infallible unsigned non-narrowing conversions.
    duplicate! {
        [
            SourceType;
            [BigUint];
            [StaticCowBigUint];
            [RcBigUint];
            [u8];
            [u16];
            [u32];
            [u64];
            [u128];
            [usize];
        ]
        duplicate! {
            [
                TargetType;
                [BigUint];
                [StaticCowBigUint];
                [RcBigUint];
            ]
            if_tokens! {
                if [SourceType] != [TargetType] {
                    paste! {
                        #[quickcheck]
                        fn [<test_ SourceType:lower _to_ TargetType:lower>](value: SourceType) {
                            #[allow(clippy::clone_on_copy)]
                            let converted: TargetType = TargetType::from(value.clone());
                            #[allow(clippy::unnecessary_fallible_conversions)]
                            let round_trip: Option<SourceType> = SourceType::try_from(converted).ok();
                            assert_eq!(Some(value), round_trip);
                        }
                    }
                }
            }
        }
    }

    duplicate! {
        [
            TargetType;
            [CowBigInt];
            [CowBigUint];
            [RcBigInt];
            [RcBigUint];
        ]
        paste! {
            #[test]
            fn [<test_bool_to_ TargetType:lower>]() {
                assert!(TargetType::from(true).is_one());
                assert!(TargetType::from(false).is_zero());
            }
        }
    }

    // Test fallible non-primitive conversions.
    duplicate! {
        [
            SourceType         SourceImplType;
            [BigInt]           [BigInt];
            [BigUint]          [BigUint];
            [StaticCowBigInt]  [BigInt];
            [StaticCowBigUint] [BigUint];
            [RcBigInt]         [BigInt];
            [RcBigUint]        [BigUint];
        ]
        duplicate! {
            [
                TargetType        TargetImplType;
                [BigInt]           [BigInt];
                [BigUint]          [BigUint];
                [StaticCowBigInt]  [BigInt];
                [StaticCowBigUint] [BigUint];
                [RcBigInt]         [BigInt];
                [RcBigUint]        [BigUint];
            ]
            if_tokens! {
                if [SourceType] != [TargetType] {
                    paste! {
                        #[quickcheck]
                        fn [<test_try_ SourceType:lower _to_ TargetType:lower>](value: SourceType) {
                            #[allow(clippy::unnecessary_fallible_conversions)]
                            let converted: Option<TargetType> = TargetType::try_from(value.clone()).ok();
                            #[allow(clippy::unnecessary_fallible_conversions)]
                            #[allow(clippy::useless_conversion)]
                            let expected: Option<TargetType> = TargetImplType::try_from(SourceImplType::from(value))
                                .ok().map(TargetType::from);
                            assert_eq!(converted, expected);
                        }
                    }
                }
            }
        }
    }

    // Test fallible widening conversions.
    duplicate! {
        [
            SourceType;
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
        ]
        duplicate! {
            [
                TargetType         TargetImplType;
                [BigInt]           [BigInt];
                [BigUint]          [BigUint];
                [StaticCowBigInt]  [BigInt];
                [StaticCowBigUint] [BigUint];
                [RcBigInt]         [BigInt];
                [RcBigUint]        [BigUint];
                [i8]               [BigInt];
                [i16]              [BigInt];
                [i32]              [BigInt];
                [i64]              [BigInt];
                [i128]             [BigInt];
                [isize]            [BigInt];
                [u8]               [BigUint];
                [u16]              [BigUint];
                [u32]              [BigUint];
                [u64]              [BigUint];
                [u128]             [BigUint];
                [usize]            [BigUint];
            ]
            if_tokens! {
                if SourceType in [i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize] &&
                    TargetType in [i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize] {
                    // No need to test stdlib!
                }
                if [SourceType] != [TargetType] {
                    paste! {
                        #[quickcheck]
                        fn [<test_try_ SourceType:lower _to_ TargetType:lower>](value: SourceType) {
                            #[allow(clippy::clone_on_copy)]
                            #[allow(clippy::unnecessary_fallible_conversions)]
                            let converted: Option<TargetType> = TargetType::try_from(value.clone()).ok();
                            #[allow(clippy::unnecessary_fallible_conversions)]
                            #[allow(clippy::useless_conversion)]
                            let expected: Option<TargetType> = TargetImplType::try_from(value)
                                .ok().map(TargetType::from);
                            assert_eq!(converted, expected);
                        }
                    }
                }
            }
        }
    }

    // Test To* Trait Conversions
    duplicate! {
        [
            SourceType;
            [BigInt];
            [BigUint];
            [StaticCowBigInt];
            [StaticCowBigUint];
            [RcBigInt];
            [RcBigUint];
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
        ]
        duplicate! {
            [
                TargetType         TargetImplType  method;
                [BigInt]           [BigInt]        [ToBigInt::to_bigint];
                [BigUint]          [BigUint]       [ToBigUint::to_biguint];
                [CowBigInt]        [BigInt]        [ToCowBigInt::to_cow_bigint];
                [CowBigUint]       [BigUint]       [ToCowBigUint::to_cow_biguint];
                [RcBigInt]         [BigInt]        [ToRcBigInt::to_rc_bigint];
                [RcBigUint]        [BigUint]       [ToRcBigUint::to_rc_biguint];
                [i8]               [BigInt]        [ToPrimitive::to_i8];
                [i16]              [BigInt]        [ToPrimitive::to_i16];
                [i32]              [BigInt]        [ToPrimitive::to_i32];
                [i64]              [BigInt]        [ToPrimitive::to_i64];
                [i128]             [BigInt]        [ToPrimitive::to_i128];
                [isize]            [BigInt]        [ToPrimitive::to_isize];
                [u8]               [BigUint]       [ToPrimitive::to_u8];
                [u16]              [BigUint]       [ToPrimitive::to_u16];
                [u32]              [BigUint]       [ToPrimitive::to_u32];
                [u64]              [BigUint]       [ToPrimitive::to_u64];
                [u128]             [BigUint]       [ToPrimitive::to_u128];
                [usize]            [BigUint]       [ToPrimitive::to_usize];
            ]
            if_tokens! {
                if SourceType in [i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize] &&
                    TargetType in [i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize] {
                    // No need to test stdlib!
                }
                else {
                    paste! {
                        #[quickcheck]
                        fn [<test_try_ SourceType:lower _to_ TargetType:lower _with_trait>](value: SourceType) {
                            let converted: Option<TargetType> = method(&value);
                            #[allow(irrefutable_let_patterns)]
                            #[allow(clippy::unnecessary_fallible_conversions)]
                            #[allow(clippy::useless_conversion)]
                            #[allow(clippy::clone_on_copy)]
                            if let Ok(expected_impl) = TargetImplType::try_from(value.clone()) {
                                let expected: Option<TargetType> = method(&expected_impl);
                                assert_eq!(converted, expected);
                            } else {
                                assert_eq!(converted, None);
                            }
                        }
                    }
                }
            }
        }
    }
}
