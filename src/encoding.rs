use std::borrow::Cow;
use std::fmt::Debug;

use num_bigint::BigInt;
use num_bigint::BigUint;
use num_traits::ConstOne;
use num_traits::ConstZero;

use duplicate::duplicate;

use crate::CBigInt;
use crate::{SmallInt, SmallUint};
use crate::{duplicate_prims, duplicate_uprims};

/// A wrapper type around `Encoding` that maintains the the invariant that
/// values that can be represented as `SmallInt` or `SmallUint` are always
/// stored as such, and only values that cannot be represented as `SmallInt` or
/// `SmallUint` are stored as `BigInt` or `BigUint`.  This type, in turn, is the
/// content of `CBigInt` and `CBigUint`, which implement high-level operations
/// and traits.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Encoded<'a, S, T: Clone>(Encoding<'a, S, T>);

/// The content of an `Encoded` value, which is either a small integer or a big
/// integer.  Typically `S` will be `SmallInt` or `SmallUint`, and `T` will be
/// `BigInt` or `BigUint`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Encoding<'a, S, T: Clone> {
    Small(S),
    Big(Cow<'a, T>),
}

impl<S, T> Encoded<'static, S, T>
where
    S: ConstZero + ConstOne,
    T: Clone,
{
    pub const ONE: Encoded<'static, S, T> = Encoded::from_small(S::ONE);
    pub const ZERO: Encoded<'static, S, T> = Encoded::from_small(S::ZERO);
}

impl<'a, S, T: Clone> Encoded<'a, S, T> {
    /// Encodes a small integer as an `Encoded` value, using the small encoding.
    pub const fn from_small(x: S) -> Self {
        Encoded(Encoding::Small(x))
    }

    /// Encodes a big integer as an `Encoded` value, using the small encoding if possible.
    pub fn from_big(x: T) -> Self
    where
        for<'b> S: TryFrom<&'b T>,
    {
        if let Ok(s) = S::try_from(&x) {
            Encoded(Encoding::Small(s))
        } else {
            Encoded(Encoding::Big(Cow::Owned(x)))
        }
    }

    /// Encodes a big integer as an `Encoded` value, using the small encoding if possible.
    pub fn from_big_cow(x: Cow<'a, T>) -> Self
    where
        for<'b> S: TryFrom<&'b T>,
    {
        if let Ok(s) = S::try_from(&x) {
            Encoded(Encoding::Small(s))
        } else {
            Encoded(Encoding::Big(x))
        }
    }

    pub fn borrow_encoding(&self) -> &Encoding<'a, S, T> {
        &self.0
    }

    pub fn update_encoding(&mut self, f: impl FnOnce(&mut Encoding<'a, S, T>))
    where
        for<'b> S: TryFrom<&'b T>,
    {
        f(&mut self.0);
        let mut small_value = None;
        if let Encoding::Big(big) = &mut self.0 {
            small_value = S::try_from(big).ok();
        }
        if let Some(small) = small_value {
            self.0 = Encoding::Small(small);
        }
    }
}

impl<S, T> From<S> for Encoded<'static, S, T>
where
    for<'a> S: TryFrom<&'a T>,
    T: Clone,
{
    fn from(x: S) -> Self {
        Encoded::from_small(x)
    }
}

impl From<BigInt> for Encoded<'static, SmallInt, BigInt> {
    fn from(x: BigInt) -> Self {
        Encoded::from_big(x)
    }
}

impl<'a> From<&'a BigInt> for Encoded<'a, SmallInt, BigInt> {
    fn from(x: &'a BigInt) -> Self {
        Encoded::from_big_cow(Cow::Borrowed(x))
    }
}

impl From<Encoded<'static, SmallInt, BigInt>> for BigInt {
    fn from(x: Encoded<'static, SmallInt, BigInt>) -> Self {
        match x.0 {
            Encoding::Small(n) => n.into(),
            Encoding::Big(n) => n.into_owned(),
        }
    }
}

impl From<BigUint> for Encoded<'static, SmallUint, BigUint> {
    fn from(x: BigUint) -> Self {
        Encoded::from_big(x)
    }
}

