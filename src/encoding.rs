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
pub struct Encoded<S, T>(Encoding<S, T>);

/// The content of an `Encoded` value, which is either a small integer or a big
/// integer.  Typically `S` will be `SmallInt` or `SmallUint`, and `T` will be
/// `BigInt` or `BigUint`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Encoding<S, T> {
    Small(S),
    Big(T),
}

impl<S, T> Encoded<S, T>
where
    S: ConstZero + ConstOne,
{
    pub const ONE: Encoded<S, T> = Encoded::from_small(S::ONE);
    pub const ZERO: Encoded<S, T> = Encoded::from_small(S::ZERO);
}

impl<S, T> Encoded<S, T> {
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
            Encoded(Encoding::Big(x))
        }
    }

    /// Encodes a big integer as an `Encoded` value, using the small encoding if
    /// possible, and cloning the big integer if otherwise.
    pub fn from_big_ref<'a>(x: &'a T) -> Encoded<S, T>
    where
        for<'b> S: TryFrom<&'b T>,
        T: Clone,
    {
        if let Ok(s) = S::try_from(x) {
            Encoded(Encoding::Small(s))
        } else {
            Encoded(Encoding::Big(x.clone()))
        }
    }

    /// Encodes a big integer as an `Encoded` value, using the small encoding if
    /// possible, and borrowing the big integer otherwise.
    pub fn from_big_cow<'a>(x: Cow<'a, T>) -> Encoded<S, Cow<'a, T>>
    where
        for<'b> S: TryFrom<&'b T>,
        T: Clone,
    {
        if let Ok(s) = S::try_from(x.as_ref()) {
            Encoded(Encoding::Small(s))
        } else {
            Encoded(Encoding::Big(x))
        }
    }

    pub fn into_encoding(self) -> Encoding<S, T> {
        self.0
    }

    pub fn borrow_encoding(&self) -> &Encoding<S, T> {
        &self.0
    }

    pub fn update_encoding(&mut self, f: impl FnOnce(&mut Encoding<S, T>))
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

impl<S, T> From<S> for Encoded<S, T>
where
    for<'a> S: TryFrom<&'a T>,
{
    fn from(x: S) -> Self {
        Encoded::from_small(x)
    }
}

impl From<BigInt> for Encoded<SmallInt, BigInt> {
    fn from(x: BigInt) -> Self {
        Encoded::from_big(x)
    }
}

impl From<&BigInt> for Encoded<SmallInt, BigInt> {
    fn from(x: &BigInt) -> Self {
        Encoded::from_big_ref(x)
    }
}

impl From<Encoded<SmallInt, BigInt>> for BigInt {
    fn from(x: Encoded<SmallInt, BigInt>) -> Self {
        match x.0 {
            Encoding::Small(n) => n.into(),
            Encoding::Big(n) => n,
        }
    }
}

impl<'a> From<Encoded<SmallInt, BigInt>> for Encoded<SmallInt, Cow<'a, BigInt>> {
    fn from(value: Encoded<SmallInt, BigInt>) -> Self {
        Encoded(match value.0 {
            Encoding::Small(x) => Encoding::Small(x),
            Encoding::Big(x) => Encoding::Big(Cow::Owned(x)),
        })
    }
}

impl<'a> From<Encoded<SmallInt, &'a BigInt>> for Encoded<SmallInt, Cow<'a, BigInt>> {
    fn from(value: Encoded<SmallInt, &'a BigInt>) -> Self {
        Encoded(match value.0 {
            Encoding::Small(x) => Encoding::Small(x),
            Encoding::Big(x) => Encoding::Big(Cow::Borrowed(x)),
        })
    }
}

impl From<BigUint> for Encoded<SmallUint, BigUint> {
    fn from(x: BigUint) -> Self {
        Encoded::from_big(x)
    }
}

impl From<&BigUint> for Encoded<SmallUint, BigUint> {
    fn from(x: &BigUint) -> Self {
        Encoded::from_big_ref(x)
    }
}

impl From<Encoded<SmallUint, BigUint>> for BigUint {
    fn from(value: Encoded<SmallUint, BigUint>) -> Self {
        match value.0 {
            Encoding::Small(n) => n.into(),
            Encoding::Big(n) => n,
        }
    }
}

impl<'a> From<Encoded<SmallUint, BigUint>> for Encoded<SmallUint, Cow<'a, BigUint>> {
    fn from(value: Encoded<SmallUint, BigUint>) -> Self {
        Encoded(match value.0 {
            Encoding::Small(x) => Encoding::Small(x),
            Encoding::Big(x) => Encoding::Big(Cow::Owned(x)),
        })
    }
}

