use crate::{
    cow_encoding::CowEncoding,
    encoding::{Decode, Decoded, Encode, Encoding, EncodingMut},
    small_num::SmallNumber,
};
use num_bigint::{BigInt, BigUint};
use std::borrow::Cow;

pub trait BorrowBigNum {
    fn borrow(&self) -> &Self;
    fn into_owned(self) -> Self;
}

//
// BigInt
//

impl<'enc> BorrowBigNum for BigInt {
    fn borrow(&self) -> &Self {
        self
    }

    fn into_owned(self) -> Self {
        self
    }
}

impl<'enc> Decode<'enc, i128> for BigInt {
    fn decode<'a>(&'a self) -> Decoded<i128, Cow<'a, <i128 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Borrowed(self))
    }

    fn into_decoded(self) -> Decoded<i128, Cow<'enc, <i128 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Owned(self))
    }
}

impl<'enc> Encode<'enc, i128> for BigInt {
    fn from_small(s: i128) -> Self {
        s.into()
    }

    fn from_big(b: <i128 as SmallNumber>::Big) -> Self {
        b
    }

    fn from_big_ref(b: &'enc <i128 as SmallNumber>::Big) -> Self {
        b.clone()
    }
}

impl<'enc> Encoding<'enc> for BigInt {
    type Small = i128;
    type Big = Self;
    type Unsigned = BigUint;
    type Owned = Self;
    type Borrowed<'a>
        = CowEncoding<'a, i128>
    where
        Self: 'a;

    const ZERO: Self = BigInt::ZERO;

    fn into_owned(self) -> Self::Owned {
        self
    }

    fn borrow<'a>(&'a self) -> Self::Borrowed<'a> {
        CowEncoding::from_big_ref(self)
    }
}

impl<'enc> EncodingMut<'enc> for BigInt {
    fn update_encoding(
        &mut self,
        f: impl FnOnce(Decoded<Self::Small, &mut Self::Big>) -> Option<Decoded<Self::Small, Self::Big>>,
    ) {
        let new_decoded = f(Decoded::Big(self));
        match new_decoded {
            Some(Decoded::Small(s)) => *self = s.into(),
            Some(Decoded::Big(b)) => *self = b,
            _ => (),
        }
    }
}

// &BigInt

impl<'enc> Decode<'enc, i128> for &'enc BigInt {
    fn decode<'a>(&'a self) -> Decoded<i128, Cow<'a, <i128 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Borrowed(self))
    }

    fn into_decoded(self) -> Decoded<i128, Cow<'static, <i128 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Owned(self.clone()))
    }
}

//
// BigUint
//

impl BorrowBigNum for BigUint {
    fn borrow(&self) -> &Self {
        self
    }

    fn into_owned(self) -> Self {
        self
    }
}

impl<'enc> Decode<'enc, u128> for BigUint {
    fn decode<'a>(&'a self) -> Decoded<u128, Cow<'a, <u128 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Borrowed(self))
    }

    fn into_decoded(self) -> Decoded<u128, Cow<'enc, <u128 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Owned(self))
    }
}

impl<'enc> Encode<'enc, u128> for BigUint {
    fn from_small(s: u128) -> Self {
        s.into()
    }

    fn from_big(b: <u128 as SmallNumber>::Big) -> Self {
        b
    }

    fn from_big_ref(b: &'enc <u128 as SmallNumber>::Big) -> Self {
        b.clone()
    }
}

impl<'enc> Encoding<'enc> for BigUint {
    type Small = u128;
    type Big = Self;
    type Unsigned = BigUint;
    type Owned = Self;
    type Borrowed<'a>
        = CowEncoding<'a, u128>
    where
        Self: 'a;

    const ZERO: Self = BigUint::ZERO;

    fn into_owned(self) -> Self::Owned {
        self
    }

    fn borrow<'a>(&'a self) -> Self::Borrowed<'a> {
        CowEncoding::from_big_ref(self)
    }
}

impl<'enc> EncodingMut<'enc> for BigUint {
    fn update_encoding(
        &mut self,
        f: impl FnOnce(Decoded<Self::Small, &mut Self::Big>) -> Option<Decoded<Self::Small, Self::Big>>,
    ) {
        let new_decoded = f(Decoded::Big(self));
        match new_decoded {
            Some(Decoded::Small(s)) => *self = s.into(),
            Some(Decoded::Big(b)) => *self = b,
            _ => (),
        }
    }
}

// &BigUint

impl<'enc> Decode<'enc, u128> for &'enc BigUint {
    fn decode<'a>(&'a self) -> Decoded<u128, Cow<'a, <u128 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Borrowed(self))
    }

    fn into_decoded(self) -> Decoded<u128, Cow<'static, <u128 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Owned(self.clone()))
    }
}
