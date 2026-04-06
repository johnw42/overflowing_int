use std::{borrow::Cow, hash::Hash, rc::Rc};

use num_bigint::BigInt;

use crate::small_num::SmallNumber;
use crate::{big_number::BigNumber, duplicate_prims};

/// A decoded big number, which may be either small or big.  Also used to
/// represent various other decoded values, such as the pairs of numbers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Decoded<S, B> {
    Small(S),
    Big(B),
}

/// A type that can be decoded into a small or big value.  This this applies to
/// encoded big numbers, but it also applies to types that can be trivially
/// decoded, such as integer types and bignum types.
pub trait Decode<'a, S>: Sized + Clone
where
    S: SmallNumber,
{
    /// The main method of this trait, which decodes the value into either a
    /// small or big value without ever cloning it.
    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T;

    /// Decodes the value.  This method may be less efficient than
    /// [`Self::with_decoded`] since it may require cloning the big value, but
    /// it is more convenient when an owned value is needed.
    fn decode(self) -> Decoded<S, Cow<'a, S::Big>>;

    /// Gets the big value as a `Cow`, creating a bignum if necessary.
    fn into_big_cow(self) -> Cow<'a, S::Big> {
        match self.decode() {
            Decoded::Small(s) => Cow::Owned(s.to_big()),
            Decoded::Big(b) => b,
        }
    }

    /// Gets the small value if it the encoding is small, or `None` if it is big.
    fn small(&self) -> Option<S>;

    /// Gets the big value  This will always create or clone a bignum.
    fn big(&self) -> S::Big {
        match self.clone().decode() {
            Decoded::Small(s) => s.to_big(),
            Decoded::Big(b) => b.into_owned(),
        }
    }

    /// Gets the big value as an owned value while consuming self.  Compared to
    /// [`Self::big`], using this method may allow for more efficient code when
    /// the encoding is big, since it can avoid an extra clone of the big value.
    fn into_big(self) -> S::Big {
        match self.decode() {
            Decoded::Small(s) => s.to_big(),
            Decoded::Big(b) => b.into_owned(),
        }
    }

    /// Gets the big value as a `Cow`, creating a bignum if necessary, and passes it to the provided function.
    fn with_big_cow<T>(&self, f: impl FnOnce(Cow<S::Big>) -> T) -> T {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => f(Cow::Owned(s.to_big())),
            Decoded::Big(b) => f(b),
        })
    }

    /// A helper method for working with two decoded values at the same time.
    /// This calls [`Self::with_big_cow`] for both `self` and `other`, and
    /// passes the resulting big `Cow`s to the provided function.  This is useful
    /// for implementing binary operations on big numbers, since it allows us to
    /// avoid cloning the big values when both encodings are big.
    fn with_big_cows<'b, T>(
        &self,
        other: &impl Decode<'b, S>,
        f: impl FnOnce(Cow<S::Big>, Cow<S::Big>) -> T,
    ) -> T {
        self.with_big_cow(|lhs| other.with_big_cow(|rhs| f(lhs, rhs)))
    }

    /// A helper method for working with two decoded values at the same time,
    /// where the values need to both be small or both be big.  This calls
    /// [`Self::with_decoded`] for both `self` and `other`, and passes the
    /// resulting decoded values to the provided function.  This is useful for
    /// implementing binary operations on big numbers, since it allows us to
    /// avoid cloning the big values when both encodings are big, while still
    /// allowing us to work with the small values when both encodings are small.
    fn with_matching_size<'b, T>(
        &self,
        other: &impl Decode<'b, S>,
        f: impl FnOnce(Decoded<(S, S), (Cow<S::Big>, Cow<S::Big>)>) -> T,
    ) -> T {
        self.with_decoded(|lhs| {
            other.with_decoded(|rhs| {
                let enc = match (lhs, rhs) {
                    (Decoded::Small(s1), Decoded::Small(s2)) => Decoded::Small((s1, s2)),
                    (Decoded::Small(s1), Decoded::Big(b2)) => {
                        Decoded::Big((Cow::Owned(s1.to_big()), b2))
                    }
                    (Decoded::Big(b1), Decoded::Small(s2)) => {
                        Decoded::Big((b1, Cow::Owned(s2.to_big())))
                    }
                    (Decoded::Big(b1), Decoded::Big(b2)) => Decoded::Big((b1, b2)),
                };
                f(enc)
            })
        })
    }
}