impl<'a> From<Encoded<SmallUint, &'a BigUint>> for Encoded<SmallUint, Cow<'a, BigUint>> {
    fn from(value: Encoded<SmallUint, &'a BigUint>) -> Self {
        Encoded(match value.0 {
            Encoding::Small(x) => Encoding::Small(x),
            Encoding::Big(x) => Encoding::Big(Cow::Borrowed(x)),
        })
    }
}

pub trait ToCow<'a, T: Clone> {
    fn to_cow(self) -> Cow<'a, T>;
}

impl<'a> ToCow<'a, BigInt> for CBigInt {
    fn to_cow(self) -> Cow<'a, BigInt> {
        Cow::Owned(BigInt::from(self))
    }
}

impl<'a> ToCow<'a, BigInt> for &'a CBigInt {
    fn to_cow(self) -> Cow<'a, BigInt> {
        self.to_bigint_cow()
    }
}

impl<'a> ToCow<'a, BigInt> for BigInt {
    fn to_cow(self) -> Cow<'a, BigInt> {
        Cow::Owned(self)
    }
}

impl<'a> ToCow<'a, BigInt> for &'a BigInt {
    fn to_cow(self) -> Cow<'a, BigInt> {
        Cow::Borrowed(self)
    }
}

impl<'a> ToCow<'a, BigInt> for Encoded<SmallInt, Cow<'a, BigInt>> {
    fn to_cow(self) -> Cow<'a, BigInt> {
        match self.into_encoding() {
            Encoding::Small(n) => Cow::Owned(n.into()),
            Encoding::Big(cow) => cow,
        }
    }
}

// pub trait ToEncodedCow<'a, T: Clone> {
//     fn to_encoded_cow(self) -> Encoded<SmallInt, Cow<'a, T>>;
// }

// impl<'a> ToEncodedCow<'a, BigInt> for CBigInt {
//     fn to_encoded_cow(self) -> Encoded<SmallInt, Cow<'a, BigInt>> {
//         match self.0.into_encoding() {
//             Encoding::Small(n) => Encoded::from_small(n),
//             Encoding::Big(n) => Encoded(Encoding::Big(Cow::Owned(n))),
//         }
//     }
// }

// impl<'a> ToEncodedCow<'a, BigInt> for &'a CBigInt {
//     fn to_encoded_cow(self) -> Encoded<SmallInt, Cow<'a, BigInt>> {
//         match self.0.borrow_encoding() {
//             Encoding::Small(n) => Encoded::from_small(*n),
//             Encoding::Big(n) => Encoded::from_big_cow(Cow::Borrowed(n)),
//         }
//     }
// }

pub trait ToEncodingCow<'a, T: Clone> {
    fn to_encoding_cow(self) -> Encoding<SmallInt, Cow<'a, T>>;
}

impl<'a> ToEncodingCow<'a, BigInt> for CBigInt {
    fn to_encoding_cow(self) -> Encoding<SmallInt, Cow<'a, BigInt>> {
        match self.0.into_encoding() {
            Encoding::Small(n) => Encoding::Small(n),
            Encoding::Big(n) => Encoding::Big(Cow::Owned(n)),
        }
    }
}

impl<'a> ToEncodingCow<'a, BigInt> for &'a CBigInt {
    fn to_encoding_cow(self) -> Encoding<SmallInt, Cow<'a, BigInt>> {
        match self.0.borrow_encoding() {
            Encoding::Small(n) => Encoding::Small(*n),
            Encoding::Big(n) => Encoding::Big(Cow::Borrowed(n)),
        }
    }
}

duplicate_prims! {
    impl<'a> ToEncodingCow<'a, BigInt> for prim {
        fn to_encoding_cow(self) -> Encoding<SmallInt, Cow<'a, BigInt>> {
            #[allow(irrefutable_let_patterns)]
            if let Ok(small) = SmallInt::try_from(self) {
                Encoding::Small(small)
            } else {
                Encoding::Big(Cow::Owned(BigInt::from(self)))
            }
        }
    }

    impl<'a> ToEncodingCow<'a, BigInt> for &'a prim {
        fn to_encoding_cow(self) -> Encoding<SmallInt, Cow<'a, BigInt>> {
            (*self).to_encoding_cow()
        }
    }
}
