use super::encoding::{Encoded, Encoding, IntoEncoding as _};
use crate::CowBigInt;
use crate::SmallInt;
use num_bigint::{BigInt, BigUint, Sign::*, ToBigInt, ToBigUint, TryFromBigIntError};
use num_traits::ToPrimitive;
use paste::paste;
use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

impl<'a> ToBigInt for CowBigInt<'a> {
    fn to_bigint(&self) -> Option<BigInt> {
        Some(Cow::into_owned(Cow::from(self)))
    }
}

impl<'a> ToBigUint for CowBigInt<'a> {
    fn to_biguint(&self) -> Option<BigUint> {
        self.clone().try_into().ok()
    }
}

impl<'a> From<&BigInt> for CowBigInt<'a> {
    fn from(value: &BigInt) -> Self {
        CowBigInt::from(value.clone())
    }
}

impl<'a> From<&CowBigInt<'a>> for BigInt {
    fn from(value: &CowBigInt<'a>) -> Self {
        match value.into_encoding() {
            Encoding::Small(n) => BigInt::from(n),
            Encoding::Big(n) => n.into_owned(),
        }
    }
}

impl<'a> From<&CowBigInt<'a>> for Cow<'a, BigInt> {
    fn from(value: &CowBigInt<'a>) -> Self {
        match value.into_encoding() {
            Encoding::Small(n) => Cow::<'a, BigInt>::Owned(n.into()),
            Encoding::Big(n) => n,
        }
    }
}

impl<'a> From<BigInt> for CowBigInt<'a> {
    fn from(value: BigInt) -> Self {
        Encoded::from_big(value).into()
    }
}

impl<'a> From<BigUint> for CowBigInt<'a> {
    fn from(value: BigUint) -> Self {
        Self::from_biguint(Plus, value)
    }
}

impl<'a> From<CowBigInt<'a>> for BigInt {
    fn from(value: CowBigInt<'a>) -> Self {
        match value.into_encoding() {
            Encoding::Small(n) => BigInt::from(n),
            Encoding::Big(n) => n.into_owned(),
        }
    }
}

impl<'a> TryFrom<CowBigInt<'a>> for BigUint {
    type Error = TryFromBigIntError<BigInt>;

    fn try_from(value: CowBigInt<'a>) -> Result<Self, Self::Error> {
        match value.into_encoding() {
            Encoding::Small(n) => BigInt::from(n).try_into(),
            Encoding::Big(n) => n.into_owned().try_into(),
        }
    }
}

impl<'a> TryFrom<&CowBigInt<'a>> for BigUint {
    type Error = TryFromBigIntError<BigInt>;

    fn try_from(value: &CowBigInt<'a>) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}

impl From<bool> for CowBigInt<'_> {
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
        impl From<prim> for CowBigInt<'_> {
            fn from(value: prim) -> Self {
                #[allow(irrefutable_let_patterns)]
                #[allow(clippy::unnecessary_fallible_conversions)]
                if let Ok(n) = SmallInt::try_from(value) {
                    Encoded::from_small(n).into()
                } else {
                    BigInt::from(value).into()
                }
            }
        }

        impl<'a> TryFrom<CowBigInt<'a>> for prim {
            type Error = TryFromBigIntError<BigInt>;
            fn try_from(value: CowBigInt<'a>) -> Result<Self, Self::Error> {
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

        impl<'a> TryFrom<&CowBigInt<'a>> for prim {
            type Error = TryFromBigIntError<BigInt>;
            fn try_from(value: &CowBigInt<'a>) -> Result<Self, Self::Error> {
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
