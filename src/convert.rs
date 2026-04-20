#![allow(unused_imports)]
use crate::encoding::{Decode, Decode as _, Decoded, Encode, Encoding};
use crate::signed::Int;
use crate::small_num::SmallNumber;
use crate::unsigned::Uint;
use crate::{
    CowBigInt, CowBigUint, RcBigInt, RcBigUint, duplicate_generic_bignum, duplicate_iprims,
    duplicate_iprims_if_unsigned, duplicate_prims, duplicate_signed_encoded_types,
    duplicate_unsigned_encoded_types, duplicate_uprims, duplicate_uprims_and_iprims_if_signed,
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

impl<T> Display for TryFromBigIntError<T>
where
    T: Debug,
{
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
    impl<'enc, E> ToBigInt for EncodedType<E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn to_bigint(&self) -> Option<BigInt> {
            Some(BigInt::from(self))
        }
    }

    impl<'enc, E> ToBigUint for EncodedType<E>
    where
        E: Encoding<'enc, Big = ImplType>,
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

impl<'enc, E> TryFrom<BigInt> for Uint<E>
where
    E: Encoding<'enc, Big = BigUint>,
{
    type Error = TryFromBigIntError<BigInt>;

    fn try_from(value: BigInt) -> Result<Self, Self::Error> {
        value
            .to_biguint()
            .map(Self::from_big)
            .ok_or_else(|| TryFromBigIntError::new(value))
    }
}

impl<'enc, E> TryFrom<&BigInt> for Uint<E>
where
    E: Encoding<'enc, Big = BigUint>,
{
    type Error = TryFromBigIntError<()>;

    fn try_from(value: &BigInt) -> Result<Self, Self::Error> {
        value
            .to_biguint()
            .map(Self::from_big)
            .ok_or_else(|| TryFromBigIntError::new(()))
    }
}

impl<'enc, E> TryFrom<Int<E>> for BigUint
where
    E: Encoding<'enc, Big = BigInt>,
{
    type Error = TryFromBigIntError<Int<E>>;

    fn try_from(value: Int<E>) -> Result<Self, Self::Error> {
        value
            .to_biguint()
            .map(Uint::into_big)
            .ok_or_else(|| TryFromBigIntError::new(value))
    }
}

impl<'enc, E> TryFrom<&Int<E>> for BigUint
where
    E: Encoding<'enc, Big = BigInt>,
{
    type Error = TryFromBigIntError<()>;

    fn try_from(value: &Int<E>) -> Result<Self, Self::Error> {
        value
            .to_biguint()
            .map(Uint::into_big)
            .ok_or_else(|| TryFromBigIntError::new(()))
    }
}

impl<'e1, 'e2, E1, E2> TryFrom<Int<E1>> for Uint<E2>
where
    E1: Encoding<'e1, Big = BigInt>,
    E2: Encoding<'e2, Big = BigUint>,
    E2::Small: TryFrom<E1::Small>,
{
    type Error = TryFromBigIntError<Int<E1>>;

    fn try_from(value: Int<E1>) -> Result<Self, Self::Error> {
        match value.decode() {
            Decoded::Small(s) => match E2::Small::try_from(s) {
                Ok(u) => Some(Self::from_small(u)),
                Err(_) => s.to_biguint().map(Self::from_big),
            },
            Decoded::Big(b) => b.to_biguint().map(Self::from_big),
        }
        .ok_or_else(|| TryFromBigIntError::new(value))
    }
}

duplicate_iprims! {
    paste! {
        impl<'enc, E> TryFrom<prim> for Uint<E>
        where
            E: Encoding<'enc, Big = BigUint>,
        {
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

impl<'enc, E> From<BigUint> for Int<E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    fn from(value: BigUint) -> Self {
        Self::from_big(value.into())
    }
}

impl<'enc, E> From<&BigUint> for Int<E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    fn from(value: &BigUint) -> Self {
        Self::from_big(value.clone().into())
    }
}

impl<'enc, E> From<Uint<E>> for BigInt
where
    E: Encoding<'enc, Big = BigUint>,
{
    fn from(value: Uint<E>) -> Self {
        BigInt::from(value.into_big())
    }
}

impl<'enc, E> From<&Uint<E>> for BigInt
where
    E: Encoding<'enc, Big = BigUint>,
{
    fn from(value: &Uint<E>) -> Self {
        BigInt::from(value.clone().into_big())
    }
}

impl<'e1, 'e2, E1, E2> From<Uint<E1>> for Int<E2>
where
    E1: Encoding<'e1, Big = BigUint>,
    E2: Encoding<'e2, Big = BigInt>,
    E2::Small: TryFrom<E1::Small>,
{
    fn from(value: Uint<E1>) -> Self {
        match value.decode() {
            Decoded::Small(s) => match E2::Small::try_from(s) {
                Ok(small) => Self::from_small(small),
                Err(_) => Self::from_big(s.to_big().into()),
            },
            Decoded::Big(b) => Self::from_big(b.into_owned().into()),
        }
    }
}

