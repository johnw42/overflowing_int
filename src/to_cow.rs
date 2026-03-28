use crate::cbigint::CBigInt;
use crate::encoding::Encoded;
use num_bigint::BigInt;
use std::borrow::Cow;

pub trait ToCow<'a> {
    fn to_cow(self) -> Cow<'a, BigInt>;
}

impl<'a> ToCow<'a> for CBigInt {
    fn to_cow(self) -> Cow<'a, BigInt> {
        Cow::Owned(BigInt::from(self))
    }
}

impl<'a> ToCow<'a> for &'a CBigInt {
    fn to_cow(self) -> Cow<'a, BigInt> {
        self.to_bigint()
    }
}

impl<'a> ToCow<'a> for BigInt {
    fn to_cow(self) -> Cow<'a, BigInt> {
        Cow::Owned(self)
    }
}

impl<'a> ToCow<'a> for &'a BigInt {
    fn to_cow(self) -> Cow<'a, BigInt> {
        Cow::Borrowed(self)
    }
}

impl<'a> ToCow<'a> for Encoded<Cow<'a, BigInt>> {
    fn to_cow(self) -> Cow<'a, BigInt> {
        match self {
            Encoded::Digit(n) => Cow::Owned(n.into()),
            Encoded::Big(cow) => cow,
        }
    }
}

pub trait ToDecodedCow<'a> {
    fn to_decoded_cow(self) -> Encoded<Cow<'a, BigInt>>;
}

impl<'a> ToDecodedCow<'a> for CBigInt {
    fn to_decoded_cow(self) -> Encoded<Cow<'a, BigInt>> {
        match self.0 {
            Encoded::Digit(n) => Encoded::Digit(n),
            Encoded::Big(n) => Encoded::Big(n.to_cow()),
        }
    }
}

impl<'a> ToDecodedCow<'a> for &'a CBigInt {
    fn to_decoded_cow(self) -> Encoded<Cow<'a, BigInt>> {
        match &self.0 {
            &Encoded::Digit(n) => Encoded::Digit(n),
            Encoded::Big(n) => Encoded::Big(n.to_cow()),
        }
    }
}
