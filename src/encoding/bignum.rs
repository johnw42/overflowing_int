use crate::{
    encoding::{Decode, Decoded, Encoding, OwnedEncoding},
    num_traits::small_number::SmallNumber,
};
use num_bigint::{BigInt, BigUint};
use std::borrow::Cow;

//
// BigInt
//

impl<'enc> Decode<'enc, i64> for BigInt {
    fn decode<'a>(&'a self) -> Decoded<i64, Cow<'a, <i64 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Borrowed(self))
    }

    fn into_decoded(self) -> Decoded<i64, Cow<'enc, <i64 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Owned(self))
    }
}

impl<'enc> Encoding<'enc> for BigInt {
    type Small = i64;
    type Big = Self;
    type Unsigned = BigUint;
    type Static = Self;
    type Owned = Self;
    type Borrowed<'a>
        = &'a BigInt
    where
        Self: 'a;

    const ZERO: Self = BigInt::ZERO;

    fn from_small(s: i64) -> Self {
        s.into()
    }

    fn from_big(b: <i64 as SmallNumber>::Big) -> Self {
        b
    }

    fn from_big_ref(b: &'enc <i64 as SmallNumber>::Big) -> Self::Borrowed<'enc> {
        b
    }

    fn into_static(self) -> Self::Static {
        self
    }

    fn into_owned(self) -> Self::Owned {
        self
    }

    fn borrow<'a>(&'a self) -> Self::Borrowed<'a> {
        self
    }
}

impl<'enc> OwnedEncoding<'enc> for BigInt {
    fn decode_mut(&mut self) -> Decoded<i64, &mut Self> {
        Decoded::Big(self)
    }
}

//
// BigUint
//

impl<'enc> Decode<'enc, u64> for BigUint {
    fn decode<'a>(&'a self) -> Decoded<u64, Cow<'a, <u64 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Borrowed(self))
    }

    fn into_decoded(self) -> Decoded<u64, Cow<'enc, <u64 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Owned(self))
    }
}

impl<'enc> Encoding<'enc> for BigUint {
    type Small = u64;
    type Big = Self;
    type Unsigned = BigUint;
    type Static = Self;
    type Owned = Self;
    type Borrowed<'a>
        = &'a BigUint
    where
        Self: 'a;

    const ZERO: Self = BigUint::ZERO;

    fn from_small(s: u64) -> Self {
        s.into()
    }

    fn from_big(b: <u64 as SmallNumber>::Big) -> Self {
        b
    }

    fn from_big_ref(b: &'enc <u64 as SmallNumber>::Big) -> Self::Borrowed<'enc> {
        b
    }

    fn into_static(self) -> Self::Static {
        self
    }

    fn into_owned(self) -> Self::Owned {
        self
    }

    fn borrow<'a>(&'a self) -> Self::Borrowed<'a> {
        self
    }
}

impl<'enc> OwnedEncoding<'enc> for BigUint {
    fn decode_mut(&mut self) -> Decoded<u64, &mut Self> {
        Decoded::Big(self)
    }
}
