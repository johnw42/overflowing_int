use crate::CBigInt;
use crate::SmallInt;
use crate::encoding::IntoEncoding;
use crate::encoding::{Encoded, Encoding};
use num_bigint::{BigInt, BigUint, Sign::*, ToBigInt, ToBigUint, TryFromBigIntError};
use num_traits::ToPrimitive;
use paste::paste;
use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

impl<'a> ToBigInt for CBigInt<'a> {
    fn to_bigint(&self) -> Option<BigInt> {
        Some(Cow::into_owned(Cow::from(self)))
    }
}

impl<'a> ToBigUint for CBigInt<'a> {
    fn to_biguint(&self) -> Option<BigUint> {
        self.clone().try_into().ok()
    }
}

impl<'a> From<&BigInt> for CBigInt<'a> {
    fn from(value: &BigInt) -> Self {
        CBigInt::from(value.clone())
    }
}

impl<'a> From<&CBigInt<'a>> for BigInt {
    fn from(value: &CBigInt<'a>) -> Self {
        match value.into_encoding() {
            Encoding::Small(n) => BigInt::from(n),
            Encoding::Big(n) => n.into_owned(),
        }
    }
}

impl<'a> From<&CBigInt<'a>> for Cow<'a, BigInt> {
    fn from(value: &CBigInt<'a>) -> Self {
        match value.into_encoding() {
            Encoding::Small(n) => Cow::<'a, BigInt>::Owned(n.into()),
            Encoding::Big(n) => n,
        }
    }
}

impl<'a> From<BigInt> for CBigInt<'a> {
    fn from(value: BigInt) -> Self {
        CBigInt(Encoded(Encoding::from_big(value)))
    }
}

impl<'a> From<BigUint> for CBigInt<'a> {
    fn from(value: BigUint) -> Self {
        Self::from_biguint(Plus, value)
    }
}

impl<'a> From<CBigInt<'a>> for BigInt {
    fn from(value: CBigInt<'a>) -> Self {
        match value.into_encoding() {
            Encoding::Small(n) => BigInt::from(n),
            Encoding::Big(n) => n.into_owned(),
        }
    }
}

impl<'a> TryFrom<CBigInt<'a>> for BigUint {
    type Error = TryFromBigIntError<BigInt>;

    fn try_from(value: CBigInt<'a>) -> Result<Self, Self::Error> {
        match value.into_encoding() {
            Encoding::Small(n) => BigInt::from(n).try_into(),
            Encoding::Big(n) => n.into_owned().try_into(),
        }
    }
}

impl<'a> TryFrom<&CBigInt<'a>> for BigUint {
    type Error = TryFromBigIntError<BigInt>;

    fn try_from(value: &CBigInt<'a>) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}

impl From<bool> for CBigInt<'_> {
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
        impl From<prim> for CBigInt<'_> {
            fn from(value: prim) -> Self {
                #[allow(irrefutable_let_patterns)]
                #[allow(clippy::unnecessary_fallible_conversions)]
                if let Ok(n) = SmallInt::try_from(value) {
                    CBigInt(Encoded(Encoding::from_small(n)))
                } else {
                    BigInt::from(value).into()
                }
            }
        }

        impl<'a> TryFrom<CBigInt<'a>> for prim {
            type Error = TryFromBigIntError<BigInt>;
            fn try_from(value: CBigInt<'a>) -> Result<Self, Self::Error> {
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

        impl<'a> TryFrom<&CBigInt<'a>> for prim {
            type Error = TryFromBigIntError<BigInt>;
            fn try_from(value: &CBigInt<'a>) -> Result<Self, Self::Error> {
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
