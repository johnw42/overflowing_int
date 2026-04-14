#![allow(unused_imports)]
use crate::encoding::{Decode, Decode as _, Decoded, Encode, Encoding};
use crate::signed::Int;
use crate::small_num::SmallNumber;
use crate::unsigned::Uint;
use crate::{
    CowBigInt, CowBigUint, RcBigInt, RcBigUint, duplicate_generic_bignum, duplicate_iprims,
    duplicate_iprims_if_unsigned, duplicate_prims, duplicate_uprims,
    duplicate_uprims_and_iprims_if_signed,
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

// =============================================================================
// ToBigInt/ToBigUint Traits
// =============================================================================

duplicate_generic_bignum! {
    impl<'a, E> ToBigInt for EncodedType<'a, E>
    where
        E: Encoding<'a, Big = ImplType>,
    {
        fn to_bigint(&self) -> Option<BigInt> {
            Some(BigInt::from(self))
        }
    }

    impl<'a, E> ToBigUint for EncodedType<'a, E>
    where
        E: Encoding<'a, Big = ImplType>,
    {
        fn to_biguint(&self) -> Option<BigUint> {
            #[allow(clippy::unnecessary_fallible_conversions)]
            BigUint::try_from(self).ok()
        }
    }
}

// =============================================================================
// TryFrom Signed to Unsigned
// =============================================================================

impl<'a, E: Encoding<'a, Big = BigUint>> TryFrom<BigInt> for Uint<'a, E> {
    type Error = TryFromBigIntError<BigInt>;

    fn try_from(value: BigInt) -> Result<Self, Self::Error> {
        value
            .to_biguint()
            .map(Self::from_big)
            .ok_or_else(|| TryFromBigIntError::new(value))
    }
}

impl<'a, E: Encoding<'a, Big = BigUint>> TryFrom<&BigInt> for Uint<'a, E> {
    type Error = TryFromBigIntError<()>;

    fn try_from(value: &BigInt) -> Result<Self, Self::Error> {
        value
            .to_biguint()
            .map(Self::from_big)
            .ok_or_else(|| TryFromBigIntError::new(()))
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> TryFrom<Int<'a, E>> for BigUint {
    type Error = TryFromBigIntError<Int<'a, E>>;

    fn try_from(value: Int<'a, E>) -> Result<Self, Self::Error> {
        value
            .to_biguint()
            .ok_or_else(|| TryFromBigIntError::new(value))
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> TryFrom<&Int<'a, E>> for BigUint {
    type Error = TryFromBigIntError<()>;

    fn try_from(value: &Int<'a, E>) -> Result<Self, Self::Error> {
        value
            .to_biguint()
            .ok_or_else(|| TryFromBigIntError::new(()))
    }
}

impl<'a, 'b, E1, E2> TryFrom<Int<'a, E1>> for Uint<'b, E2>
where
    E1: Encoding<'a, Big = BigInt>,
    E2: Encoding<'b, Big = BigUint>,
    E2::Small: TryFrom<E1::Small>,
{
    type Error = TryFromBigIntError<Int<'a, E1>>;

    fn try_from(value: Int<'a, E1>) -> Result<Self, Self::Error> {
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

duplicate_iprims! {
    paste! {
        impl<'a, E: Encoding<'a, Big = BigUint>> TryFrom<prim> for Uint<'a, E> {
            type Error = TryFromBigIntError<()>;

            fn try_from(value: prim) -> Result<Self, Self::Error> {
                #[allow(clippy::unnecessary_fallible_conversions)]
                if let Ok(small) = E::Small::try_from(value) {
                    Ok(Self::from_small(small))
                } else {
                    BigUint::try_from(value)
                        .ok()
                        .map(Self::from_big)
                        .ok_or_else(|| TryFromBigIntError::new(()))
                }
            }
        }
    }
}

// =============================================================================
// From Unsigned to Signed
// =============================================================================

impl<'a, E: Encoding<'a, Big = BigInt>> From<BigUint> for Int<'a, E> {
    fn from(value: BigUint) -> Self {
        Self::from_big(value.into())
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> From<&BigUint> for Int<'a, E> {
    fn from(value: &BigUint) -> Self {
        Self::from_big(value.clone().into())
    }
}

impl<'a, E: Encoding<'a, Big = BigUint>> From<Uint<'a, E>> for BigInt {
    fn from(value: Uint<'a, E>) -> Self {
        BigInt::from(value.into_big())
    }
}

impl<'a, E: Encoding<'a, Big = BigUint>> From<&Uint<'a, E>> for BigInt {
    fn from(value: &Uint<'a, E>) -> Self {
        BigInt::from(value.clone().into_big())
    }
}

