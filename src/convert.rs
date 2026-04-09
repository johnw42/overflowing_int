use crate::encoding::{Decoded, Encode, Encoding};
use crate::signed::GenericSignedBigNum;
use crate::small_num::SmallNumber;
use crate::unsigned::GenericUnsignedBigNum;
use crate::{CowBigInt, CowBigUint, RcBigInt, RcBigUint, duplicate_iprims, duplicate_uprims};
use duplicate::duplicate_item;
use num_bigint::{BigInt, BigUint, Sign, ToBigInt, ToBigUint};
use num_traits::ToPrimitive;
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

#[duplicate_item(
    RawType    RcType       CowType        signedness;
    [BigInt]   [RcBigInt]   [CowBigInt]    [signed];
    [BigUint]  [RcBigUint]  [CowBigUint]   [unsigned];
)]
pub mod signedness {
    use crate::{duplicate_prims, encoding::Decode as _};

    use super::*;

    impl<'a, 'b> From<&'b RawType> for CowType<'a>
    where
        'b: 'a,
    {
        fn from(value: &'b RawType) -> Self {
            Self::from_big_cow(Cow::Borrowed(value))
        }
    }

    impl<'a> From<&'a RawType> for RcType {
        fn from(value: &'a RawType) -> Self {
            Self::from_big_cow(Cow::Owned(value.clone()))
        }
    }

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

