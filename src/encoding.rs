use std::borrow::Cow;
use std::fmt::Debug;

use num_bigint::BigInt;
use num_bigint::BigUint;

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

impl<'a, S, T: Clone> Encoded<'a, S, T> {
    pub const fn from_small(small: S) -> Self {
        Encoded(Encoding::Small(small))
    }

    /// Creates an encoding from an owned value, using the small encoding if possible.
    pub fn from_big(x: T) -> Encoded<'static, S, T>
    where
        for<'b> S: TryFrom<&'b T>,
    {
        Encoded(if let Ok(s) = S::try_from(&x) {
            Encoding::Small(s)
        } else {
            Encoding::Big(Cow::Owned(x))
        })
    }

    /// Creates an encoding from a `Cow`, using the small encoding if possible.
    pub fn from_big_cow(x: Cow<'a, T>) -> Self
    where
        for<'b> S: TryFrom<&'b T>,
    {
        if let Ok(s) = S::try_from(&x) {
            Encoding::Small(s)
        } else {
            Encoding::Big(x)
        }
        .into()
    }

    pub fn encoding(&self) -> &Encoding<'a, S, T> {
        &self.0
    }

    pub fn into_encoding(self) -> Encoding<'a, S, T> {
        self.0
    }

    pub fn update_encoding(&mut self, f: impl FnOnce(&mut Encoding<'a, S, T>))
    where
        for<'b> S: TryFrom<&'b T>,
    {
        f(&mut self.0);
        self.0.normalize();
    }
}

/// The content of an `Encoded` value, which is either a small integer or a big
/// integer.  Typically `S` will be `SmallInt` or `SmallUint`, and `T` will be
/// `BigInt` or `BigUint`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Encoding<'a, S, T: Clone> {
    Small(S),
    Big(Cow<'a, T>),
}

impl<'a, S, T: Clone> Encoding<'a, S, T> {
    pub fn into_static(self) -> Encoding<'static, S, T>
    where
        for<'b> S: TryFrom<&'b T>,
    {
        match self {
            Encoding::Small(s) => Encoding::Small(s),
            Encoding::Big(b) => Encoding::Big(Cow::Owned(b.into_owned())),
        }
    }

    pub fn normalize(&mut self)
    where
        for<'b> S: TryFrom<&'b T>,
    {
        let mut small_value = None;
        if let Encoding::Big(big) = &self {
            small_value = S::try_from(big).ok();
        }
        if let Some(small) = small_value {
            *self = Encoding::Small(small);
        }
    }
}

impl<'a, S, T: Clone> From<Encoding<'a, S, T>> for Encoded<'a, S, T> {
    fn from(x: Encoding<'a, S, T>) -> Self {
        Encoded(x)
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
        match x.into_encoding() {
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
        match value.into_encoding() {
            Encoding::Small(n) => n.into(),
            Encoding::Big(n) => n.into_owned().into(),
        }
    }
}

pub trait IntoBigIntCow<'a> {
    fn into_bigint_cow(self) -> Cow<'a, BigInt>;
}

impl<'a> IntoBigIntCow<'a> for CBigInt<'a> {
    fn into_bigint_cow(self) -> Cow<'a, BigInt> {
        match self.into_encoding() {
            Encoding::Small(n) => Cow::Owned(n.into()),
            Encoding::Big(n) => n,
        }
    }
}

impl<'a> IntoBigIntCow<'a> for &CBigInt<'a> {
    fn into_bigint_cow(self) -> Cow<'a, BigInt> {
        match self.encoding() {
            Encoding::Small(n) => Cow::Owned((*n).into()),
            Encoding::Big(n) => n.clone(),
        }
    }
}

impl<'a> IntoBigIntCow<'a> for BigInt {
    fn into_bigint_cow(self) -> Cow<'a, BigInt> {
        Cow::Owned(self)
    }
}

impl<'a> IntoBigIntCow<'a> for &'a BigInt {
    fn into_bigint_cow(self) -> Cow<'a, BigInt> {
        Cow::Borrowed(self)
    }
}

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
