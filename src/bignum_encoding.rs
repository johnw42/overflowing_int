use crate::{
    cow_encoding::CowEncoding,
    encoding::{Decode, Decoded, Encoding, OwnedEncoding},
    small_num::SmallNumber,
};
use num_bigint::{BigInt, BigUint};
use std::borrow::Cow;

//
// BigInt
//

impl<'enc> Decode<'enc, i128> for BigInt {
    fn decode<'a>(&'a self) -> Decoded<i128, Cow<'a, <i128 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Borrowed(self))
    }

    fn into_decoded(self) -> Decoded<i128, Cow<'enc, <i128 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Owned(self))
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

    fn from_small(s: i128) -> Self {
        s.into()
    }

    fn from_big(b: <i128 as SmallNumber>::Big) -> Self {
        b
    }

    fn from_big_ref(b: &'enc <i128 as SmallNumber>::Big) -> Self {
        b.clone()
    }

    fn into_owned(self) -> Self::Owned {
        self
    }

    fn borrow<'a>(&'a self) -> Self::Borrowed<'a> {
        CowEncoding::from_big_ref(self)
    }
}

impl<'enc> OwnedEncoding<'enc> for BigInt {
    fn decode_mut(&mut self) -> Decoded<i128, &mut Self> {
        Decoded::Big(self)
    }
}

impl<'enc> Encoding<'enc> for &'enc BigInt {
    type Small = <BigInt as Encoding<'enc>>::Small;
    type Big = <BigInt as Encoding<'enc>>::Big;
    type Unsigned = <BigInt as Encoding<'enc>>::Unsigned;

    type Owned = BigInt;

    type Borrowed<'a>
        = &'a BigInt
    where
        Self: 'a;

    const ZERO: Self = &BigInt::ZERO;

    fn from_small(s: Self::Small) -> Self::Owned {
        Self::Owned::from_small(s)
    }

    fn from_big(b: Self::Big) -> Self::Owned {
        Self::Owned::from_big(b)
    }

    fn from_big_ref(b: &'enc Self::Big) -> Self {
        b
    }

    fn into_owned(self) -> Self::Owned {
        (*self).clone()
    }

    fn borrow<'a>(&'a self) -> Self::Borrowed<'a> {
        self
    }
}

// &BigInt

// impl<'enc> Decode<'enc, i128> for &'enc BigInt {
//     fn decode<'a>(&'a self) -> Decoded<i128, Cow<'a, <i128 as SmallNumber>::Big>> {
//         Decoded::Big(Cow::Borrowed(self))
//     }

//     fn into_decoded(self) -> Decoded<i128, Cow<'static, <i128 as SmallNumber>::Big>> {
//         Decoded::Big(Cow::Owned(self.clone()))
//     }
// }

//
// BigUint
//

impl<'enc> Decode<'enc, u128> for BigUint {
    fn decode<'a>(&'a self) -> Decoded<u128, Cow<'a, <u128 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Borrowed(self))
    }

    fn into_decoded(self) -> Decoded<u128, Cow<'enc, <u128 as SmallNumber>::Big>> {
        Decoded::Big(Cow::Owned(self))
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

    fn from_small(s: u128) -> Self {
        s.into()
    }

    fn from_big(b: <u128 as SmallNumber>::Big) -> Self {
        b
    }

    fn from_big_ref(b: &'enc <u128 as SmallNumber>::Big) -> Self {
        b.clone()
    }

    fn into_owned(self) -> Self::Owned {
        self
    }

    fn borrow<'a>(&'a self) -> Self::Borrowed<'a> {
        CowEncoding::from_big_ref(self)
    }
}

impl<'enc> OwnedEncoding<'enc> for BigUint {
    fn decode_mut(&mut self) -> Decoded<u128, &mut Self> {
        Decoded::Big(self)
    }
}

impl<'enc> Encoding<'enc> for &'enc BigUint {
    type Small = <BigUint as Encoding<'enc>>::Small;
    type Big = <BigUint as Encoding<'enc>>::Big;
    type Unsigned = Self;

    type Owned = BigUint;

    type Borrowed<'a>
        = &'a BigUint
    where
        Self: 'a;

    const ZERO: Self = &BigUint::ZERO;

    fn from_small(s: Self::Small) -> Self::Owned {
        Self::Owned::from_small(s)
    }

    fn from_big(b: Self::Big) -> Self::Owned {
        Self::Owned::from_big(b)
    }

    fn from_big_ref(b: &'enc Self::Big) -> Self {
        b
    }

    fn into_owned(self) -> Self::Owned {
        (*self).clone()
    }

    fn borrow<'a>(&'a self) -> Self::Borrowed<'a> {
        self
    }
}

// &BigUint

// impl<'enc> Decode<'enc, u128> for &'enc BigUint {
//     fn decode<'a>(&'a self) -> Decoded<u128, Cow<'a, <u128 as SmallNumber>::Big>> {
//         Decoded::Big(Cow::Borrowed(self))
//     }

//     fn into_decoded(self) -> Decoded<u128, Cow<'static, <u128 as SmallNumber>::Big>> {
//         Decoded::Big(Cow::Owned(self.clone()))
//     }
// }
