use num_bigint::{BigInt, BigUint, Sign, ToBigInt, ToBigUint};
use num_traits::ToPrimitive;
use paste::paste;
use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

use crate::duplicate_prims;
use crate::generic_bignum::GenericBigNum;
use crate::generic_bignum::encoding::{Decoded, EncodedBigNum};

// impl<'a, E: EncodedBigNum<'a>> ToBigInt for GenericBigNum<'a, E> {
//     fn to_bigint(&self) -> Option<BigInt> {
//         Cow::into_owned(self.big_cow()).to_bigint()
//     }
// }

// impl<'a, E: EncodedBigNum<'a, Big = BigUint>> ToBigUint for GenericBigNum<'a, E> {
//     fn to_biguint(&self) -> Option<BigUint> {
//         Cow::into_owned(self.big_cow()).to_biguint()
//     }
// }

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> From<&'a BigInt> for GenericBigNum<'a, E> {
    fn from(value: &'a BigInt) -> Self {
        Self::from_big_cow(Cow::Borrowed(value))
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> From<BigInt> for GenericBigNum<'a, E> {
    fn from(value: BigInt) -> Self {
        Self::from_big(value)
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> From<BigUint> for GenericBigNum<'a, E> {
    fn from(value: BigUint) -> Self {
        Self::from_big(BigInt::from_biguint(Sign::Plus, value))
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> From<GenericBigNum<'a, E>> for BigInt {
    fn from(value: GenericBigNum<'a, E>) -> Self {
        match value.decode() {
            Decoded::Small(n) => n.into(),
            Decoded::Big(n) => n.into_owned(),
        }
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> From<&GenericBigNum<'a, E>> for BigInt {
    fn from(value: &GenericBigNum<'a, E>) -> Self {
        match value.decode_ref() {
            Decoded::Small(n) => n.into(),
            Decoded::Big(n) => n.into_owned(),
        }
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> TryFrom<GenericBigNum<'a, E>> for BigUint {
    type Error = ();

    fn try_from(value: GenericBigNum<'a, E>) -> Result<Self, Self::Error> {
        match value.decode() {
            Decoded::Small(n) => n.try_into().map_err(|_| ()),
            Decoded::Big(n) => n.into_owned().try_into().map_err(|_| ()),
        }
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> TryFrom<&GenericBigNum<'a, E>> for BigUint {
    type Error = ();

    fn try_from(value: &GenericBigNum<'a, E>) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> From<bool> for GenericBigNum<'a, E> {
    fn from(value: bool) -> Self {
        (value as u8).into()
    }
}

duplicate_prims! {
    paste! {
        impl<'a, E: EncodedBigNum<'a, Big=BigInt>> From<prim> for GenericBigNum<'a, E> {
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

        impl<'a, E: EncodedBigNum<'a, Big=BigInt>> TryFrom<GenericBigNum<'a, E>> for prim {
            type Error = ();

            fn try_from(value: GenericBigNum<'a, E>) -> Result<Self, Self::Error> {
                prim::try_from(&value)
            }
        }

        impl<'a, E: EncodedBigNum<'a, Big=BigInt>> TryFrom<&GenericBigNum<'a, E>> for prim {
            type Error = ();
            fn try_from(value: &GenericBigNum<'a, E>) -> Result<Self, Self::Error> {
                match value.decode_ref() {
                    Decoded::Small(n) => n.[<to_ prim>]().ok_or(()),
                    Decoded::Big(n) => n.[<to_ prim>]().ok_or(()),
                }
            }
        }
    }
}
