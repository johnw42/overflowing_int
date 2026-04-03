use num_bigint::{BigInt, BigUint, Sign, ToBigInt, ToBigUint};
use num_traits::ToPrimitive;
use paste::paste;
use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

use crate::generic_bigint::encoding::{Decoded, EncodedBigNum};
use crate::generic_bigint::struct_def::GenericBigInt;

impl<'a, E: EncodedBigNum<'a>> ToBigInt for GenericBigInt<'a, E> {
    fn to_bigint(&self) -> Option<BigInt> {
        Cow::into_owned(self.big_cow()).to_bigint()
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigUint>> ToBigUint for GenericBigInt<'a, E> {
    fn to_biguint(&self) -> Option<BigUint> {
        Cow::into_owned(self.big_cow()).to_biguint()
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> From<&'a BigInt> for GenericBigInt<'a, E> {
    fn from(value: &'a BigInt) -> Self {
        Self::from_big_cow(Cow::Borrowed(value))
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigUint>> From<&'a BigUint> for GenericBigInt<'a, E> {
    fn from(value: &'a BigUint) -> Self {
        Self::from_big_cow(Cow::Borrowed(value))
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> From<BigInt> for GenericBigInt<'a, E> {
    fn from(value: BigInt) -> Self {
        Self::from_big(value)
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> From<BigUint> for GenericBigInt<'a, E> {
    fn from(value: BigUint) -> Self {
        Self::from_big(BigInt::from_biguint(Sign::Plus, value))
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> From<GenericBigInt<'a, E>> for BigInt {
    fn from(value: GenericBigInt<'a, E>) -> Self {
        match value.decode() {
            Decoded::Small(n) => n.into(),
            Decoded::Big(n) => n.into_owned(),
        }
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> From<&GenericBigInt<'a, E>> for BigInt {
    fn from(value: &GenericBigInt<'a, E>) -> Self {
        match value.decode_ref() {
            Decoded::Small(n) => n.into(),
            Decoded::Big(n) => BigInt::clone(n),
        }
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigUint>> TryFrom<GenericBigInt<'a, E>> for BigUint {
    type Error = ();

    fn try_from(value: GenericBigInt<'a, E>) -> Result<Self, Self::Error> {
        match value.decode() {
            Decoded::Small(n) => n.try_into().map_err(|_| ()),
            Decoded::Big(n) => Ok(n.into_owned().into()),
        }
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigUint>> TryFrom<&GenericBigInt<'a, E>> for BigUint {
    type Error = ();

    fn try_from(value: &GenericBigInt<'a, E>) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}

impl<'a, E: EncodedBigNum<'a>> From<bool> for GenericBigInt<'a, E> {
    fn from(value: bool) -> Self {
        (value as u8).into()
    }
}

duplicate::duplicate! {
    [
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
    ]
    paste! {
        impl<'a, E: EncodedBigNum<'a>> From<prim> for GenericBigInt<'a, E> where E::Small: TryFrom<prim>, E::Big: From<prim> {
            fn from(value: prim) -> Self {
                #[allow(irrefutable_let_patterns)]
                #[allow(clippy::unnecessary_fallible_conversions)]
                if let Some(big) = E::Big::from(value) {
                    if let Ok(n) = E::Small::try_from(value) {
                        Self::from_small(n)
                    } else {
                        Self::from_big(big)
                    }
                } else {
                    value.into()
                }
            }
        }

        impl<'a, E: EncodedBigNum<'a>> TryFrom<GenericBigInt<'a, E>> for prim {
            type Error = ();

            fn try_from(value: GenericBigInt<'a, E>) -> Result<Self, Self::Error> {
                prim::try_from(&value)
            }
        }

        impl<'a, E: EncodedBigNum<'a>> TryFrom<&GenericBigInt<'a, E>> for prim {
            type Error = ();
            fn try_from(value: &GenericBigInt<'a, E>) -> Result<Self, Self::Error> {
                match value.decode_ref() {
                    Decoded::Small(n) => n.[<to_ prim>]().ok_or(()),
                    Decoded::Big(n) => n.[<to_ prim>]().ok_or(()),
                }
            }
        }
    }
}