    paste! {
        impl<'a> [<To RawType>] for CowType<'a> {
            fn [<to_ RawType:lower>](&self) -> Option<RawType> {
                self.try_into().ok()
            }
        }

        impl [<To RawType>] for RcType {
            fn [<to_ RawType:lower>](&self) -> Option<RawType> {
                self.try_into().ok()
            }
        }

        impl<'a> [<To CowType>]<'a> for RawType {
            fn [<to_cow_ RawType:lower>](&'a self) -> Option<CowType<'a>> {
                self.try_into().ok()
            }
        }

        impl<'a> [<To CowType>]<'a> for CowType<'a> {
            fn [<to_cow_ RawType:lower>](&'a self) -> Option<CowType<'a>> {
                Some(self.clone())
            }
        }

        impl<'a> [<To CowType>]<'a> for RcType {
            fn [<to_cow_ RawType:lower>](&'a self) -> Option<CowType<'a>> {
                self.try_into().ok()
            }
        }

        impl [<To RcType>] for RawType {
            fn [<to_rc_ RawType:lower>](&self) -> Option<RcType> {
                self.try_into().ok()
            }
        }

        impl<'a> [<To RcType>] for CowType<'a> {
            fn [<to_rc_ RawType:lower>](&self) -> Option<RcType> {
                self.try_into().ok()
            }
        }

        impl [<To RcType>] for RcType {
            fn [<to_rc_ RawType:lower>](&self) -> Option<RcType> {
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
        TryInto::try_into(self).ok()
    }
}

impl ToBigUint for RcBigInt {
    fn to_biguint(&self) -> Option<BigUint> {
        TryInto::try_into(self).ok()
    }
}

impl<'a> ToCowBigUint<'a> for BigInt {
    fn to_cow_biguint(&'a self) -> Option<CowBigUint<'a>> {
        TryInto::try_into(self).ok()
    }
}

impl ToRcBigUint for BigInt {
    fn to_rc_biguint(&self) -> Option<RcBigUint> {
        TryInto::try_into(self).ok()
    }
}

impl<'a> ToCowBigUint<'a> for CowBigInt<'a> {
    fn to_cow_biguint(&'a self) -> Option<CowBigUint<'a>> {
        TryInto::try_into(self).ok()
    }
}

impl<'a> ToCowBigUint<'a> for RcBigInt {
    fn to_cow_biguint(&'a self) -> Option<CowBigUint<'a>> {
        TryInto::try_into(self).ok()
    }
}

impl ToRcBigUint for RcBigInt {
    fn to_rc_biguint(&self) -> Option<RcBigUint> {
        TryInto::try_into(self).ok()
    }
}

duplicate_iprims! {
    impl<'a> ToCowBigUint<'a> for prim {
        fn to_cow_biguint(&'a self) -> Option<CowBigUint<'a>> {
            TryInto::try_into(self).ok()
        }
    }


    impl ToRcBigUint for prim {
        fn to_rc_biguint(&self) -> Option<RcBigUint> {
            TryInto::try_into(self).ok()
        }
    }
}

// =============================================================================
// To* Unsigned to Signed
// =============================================================================

impl<'a> ToBigInt for CowBigUint<'a> {
    fn to_bigint(&self) -> Option<BigInt> {
        TryInto::try_into(self).ok()
    }
}

impl ToBigInt for RcBigUint {
    fn to_bigint(&self) -> Option<BigInt> {
        TryInto::try_into(self).ok()
    }
}

impl<'a> ToCowBigInt<'a> for BigUint {
    fn to_cow_bigint(&'a self) -> Option<CowBigInt<'a>> {
        TryInto::try_into(self).ok()
    }
}

impl ToRcBigInt for BigUint {
    fn to_rc_bigint(&self) -> Option<RcBigInt> {
        TryInto::try_into(self).ok()
    }
}

impl<'a> ToCowBigInt<'a> for CowBigUint<'a> {
    fn to_cow_bigint(&'a self) -> Option<CowBigInt<'a>> {
        TryInto::try_into(self).ok()
    }
}

impl<'a> ToRcBigInt for CowBigUint<'a> {
    fn to_rc_bigint(&self) -> Option<RcBigInt> {
        TryInto::try_into(self).ok()
    }
}

impl<'a> ToCowBigInt<'a> for RcBigUint {
    fn to_cow_bigint(&'a self) -> Option<CowBigInt<'a>> {
        TryInto::try_into(self).ok()
    }
}

impl ToRcBigInt for RcBigUint {
    fn to_rc_bigint(&self) -> Option<RcBigInt> {
        TryInto::try_into(self).ok()
    }
}

duplicate_iprims! {
    impl<'a> ToCowBigInt<'a> for prim {
        fn to_cow_bigint(&'a self) -> Option<CowBigInt<'a>> {
            TryInto::try_into(self).ok()
        }
    }


    impl ToRcBigInt for prim {
        fn to_rc_bigint(&self) -> Option<RcBigInt> {
            TryInto::try_into(self).ok()
        }
    }
}

duplicate_uprims! {
    impl<'a> ToCowBigUint<'a> for prim {
        fn to_cow_biguint(&'a self) -> Option<CowBigUint<'a>> {
            self.try_into().ok()
        }
    }

    impl ToRcBigUint for prim {
        fn to_rc_biguint(&self) -> Option<RcBigUint> {
            self.try_into().ok()
        }
    }
}

// =============================================================================
// From Signed to Unsigned
// =============================================================================

// -----------------------------------------------------------------------------
// For Values
// -----------------------------------------------------------------------------

impl<'a, E: Encoding<'a, Big = BigUint>> TryFrom<BigInt> for GenericUnsignedBigNum<'a, E> {
    type Error = TryFromBigIntError<BigInt>;

    fn try_from(value: BigInt) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl<'a, E: Encoding<'a, Big = BigUint>> TryFrom<&BigInt> for GenericUnsignedBigNum<'a, E> {
    type Error = TryFromBigIntError<BigInt>;

    fn try_from(value: &BigInt) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> TryFrom<GenericSignedBigNum<'a, E>> for BigUint {
    type Error = TryFromBigIntError<GenericSignedBigNum<'a, E>>;

    fn try_from(value: GenericSignedBigNum<'a, E>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> TryFrom<&GenericSignedBigNum<'a, E>> for BigUint {
    type Error = TryFromBigIntError<()>;

    fn try_from(value: &GenericSignedBigNum<'a, E>) -> Result<Self, Self::Error> {
        todo!()
    }
}

// -----------------------------------------------------------------------------
// For References
// -----------------------------------------------------------------------------

// =============================================================================
// TryFrom Unsigned to Signed
// =============================================================================

// -----------------------------------------------------------------------------
// For Values
// -----------------------------------------------------------------------------

impl<'a, E: Encoding<'a, Big = BigInt>> From<BigUint> for GenericSignedBigNum<'a, E> {
    fn from(value: BigUint) -> Self {
        todo!()
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> From<&BigUint> for GenericSignedBigNum<'a, E> {
    fn from(value: &BigUint) -> Self {
        todo!()
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> From<GenericUnsignedBigNum<'a, E>> for BigInt {
    fn from(value: GenericUnsignedBigNum<'a, E>) -> Self {
        todo!()
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> From<&GenericUnsignedBigNum<'a, E>> for BigInt {
    fn from(value: &GenericUnsignedBigNum<'a, E>) -> Self {
        todo!()
    }
}

// -----------------------------------------------------------------------------
// For References
// -----------------------------------------------------------------------------

#[duplicate_item(
    tag       signedness  RawType    EncodedType;
    [bigint]  [signed]    [BigInt]   [GenericSignedBigNum];
    [biguint] [unsigned]  [BigUint]  [GenericUnsignedBigNum];
)]
pub mod tag {

    use crate::{
        duplicate_iprims_if_unsigned, duplicate_prims, duplicate_uprims,
        duplicate_uprims_and_iprims_if_signed,
    };

    use super::*;

    // #[duplicate_item(
    //     from_mod_name  FromRawType    FromEncodedType             from_signedness;
    //     [bigint]       [BigInt]       [GenericSignedBigNum]       [signed];
    //     [biguint]      [BigUint]      [GenericUnsignedBigNum]     [unsigned];
    // )]
    // pub mod mod_name {}

    impl<'a, E: Encoding<'a, Big = RawType>> From<RawType> for EncodedType<'a, E> {
        fn from(value: RawType) -> Self {
            Self::from_big(value)
        }
    }

    impl<'a, E: Encoding<'a, Big = RawType>> From<EncodedType<'a, E>> for RawType {
        fn from(value: EncodedType<'a, E>) -> Self {
            value.0.into_big()
        }
    }

    impl<'a, E: Encoding<'a, Big = RawType>> From<&EncodedType<'a, E>> for RawType {
        fn from(value: &EncodedType<'a, E>) -> Self {
            value.clone().0.into_big()
        }
    }

    impl<'a, E: Encoding<'a, Big = RawType>> From<bool> for EncodedType<'a, E> {
        fn from(value: bool) -> Self {
            u8::from(value).into()
        }
    }

    duplicate_uprims_and_iprims_if_signed! { signedness;
        paste! {
            impl<'a, E: Encoding<'a, Big=RawType>> From<prim> for EncodedType<'a, E> {
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

            impl<'a, E: Encoding<'a, Big=RawType>> From<&prim> for EncodedType<'a, E> {
                fn from(value: &prim) -> Self {
                    Self::from(*value)
                }
            }
        }
    }

    duplicate_iprims_if_unsigned! { signedness;
        paste! {
            impl<'a, E: Encoding<'a, Big=RawType>> TryFrom<prim> for EncodedType<'a, E> {
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

            impl<'a, E: Encoding<'a, Big=RawType>> TryFrom<&prim> for EncodedType<'a, E> {
                type Error = TryFromBigIntError<()>;

                fn try_from(value: &prim) -> Result<Self, Self::Error> {
                    Self::try_from(*value)
                }
            }
        }
    }

    duplicate_prims! {
    paste! {
        impl<'a, E: Encoding<'a, Big=RawType>> TryFrom<EncodedType<'a, E>> for prim {
                type Error = TryFromBigIntError<EncodedType<'a, E>>;

                fn try_from(value: EncodedType<'a, E>) -> Result<Self, Self::Error> {
                    value.0.with_decoded(|decoded| match decoded {
                        Decoded::Small(n) => n.[<to_ prim>](),
                        Decoded::Big(n) => n.[<to_ prim>](),
                    }).ok_or_else(|| TryFromBigIntError::new(value))
                }
            }

            impl<'a, 'b, E: Encoding<'a, Big=RawType>> TryFrom<&'b EncodedType<'a, E>> for prim {
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

#[cfg(test)]
mod test {

    use super::*;

    #[duplicate_item(
        mod_name        RawType    EncodedType   EncodedTypeStatic        signedness;
        [signed_cow]    [BigInt]   [CowBigInt]   [CowBigInt::<'static>]   [signed];
        [unsigned_cow]  [BigUint]  [CowBigUint]  [CowBigUint::<'static>]  [unsigned];
        [signed_rc]     [BigInt]   [RcBigInt]    [RcBigInt]               [signed];
        [unsigned_rc]   [BigUint]  [RcBigUint]   [RcBigUint]              [unsigned];
    )]
    mod mod_name {
        use super::*;
        use crate::{duplicate_prims, duplicate_uprims_and_iprims_if_signed};
        use quickcheck_macros::quickcheck;

        #[quickcheck]
        fn test_raw_to_encoded(raw: RawType) {
            assert_eq!(raw, RawType::from(EncodedType::from(raw.clone())));
        }

        #[quickcheck]
        fn test_ref_raw_to_encoded(raw: RawType) {
            assert_eq!(&raw, &RawType::from(EncodedType::from(&raw)));
        }

        #[quickcheck]
        fn test_bool_to_encoded(value: bool) {
            assert_eq!(
                RawType::from(value),
                RawType::from(EncodedType::from(value))
            )
        }

        duplicate_uprims_and_iprims_if_signed! { signedness;
            paste! {
                #[quickcheck]
                fn [<test_ prim _to_encoded>](n: prim) {
                    assert_eq!(RawType::from(n), RawType::from(EncodedType::from(n)))
                }

                #[quickcheck]
                fn [<test_ref_ prim _to_encoded>](n: prim) {
                    assert_eq!(RawType::from(n), RawType::from(EncodedType::from(&n)))
                }
            }
        }

        #[quickcheck]
        fn test_raw_from_encoded(encoded: EncodedTypeStatic) {
            assert_eq!(encoded, EncodedType::from(RawType::from(encoded.clone())));
        }

        #[quickcheck]
        fn test_raw_from_ref_encoded(encoded: EncodedTypeStatic) {
            assert_eq!(&encoded, &EncodedType::from(RawType::from(&encoded)));
        }

        // duplicate_prims! {
        //     paste! {
        //         #[quickcheck]
        //         fn [<test_ prim _from_encoded>](n: prim) {
        //             assert_eq!(n, prim::from(EncodedType::from(n)))
        //         }

        //         #[quickcheck]
        //         fn [<test_ref_ prim _from_encoded>](n: prim) {
        //             assert_eq!(n, prim::from(EncodedType::from(&n)))
        //         }
        //     }
        // }
    }
}
