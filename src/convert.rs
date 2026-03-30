use crate::SmallInt;
use crate::big_integer::BigInteger as _;
use crate::cbigint::CBigInt;
use crate::encoding::{Encoded, Encoding};
use num_bigint::{BigInt, BigUint, Sign::*, ToBigInt, ToBigUint, TryFromBigIntError};
use num_traits::ToPrimitive;
use paste::paste;
use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

impl ToBigInt for CBigInt {
    fn to_bigint(&self) -> Option<BigInt> {
        Some(Cow::into_owned(self.to_bigint_cow()))
    }
}

impl ToBigUint for CBigInt {
    fn to_biguint(&self) -> Option<BigUint> {
        self.clone().try_into().ok()
    }
}

impl From<&BigInt> for CBigInt {
    fn from(value: &BigInt) -> Self {
        CBigInt::from(value.clone())
    }
}

impl From<&CBigInt> for BigInt {
    fn from(value: &CBigInt) -> Self {
        value.0.clone().into()
    }
}

impl<'a> From<&'a CBigInt> for Cow<'a, BigInt> {
    fn from(value: &'a CBigInt) -> Self {
        match value.0.borrow_encoding() {
            Encoding::Small(n) => Cow::<'a, BigInt>::Owned((*n).into()),
            Encoding::Big(n) => Cow::<'a, BigInt>::Borrowed(n),
        }
    }
}

impl From<BigInt> for CBigInt {
    fn from(value: BigInt) -> Self {
        CBigInt(value.into())
    }
}

impl From<BigUint> for CBigInt {
    fn from(value: BigUint) -> Self {
        Self::from_biguint(Plus, value)
    }
}

impl From<CBigInt> for BigInt {
    fn from(value: CBigInt) -> Self {
        match value.0.into_encoding() {
            Encoding::Small(n) => BigInt::from(n),
            Encoding::Big(n) => n,
        }
    }
}

impl TryFrom<CBigInt> for BigUint {
    type Error = TryFromBigIntError<BigInt>;

    fn try_from(value: CBigInt) -> Result<Self, Self::Error> {
        match value.0.into_encoding() {
            Encoding::Small(n) => BigInt::from(n).try_into(),
            Encoding::Big(n) => n.try_into(),
        }
    }
}

impl TryFrom<&CBigInt> for BigUint {
    type Error = TryFromBigIntError<BigInt>;

    fn try_from(value: &CBigInt) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}

impl From<bool> for CBigInt {
    fn from(value: bool) -> Self {
        SmallInt::from(value).into()
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
        impl From<prim> for CBigInt {
            fn from(value: prim) -> Self {
                #[allow(irrefutable_let_patterns)]
                #[allow(clippy::unnecessary_fallible_conversions)]
                if let Ok(n) = SmallInt::try_from(value) {
                    CBigInt(Encoded::from_small(n))
                } else {
                    BigInt::from(value).into()
                }
            }
        }

        impl TryFrom<CBigInt> for prim {
            type Error = TryFromBigIntError<BigInt>;
            fn try_from(value: CBigInt) -> Result<Self, Self::Error> {
                if let Some(n) = value.to_small() {
                    match n.[< to_ prim >]() {
                        Some(prim) => Ok(prim),
                        None => {
                            // This is guaranteed to fail; it's done because there's no more
                            // straightforward way to construct an appropriate TryFromBigIntError.
                            prim::try_from(BigInt::from(value))
                        }
                    }
                } else {
                    prim::try_from(BigInt::from(value))
                }
            }
        }

        impl TryFrom<&CBigInt> for prim {
            type Error = TryFromBigIntError<BigInt>;
            fn try_from(value: &CBigInt) -> Result<Self, Self::Error> {
                if let Some(n) = value.to_small() {
                    match n.[< to_ prim >]() {
                        Some(prim) => Ok(prim),
                        None => {
                            // This is guaranteed to fail; it's done because there's no more
                            // straightforward way to construct an appropriate TryFromBigIntError.
                            prim::try_from(BigInt::from(value))
                        }
                    }
                } else {
                    prim::try_from(BigInt::from(value))
                }
            }
        }
    }
}
