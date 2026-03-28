use std::borrow::Cow;

use num_bigint::BigInt;

use crate::Digit;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Encoded<T> {
    Digit(Digit),
    Big(T),
}

impl Encoded<BigInt> {
    pub fn zero() -> Self {
        Encoded::Digit(0)
    }

    pub fn one() -> Self {
        Encoded::Digit(1)
    }

    pub fn is_zero(&self) -> bool {
        *self == Self::zero()
    }

    pub fn is_one(&self) -> bool {
        *self == Self::one()
    }
}

impl From<BigInt> for Encoded<BigInt> {
    fn from(x: BigInt) -> Self {
        Encoded::Big(x)
    }
}

impl From<Encoded<BigInt>> for BigInt {
    fn from(x: Encoded<BigInt>) -> Self {
        match x {
            Encoded::Digit(n) => n.into(),
            Encoded::Big(n) => n,
        }
    }
}

impl<'a> From<Encoded<BigInt>> for Encoded<Cow<'a, BigInt>> {
    fn from(value: Encoded<BigInt>) -> Self {
        match value {
            Encoded::Digit(x) => Encoded::Digit(x),
            Encoded::Big(x) => Encoded::Big(Cow::Owned(x)),
        }
    }
}

impl<'a> From<Encoded<&'a BigInt>> for Encoded<Cow<'a, BigInt>> {
    fn from(value: Encoded<&'a BigInt>) -> Self {
        match value {
            Encoded::Digit(x) => Encoded::Digit(x),
            Encoded::Big(x) => Encoded::Big(Cow::Borrowed(x)),
        }
    }
}