impl<'a, 'b, E1, E2> From<Uint<'a, E1>> for Int<'b, E2>
where
    E1: Encoding<'a, Big = BigUint>,
    E2: Encoding<'b, Big = BigInt>,
    E2::Small: TryFrom<E1::Small>,
{
    fn from(value: Uint<'a, E1>) -> Self {
        value.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => match E2::Small::try_from(s) {
                Ok(small) => Self::from_small(small),
                Err(_) => Self::from_big(s.to_bigint()),
            },
            Decoded::Big(b) => Self::from_big(b.into_owned().into()),
        })
    }
}

duplicate_uprims! {
    impl<'a, E: Encoding<'a, Big = BigInt>> From<prim> for Int<'a, E> {
        fn from(value: prim) -> Self {
            #[allow(clippy::unnecessary_fallible_conversions)]
            match E::Small::try_from(value) {
                Ok(small) => Self::from_small(small),
                Err(_) => Self::from_big(BigInt::from(value)),
            }
        }
    }
}

// =============================================================================
// From with Same Signedness
// =============================================================================

#[duplicate_item(
    tag       signedness  ImplType    EncodedType;
    [bigint]  [signed]    [BigInt]    [Int];
    [biguint] [unsigned]  [BigUint]   [Uint];
)]
pub mod tag {
    use crate::duplicate_prims_with_signedness;

    use super::*;

