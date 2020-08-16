use crate::cbigint::CBigInt;
use crate::decoded::Decoded;
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

impl<'a> ToCow<'a> for Decoded<Cow<'a, BigInt>> {
    fn to_cow(self) -> Cow<'a, BigInt> {
        match self {
            Decoded::Digit(n) => Cow::Owned(n.into()),
            Decoded::Big(cow) => cow,
        }
    }
}

pub trait ToDecodedCow<'a> {
    fn to_decoded_cow(self) -> Decoded<Cow<'a, BigInt>>;
}

impl<'a> ToDecodedCow<'a> for CBigInt {
    fn to_decoded_cow(self) -> Decoded<Cow<'a, BigInt>> {
        match self.decode() {
            Decoded::Digit(n) => Decoded::Digit(n),
            Decoded::Big(n) => Decoded::Big(n.to_cow()),
        }
    }
}

impl<'a> ToDecodedCow<'a> for &'a CBigInt {
    fn to_decoded_cow(self) -> Decoded<Cow<'a, BigInt>> {
        match self.decode_ref() {
            Decoded::Digit(n) => Decoded::Digit(n),
            Decoded::Big(n) => Decoded::Big(n.to_cow()),
        }
    }
}