duplicate_uprims! {
    impl<'enc, E> From<prim> for Int<E>
    where
        E: Encoding<'enc, Big = BigInt>,
    {
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

    impl<'enc, E> From<ImplType> for EncodedType<E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn from(value: ImplType) -> Self {
            Self::from_big(value)
        }
    }

    impl<'enc, E> From<&ImplType> for EncodedType<E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn from(value: &ImplType) -> Self {
            Self::from_big(value.clone())
        }
    }

    impl<'enc, E> From<EncodedType<E>> for ImplType
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn from(value: EncodedType<E>) -> Self {
            value.0.into_big()
        }
    }

    impl<'enc, E> From<&EncodedType<E>> for ImplType
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn from(value: &EncodedType<E>) -> Self {
            value.clone().0.into_big()
        }
    }

    impl<'enc, E> From<bool> for EncodedType<E>
    where
        E: Encoding<'enc, Big = ImplType>,
    {
        fn from(value: bool) -> Self {
            u8::from(value).into()
        }
    }

    duplicate_prims_with_signedness! { signedness;
        paste! {
            impl<'enc, E> From<prim> for EncodedType<E>
            where
                E: Encoding<'enc, Big=ImplType>,
            {
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
            impl<'enc, E> TryFrom<EncodedType<E>> for prim
            where
                E: Encoding<'enc, Big=ImplType>,
            {
                    type Error = TryFromBigIntError<EncodedType<E>>;

                    fn try_from(value: EncodedType<E>) -> Result<Self, Self::Error> {
                        match value.decode() {
                            Decoded::Small(n) => n.[<to_ prim>](),
                            Decoded::Big(n) => n.[<to_ prim>](),
                        }.ok_or_else(|| TryFromBigIntError::new(value))
                    }
                }

            impl<'enc, 'a, E> TryFrom<&'a EncodedType<E>> for prim
            where
                E: Encoding<'enc, Big=ImplType>,
            {
                type Error = TryFromBigIntError<()>;

                fn try_from(value: &'a EncodedType<E>) -> Result<Self, Self::Error> {
                    match value.decode() {
                        Decoded::Small(n) => n.[<to_ prim>](),
                        Decoded::Big(n) => n.[<to_ prim>](),
                    }.ok_or(TryFromBigIntError::new(()))
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

    use crate::duplicate_encoded_types;

    use super::*;

    // Test infallible conversion from foreign types to signed encoded types.
    duplicate! {
        [
            ForeignType;
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
        duplicate_signed_encoded_types! {
            paste! {
                #[quickcheck]
                fn [<test_ ForeignType:lower _to_ encoding_tag>](value: ForeignType) {
                    // #[allow(clippy::clone_on_copy)]
                    let converted: EncodedType = EncodedType::from(value.clone());
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    let round_trip: Option<ForeignType> = ForeignType::try_from(converted).ok();
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
        duplicate_signed_encoded_types! {
            paste! {
                #[quickcheck]
                fn [<test_ref_ SourceType:lower _to_ encoding_tag>](value: SourceType) {
                    // #[allow(clippy::clone_on_copy)]
                    let converted: EncodedType = EncodedType::from(&value);
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
        duplicate_unsigned_encoded_types! {
            paste! {
                #[quickcheck]
                fn [<test_ SourceType:lower _to_ encoding_tag>](value: SourceType) {
                    // #[allow(clippy::clone_on_copy)]
                    let converted: EncodedType = EncodedType::from(value.clone());
                    // #[allow(clippy::unnecessary_fallible_conversions)]
                    let round_trip: Option<SourceType> = SourceType::try_from(converted).ok();
                    assert_eq!(Some(value), round_trip);
                }
            }
        }
    }

    // Test infallible conversion from unsigned foreign type refs to unsigned encoded types.
    duplicate_unsigned_encoded_types! {
        paste! {
            #[quickcheck]
            fn [<test_ref_biguint_to_ encoding_tag>](value: BigUint) {
                let converted: EncodedType = EncodedType::from(&value);
                // #[allow(clippy::unnecessary_fallible_conversions)]
                let round_trip: Option<BigUint> = BigUint::try_from(converted).ok();
                assert_eq!(Some(value), round_trip);
            }
        }
    }

    // Test conversion from bool to encoded types.
    duplicate_encoded_types! {
        paste! {
            #[test]
            fn [<test_bool_to_ encoding_tag>]() {
                assert!(EncodedType::from(true).is_one());
                assert!(EncodedType::from(false).is_zero());
            }
        }
    }

    // Test fallible conversions to/from encoded types from owned value.
    duplicate_encoded_types! {
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
                fn [<test_try_ encoding_tag _to_ ForeignType:lower>](value: EncodedType) {
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    let converted: Option<ForeignType> = ForeignType::try_from(value.clone()).ok();
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    #[allow(clippy::useless_conversion)]
                    let expected: Option<ForeignType> = ForeignType::try_from(ImplType::from(value))
                        .ok().and_then(|n| ForeignType::try_from(n).ok());
                    assert_eq!(converted, expected);
                }

                #[quickcheck]
                fn [<test_try_ ForeignType:lower _to_ encoding_tag>](value: ForeignType) {
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
    duplicate_encoded_types! {
        duplicate! {
            [
                ForeignType;
                [BigInt];
                [BigUint];
            ]
            paste! {
                #[quickcheck]
                fn [<test_try_ref_ encoding_tag _to_ ForeignType:lower>](value: EncodedType) {
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    let converted: Option<ForeignType> = ForeignType::try_from(&value).ok();
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    #[allow(clippy::useless_conversion)]
                    let expected: Option<ForeignType> = ForeignType::try_from(ImplType::from(value))
                        .ok().and_then(|n| ForeignType::try_from(n).ok());
                    assert_eq!(converted, expected);
                }

                #[quickcheck]
                fn [<test_try_ref_ encoding_tag _to ForeignType:lower>](value: EncodedType) {
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    let converted: Option<ForeignType> = [<To ForeignType>]::[<to_ ForeignType:lower>](&value);
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    #[allow(clippy::useless_conversion)]
                    let expected: Option<ForeignType> = ForeignType::try_from(ImplType::from(value))
                        .ok().and_then(|n| ForeignType::try_from(n).ok());
                    assert_eq!(converted, expected);
                }

                #[quickcheck]
                fn [<test_try_ref_ ForeignType:lower _to_ encoding_tag>](value: ForeignType) {
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
