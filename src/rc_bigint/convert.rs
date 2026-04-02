use crate::rc_bigint::RcBigInt;
use crate::rc_bigint::encoding::{Encoding, RefEncoding};
use crate::rc_bigint::small_num::SmallInt;

use num_bigint::{BigInt, BigUint, Sign::*, ToBigInt, ToBigUint, TryFromBigIntError};
use num_traits::ToPrimitive;
use paste::paste;
use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

impl ToBigInt for RcBigInt {
    fn to_bigint(&self) -> Option<BigInt> {
        Some(Cow::into_owned(Cow::from(self)))
    }
}

impl ToBigUint for RcBigInt {
    fn to_biguint(&self) -> Option<BigUint> {
        self.clone().try_into().ok()
    }
}

impl From<&BigInt> for RcBigInt {
    fn from(value: &BigInt) -> Self {
        RcBigInt::from(value.clone())
    }
}

impl From<&RcBigInt> for BigInt {
    fn from(value: &RcBigInt) -> Self {
        match value.decode_ref() {
            RefEncoding::Small(n) => BigInt::from(n),
            RefEncoding::Big(n) => n.clone(),
        }
    }
}

impl From<&RcBigInt> for Rc<BigInt> {
    fn from(value: &RcBigInt) -> Self {
        match value.clone().decode() {
            Encoding::Small(n) => Rc::new(BigInt::from(n)),
            Encoding::Big(n) => Rc::clone(&n),
        }
    }
}

impl From<BigInt> for RcBigInt {
    fn from(value: BigInt) -> Self {
        Encoded::from_big(value).into()
    }
}

impl From<BigUint> for RcBigInt {
    fn from(value: BigUint) -> Self {
        Self::from_biguint(Plus, value)
    }
}

impl From<RcBigInt> for BigInt {
    fn from(value: RcBigInt) -> Self {
        match value.into_encoding() {
            Encoding::Small(n) => BigInt::from(n),
            Encoding::Big(n) => n.into_owned(),
        }
    }
}

impl TryFrom<RcBigInt> for BigUint {
    type Error = TryFromBigIntError<BigInt>;

    fn try_from(value: RcBigInt) -> Result<Self, Self::Error> {
        match value.into_encoding() {
            Encoding::Small(n) => BigInt::from(n).try_into(),
            Encoding::Big(n) => n.into_owned().try_into(),
        }
    }
}

impl TryFrom<&RcBigInt> for BigUint {
    type Error = TryFromBigIntError<BigInt>;

    fn try_from(value: &RcBigInt) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}

impl From<bool> for RcBigInt {
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
        impl From<prim> for RcBigInt {
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

        impl TryFrom<RcBigInt> for prim {
            type Error = TryFromBigIntError<BigInt>;
            fn try_from(value: RcBigInt) -> Result<Self, Self::Error> {
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

        impl TryFrom<&RcBigInt> for prim {
            type Error = TryFromBigIntError<BigInt>;
            fn try_from(value: &RcBigInt) -> Result<Self, Self::Error> {
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