impl<'a> From<&'a BigUint> for Encoded<'a, SmallUint, BigUint> {
    fn from(x: &'a BigUint) -> Self {
        Encoded::from_big_cow(Cow::Borrowed(x))
    }
}

impl<'a> From<Encoded<'a, SmallUint, BigUint>> for BigUint {
    fn from(value: Encoded<'a, SmallUint, BigUint>) -> Self {
        match value.0 {
            Encoding::Small(n) => n.into(),
            Encoding::Big(n) => n.into_owned(),
        }
    }
}

pub trait ToBigIntCow<'a> {
    fn to_cow(self) -> Cow<'a, BigInt>;
}

impl<'a> ToBigIntCow<'a> for CBigInt<'a> {
    fn to_cow(self) -> Cow<'a, BigInt> {
        match self.into_encoding() {
            Encoding::Small(n) => Cow::Owned(n.into()),
            Encoding::Big(n) => n,
        }
    }
}

impl<'a> ToBigIntCow<'a> for &'a CBigInt<'a> {
    fn to_cow(self) -> Cow<'a, BigInt> {
        match self.encoding() {
            Encoding::Small(n) => Cow::Owned((*n).into()),
            Encoding::Big(n) => n.clone(),
        }
    }
}

impl<'a> ToBigIntCow<'a> for BigInt {
    fn to_cow(self) -> Cow<'a, BigInt> {
        Cow::Owned(self)
    }
}

impl<'a> ToBigIntCow<'a> for &'a BigInt {
    fn to_cow(self) -> Cow<'a, BigInt> {
        Cow::Borrowed(self)
    }
}

// TODO
// impl<'a> ToCow<'a, BigInt> for Encoded<SmallInt, Cow<'a, BigInt>> {
//     fn to_cow(self) -> Cow<'a, BigInt> {
//         match self.into_encoding() {
//             Encoding::Small(n) => Cow::Owned(n.into()),
//             Encoding::Big(cow) => cow,
//         }
//     }
// }

pub trait IntoEncoding<'a, S, T: Clone> {
    fn into_encoding(self) -> Encoding<'a, S, T>;
}

impl<'a> IntoEncoding<'a, SmallInt, BigInt> for CBigInt<'a> {
    fn into_encoding(self) -> Encoding<'a, SmallInt, BigInt> {
        self.0.0
    }
}

impl<'a> IntoEncoding<'a, SmallInt, BigInt> for &CBigInt<'a> {
    fn into_encoding(self) -> Encoding<'a, SmallInt, BigInt> {
        self.0.0.clone()
    }
}

duplicate_prims! {
    impl<'a> IntoEncoding<'a, SmallInt, BigInt> for prim {
        fn into_encoding(self) -> Encoding<'a, SmallInt, BigInt> {
            #[allow(irrefutable_let_patterns)]
            #[allow(clippy::unnecessary_fallible_conversions)]
            if let Ok(small) = SmallInt::try_from(self) {
                Encoding::Small(small)
            } else {
                Encoding::Big(Cow::Owned(BigInt::from(self)))
            }
        }
    }

    impl<'a> IntoEncoding<'a, SmallInt, BigInt> for &prim {
        fn into_encoding(self) -> Encoding<'a, SmallInt, BigInt> {
            (*self).into_encoding()
        }
    }
}

#[allow(clippy::wrong_self_convention)]
pub trait IntoEncodingRef<'a, S, T: Clone> {
    fn into_encoding_ref(&self) -> &Encoding<'a, S, T>;
}

impl<'a> IntoEncodingRef<'a, SmallInt, BigInt> for CBigInt<'a> {
    fn into_encoding_ref(&self) -> &Encoding<'a, SmallInt, BigInt> {
        &self.0.0
    }
}

impl<'a> IntoEncodingRef<'a, SmallInt, BigInt> for &CBigInt<'a> {
    fn into_encoding_ref(&self) -> &Encoding<'a, SmallInt, BigInt> {
        &self.0.0
    }
}
