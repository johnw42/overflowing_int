use crate::cbigint::CBigInt;
use crate::decoded::Decoded;
use crate::Digit;
use num_bigint::{BigInt, BigUint, Sign::*, ToBigInt, ToBigUint, TryFromBigIntError};
use num_traits::ToPrimitive;
use paste::paste;
use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

impl ToBigInt for CBigInt {
    fn to_bigint(&self) -> Option<BigInt> {
        Some(Cow::into_owned(self.to_bigint()))
    }
}

impl ToBigUint for CBigInt {
    fn to_biguint(&self) -> Option<BigUint> {
        self.clone().try_into().ok()
    }
}

impl From<BigInt> for CBigInt {
    fn from(value: BigInt) -> Self {
        let decoded = match Digit::try_from(value) {
            Ok(digit) => Decoded::Digit(digit),
            Err(err) => Decoded::Big(err.into_original()),
        };
        CBigInt(decoded.encode())
    }
}

impl From<BigUint> for CBigInt {
    fn from(value: BigUint) -> Self {
        Self::from_biguint(Plus, value)
    }
}

impl From<CBigInt> for BigInt {
    fn from(value: CBigInt) -> Self {
        match value.decode() {
            Decoded::Digit(n) => BigInt::from(n),
            Decoded::Big(n) => n,
        }
    }
}

// We can't construct a TryFromBigIntError directly, so we get sneaky.
fn try_into_bigint_error() -> TryFromBigIntError<()> {
    BigUint::try_from(-1).expect_err("converting -1 to BigUint fails")
}

impl TryFrom<CBigInt> for BigUint {
    type Error = TryFromBigIntError<()>;
    fn try_from(value: CBigInt) -> Result<Self, Self::Error> {
        match value.0.decode() {
            Decoded::Digit(n) => n.to_biguint(),
            Decoded::Big(n) => n.to_biguint(),
        }
        .ok_or_else(try_into_bigint_error)
    }
}

impl TryFrom<&CBigInt> for BigUint {
    type Error = TryFromBigIntError<()>;
    fn try_from(value: &CBigInt) -> Result<Self, Self::Error> {
        value.clone().try_into()
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
                if let Ok(digit) = Digit::try_from(value) {
                    CBigInt(Decoded::Digit(digit).encode())
                } else {
                    BigInt::from(value).into()
                }
            }
        }
        impl TryFrom<CBigInt> for prim {
            type Error = TryFromBigIntError<BigInt>;
            fn try_from(value: CBigInt) -> Result<Self, Self::Error> {
                if let Some(n) = value.to_digit() {
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
