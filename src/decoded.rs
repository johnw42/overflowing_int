use std::borrow::Cow;

use num_bigint::BigInt;

use crate::encoding::Encoded;
use crate::Digit;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Decoded<T> {
    Digit(Digit),
    Big(T),
}

impl From<BigInt> for Decoded<BigInt> {
    fn from(x: BigInt) -> Self {
        Decoded::Big(x)
    }
}

impl From<Decoded<BigInt>> for BigInt {
    fn from(x: Decoded<BigInt>) -> Self {
        match x {
            Decoded::Digit(n) => n.into(),
            Decoded::Big(n) => n,
        }
    }
}

impl Decoded<BigInt> {
    pub fn encode(self) -> Encoded {
        Encoded::encode(self)
    }
}

impl<'a> From<Decoded<BigInt>> for Decoded<Cow<'a, BigInt>> {
    fn from(value: Decoded<BigInt>) -> Self {
        match value {
            Decoded::Digit(x) => Decoded::Digit(x),
            Decoded::Big(x) => Decoded::Big(Cow::Owned(x)),
        }
    }
}

impl<'a> From<Decoded<&'a BigInt>> for Decoded<Cow<'a, BigInt>> {
    fn from(value: Decoded<&'a BigInt>) -> Self {
        match value {
            Decoded::Digit(x) => Decoded::Digit(x),
            Decoded::Big(x) => Decoded::Big(Cow::Borrowed(x)),
        }
    }
}