    impl<'a, E: Encoding<'a, Big = ImplType>> From<ImplType> for EncodedType<'a, E> {
        fn from(value: ImplType) -> Self {
            Self::from_big(value)
        }
    }

    impl<'a, E: Encoding<'a, Big = ImplType>> From<&ImplType> for EncodedType<'a, E> {
        fn from(value: &ImplType) -> Self {
            Self::from_big_cow(Cow::Owned(value.clone()))
        }
    }

    impl<'a, E: Encoding<'a, Big = ImplType>> From<Cow<'a, ImplType>> for EncodedType<'a, E> {
        fn from(value: Cow<'a, ImplType>) -> Self {
            Self::from_big_cow(value)
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

    duplicate_prims_with_signedness! { signedness;
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
        }
    }

    // to primitives
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
// MARK: Unit Tests for Conversions
// =============================================================================

#[cfg(test)]
mod test {
    use if_tokens::if_tokens;
    use quickcheck_macros::quickcheck;

    use super::*;

    type StaticCowBigInt = CowBigInt<'static>;
    type StaticCowBigUint = CowBigUint<'static>;

    // Test infallible conversion from foreign types to signed encoded types.
    duplicate! {
        [
            SourceType;
            [BigInt];
            [i8];
            [i16];
            [i32];
            [i64];
            [i128];
            [isize];
            [BigUint];
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
                [StaticCowBigInt];
                [RcBigInt];
            ]
            paste! {
                #[quickcheck]
                fn [<test_ SourceType:lower _to_ TargetType:lower>](value: SourceType) {
                    // #[allow(clippy::clone_on_copy)]
                    let converted: TargetType = TargetType::from(value.clone());
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    let round_trip: Option<SourceType> = SourceType::try_from(converted).ok();
                    assert_eq!(Some(value), round_trip);
                }
            }
        }
    }

    // Test infallible conversion from foreign type refs to signed encoded types.
    duplicate! {
        [
            SourceType;
            [BigInt];
            [BigUint];

        ]
        duplicate! {
            [
                TargetType;
                [StaticCowBigInt];
                [RcBigInt];
            ]
            paste! {
                #[quickcheck]
                fn [<test_ref_ SourceType:lower _to_ TargetType:lower>](value: SourceType) {
                    // #[allow(clippy::clone_on_copy)]
                    let converted: TargetType = TargetType::from(&value);
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    let round_trip: Option<SourceType> = SourceType::try_from(converted).ok();
                    assert_eq!(Some(value), round_trip);
                }
            }
        }
    }

    // Test infallible conversion from unsigned foreign types to unsigned encoded types.
    duplicate! {
        [
            SourceType;
            [BigUint];
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
                [StaticCowBigUint];
                [RcBigUint];
            ]
            paste! {
                #[quickcheck]
                fn [<test_ SourceType:lower _to_ TargetType:lower>](value: SourceType) {
                    // #[allow(clippy::clone_on_copy)]
                    let converted: TargetType = TargetType::from(value.clone());
                    // #[allow(clippy::unnecessary_fallible_conversions)]
                    let round_trip: Option<SourceType> = SourceType::try_from(converted).ok();
                    assert_eq!(Some(value), round_trip);
                }
            }
        }
    }

    // Test infallible conversion from unsigned foreign type refs to unsigned encoded types.
    duplicate! {
        [
            SourceType;
            [BigUint];
        ]
        duplicate! {
            [
                TargetType;
                [StaticCowBigUint];
                [RcBigUint];
            ]
            paste! {
                #[quickcheck]
                fn [<test_ref_ SourceType:lower _to_ TargetType:lower>](value: SourceType) {
                    let converted: TargetType = TargetType::from(&value);
                    // #[allow(clippy::unnecessary_fallible_conversions)]
                    let round_trip: Option<SourceType> = SourceType::try_from(converted).ok();
                    assert_eq!(Some(value), round_trip);
                }
            }
        }
    }

    // Test conversion from bool to encoded types.
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

    // Test fallible conversions to/from encoded types from owned value.
    duplicate! {
        [
            EncodedType        ImplType;
            [StaticCowBigInt]  [BigInt];
            [StaticCowBigUint] [BigUint];
            [RcBigInt]         [BigInt];
            [RcBigUint]        [BigUint];
        ]
        duplicate! {
            [
                ForeignType;
                [BigInt];
                [BigUint];
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
            paste! {
                #[quickcheck]
                fn [<test_try_ EncodedType:lower _to_ ForeignType:lower>](value: EncodedType) {
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    let converted: Option<ForeignType> = ForeignType::try_from(value.clone()).ok();
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    #[allow(clippy::useless_conversion)]
                    let expected: Option<ForeignType> = ForeignType::try_from(ImplType::from(value))
                        .ok().and_then(|n| ForeignType::try_from(n).ok());
                    assert_eq!(converted, expected);
                }

                #[quickcheck]
                fn [<test_try_ ForeignType:lower _to_ EncodedType:lower>](value: ForeignType) {
                    #[allow(clippy::clone_on_copy)]
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    let converted: Option<EncodedType> = EncodedType::try_from(value.clone()).ok();
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    #[allow(clippy::useless_conversion)]
                    let expected: Option<EncodedType> = ImplType::try_from(value)
                        .ok().and_then(|n| EncodedType::try_from(n).ok());
                    assert_eq!(converted, expected);
                }
            }
        }
    }

    // Test fallible conversions to/from encoded type from ref.
    duplicate! {
        [
            EncodedType        ImplType;
            [StaticCowBigInt]  [BigInt];
            [StaticCowBigUint] [BigUint];
            [RcBigInt]         [BigInt];
            [RcBigUint]        [BigUint];
        ]
        duplicate! {
            [
                ForeignType;
                [BigInt];
                [BigUint];
            ]
            paste! {
                #[quickcheck]
                fn [<test_try_ref_ EncodedType:lower _to_ ForeignType:lower>](value: EncodedType) {
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    let converted: Option<ForeignType> = ForeignType::try_from(&value).ok();
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    #[allow(clippy::useless_conversion)]
                    let expected: Option<ForeignType> = ForeignType::try_from(ImplType::from(value))
                        .ok().and_then(|n| ForeignType::try_from(n).ok());
                    assert_eq!(converted, expected);
                }

                #[quickcheck]
                fn [<test_try_ref_ EncodedType:lower _to ForeignType:lower>](value: EncodedType) {
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    let converted: Option<ForeignType> = [<To ForeignType>]::[<to_ ForeignType:lower>](&value);
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    #[allow(clippy::useless_conversion)]
                    let expected: Option<ForeignType> = ForeignType::try_from(ImplType::from(value))
                        .ok().and_then(|n| ForeignType::try_from(n).ok());
                    assert_eq!(converted, expected);
                }

                #[quickcheck]
                fn [<test_try_ref_ ForeignType:lower _to_ EncodedType:lower>](value: ForeignType) {
                    #[allow(clippy::clone_on_copy)]
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    let converted: Option<EncodedType> = EncodedType::try_from(&value).ok();
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    #[allow(clippy::useless_conversion)]
                    let expected: Option<EncodedType> = ImplType::try_from(value)
                        .ok().and_then(|n| EncodedType::try_from(n).ok());
                    assert_eq!(converted, expected);
                }
            }
        }
    }
}