/// An encoding of a big number, where small values are encoded directly in the
/// representation, and big values are encoded as a separate big type.  The
/// encoding must be able to be updated in place, and must be able to be
/// compared for equality and hashed without decoding.
pub trait Encoding<'a>: Decode<'a, Self::Small>
where
    Self: Eq,
    Self: Hash,
    Self::Big: Into<BigInt>,
{
    /// The small type that can be encoded directly in the representation.
    type Small: SmallNumber<Big = Self::Big>;

    /// The big type that is used when the value is too large to fit in the
    /// small representation.  This type is completely determined by the small
    /// type, but having it is a convenience for writing code that is generic
    /// over the encoding.
    type Big: BigNumber;

    /// Encodes a small value.
    fn from_small(s: Self::Small) -> Self;

    /// Encodes an owned big value.
    fn from_big(b: Self::Big) -> Self {
        Self::from_big_cow(Cow::Owned(b))
    }

    /// Encodes a big value from a `Cow`.
    fn from_big_cow(b: Cow<'a, Self::Big>) -> Self;

    /// Encodes a value.  Prefer [`Self::from_small`] or [`Self::from_big`] when possible.
    fn from_decoded(enc: Decoded<Self::Small, Cow<'a, Self::Big>>) -> Self {
        match enc {
            Decoded::Small(s) => Self::from_small(s),
            Decoded::Big(b) => Self::from_big_cow(b),
        }
    }

    /// Updates the encoding in place using the provided function.
    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<Self::Small, Cow<Self::Big>>));
}

// =============================================================================
// Implementations of `Decode` for various foreign and built-in types
// =============================================================================

impl<'a, S> Decode<'a, S> for Cow<'a, S::Big>
where
    S: SmallNumber,
{
    fn decode(self) -> Decoded<S, Cow<'a, S::Big>> {
        Decoded::Big(self)
    }

    fn small(&self) -> Option<S> {
        None
    }

    fn into_big_cow(self) -> Cow<'a, S::Big> {
        self
    }

    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T {
        f(Decoded::Big(Cow::Borrowed(self.as_ref())))
    }
}

impl<'a, S> Decode<'a, S> for Rc<S::Big>
where
    S: SmallNumber,
{
    fn decode(self) -> Decoded<S, Cow<'a, S::Big>> {
        Decoded::Big(Cow::Owned((*self).clone()))
    }

    fn small(&self) -> Option<S> {
        None
    }

    fn into_big_cow(self) -> Cow<'a, S::Big> {
        match Rc::try_unwrap(self) {
            Ok(b) => Cow::Owned(b),
            Err(rc) => Cow::Owned((*rc).clone()),
        }
    }

    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T {
        f(Decoded::Big(Cow::Borrowed(self.as_ref())))
    }
}

duplicate_prims! {
    impl<'a, S> Decode<'a, S> for prim
    where
        S: SmallNumber,
        S::Big: BigNumber + From<prim>
    {
        fn decode(self) -> Decoded<S, Cow<'a, S::Big>> {
            #[allow(irrefutable_let_patterns)]
            #[allow(clippy::unnecessary_fallible_conversions)]
            if let Ok(small) = S::try_from(self) {
                Decoded::Small(small)
            } else {
                Decoded::Big(Cow::Owned(S::Big::from(self)))
            }
        }

        fn small(&self) -> Option<S> {
            S::try_from(*self).ok()
        }

        fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T {
            match S::try_from(*self) {
                Ok(small) => f(Decoded::Small(small)),
                Err(_) => f(Decoded::Big(Cow::Owned(S::Big::from(*self)))),
            }
        }
    }

    impl<'a, S: SmallNumber> Decode<'a, S> for &prim
    where
        S::Big: From<prim>,
        S: TryFrom<prim>
    {
        fn decode(self) -> Decoded<S, Cow<'a, S::Big>> {
            #[allow(irrefutable_let_patterns)]
            #[allow(clippy::unnecessary_fallible_conversions)]
            if let Ok(small) = S::try_from(*self) {
                Decoded::Small(small)
            } else {
                Decoded::Big(Cow::Owned(S::Big::from(*self)))
            }
        }

        fn small(&self) -> Option<S> {
            S::try_from(**self).ok()
        }

        fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T {
            match S::try_from(**self) {
                Ok(small) => f(Decoded::Small(small)),
                Err(_) => f(Decoded::Big(Cow::Owned(S::Big::from(**self)))),
            }
        }
    }
}
