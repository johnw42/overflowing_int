use std::{
    borrow::{Borrow, Cow},
    hash::Hash,
    rc::Rc,
};

use num_bigint::BigInt;

use crate::{big_number::BigNumber, duplicate_prims};
use crate::{generic_bignum::GenericBigNum, small_num::SmallNumber};

pub trait InspectEncoding<'a, S, B>: Sized + Clone
where
    S: SmallNumber<Big = B>,
    B: BigNumber,
{
    /// Decodes the value, preferably without allocating.
    fn decode(self) -> Decoded<S, Cow<'a, B>>;

    /// Gets the big value as a `Cow`, creating a bignum if necessary.
    fn into_big_cow(self) -> Cow<'a, B> {
        match self.decode() {
            Decoded::Small(s) => Cow::Owned(s.to_big()),
            Decoded::Big(b) => b,
        }
    }

    /// Gets the small value if it the encoding is small, or `None` if it is big.
    fn small(&self) -> Option<S>;

    // Gets a big value, which will always create or clone a bignum.
    fn big(&self) -> B {
        match self.clone().decode() {
            Decoded::Small(s) => s.to_big(),
            Decoded::Big(b) => b.into_owned(),
        }
    }

    fn into_big(self) -> B {
        match self.decode() {
            Decoded::Small(s) => s.to_big(),
            Decoded::Big(b) => b.into_owned(),
        }
    }

    fn with_decoded_ref<T>(&self, f: impl FnOnce(Decoded<S, Cow<B>>) -> T) -> T;

    fn with_big_ref<T>(&self, f: impl FnOnce(Cow<B>) -> T) -> T {
        self.with_decoded_ref(|decoded| match decoded {
            Decoded::Small(s) => f(Cow::Owned(s.to_big())),
            Decoded::Big(b) => f(b),
        })
    }

    fn with_big_refs<'b, T>(
        &self,
        other: &impl InspectEncoding<'b, S, B>,
        f: impl FnOnce(Cow<B>, Cow<B>) -> T,
    ) -> T {
        self.with_decoded_ref(|lhs| {
            other.with_decoded_ref(|rhs| {
                let (lhs, rhs) = match (lhs, rhs) {
                    (Decoded::Small(s1), Decoded::Small(s2)) => {
                        (Cow::Owned(s1.to_big()), Cow::Owned(s2.to_big()))
                    }
                    (Decoded::Small(s1), Decoded::Big(b2)) => (Cow::Owned(s1.to_big()), b2),
                    (Decoded::Big(b1), Decoded::Small(s2)) => (b1, Cow::Owned(s2.to_big())),
                    (Decoded::Big(b1), Decoded::Big(b2)) => (b1, b2),
                };
                f(lhs, rhs)
            })
        })
    }

    fn with_matching_refs<'b, T>(
        &self,
        other: &impl InspectEncoding<'b, S, B>,
        f: impl FnOnce(Decoded<(S, S), (Cow<B>, Cow<B>)>) -> T,
    ) -> T {
        self.with_decoded_ref(|lhs| {
            other.with_decoded_ref(|rhs| {
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

pub trait EncodedBigNum<'a>: InspectEncoding<'a, Self::Small, Self::Big>
where
    Self: Eq,
    Self: Hash,
    Self::Big: Into<BigInt>,
{
    type Small: SmallNumber<Big = Self::Big>;
    type Big: BigNumber;
    //type Repr: EncodedRepr<'a, Self::Small, Self::Big>;

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

impl<'a, S, B> InspectEncoding<'a, S, B> for Cow<'a, B>
where
    S: SmallNumber<Big = B>,
    B: BigNumber,
{
    fn decode(self) -> Decoded<S, Cow<'a, B>> {
        Decoded::Big(self)
    }

    fn small(&self) -> Option<S> {
        None
    }

    fn into_big_cow(self) -> Cow<'a, B> {
        self
    }

    fn with_decoded_ref<T>(&self, f: impl FnOnce(Decoded<S, Cow<B>>) -> T) -> T {
        f(Decoded::Big(Cow::Borrowed(self.as_ref())))
    }
}

impl<'a, S, B> InspectEncoding<'a, S, B> for Rc<B>
where
    S: SmallNumber<Big = B>,
    B: BigNumber,
{
    fn decode(self) -> Decoded<S, Cow<'a, B>> {
        Decoded::Big(Cow::Owned((*self).clone()))
    }

    fn small(&self) -> Option<S> {
        None
    }

    fn into_big_cow(self) -> Cow<'a, B> {
        match Rc::try_unwrap(self) {
            Ok(b) => Cow::Owned(b),
            Err(rc) => Cow::Owned((*rc).clone()),
        }
    }

    fn with_decoded_ref<T>(&self, f: impl FnOnce(Decoded<S, Cow<B>>) -> T) -> T {
        f(Decoded::Big(Cow::Borrowed(self.as_ref())))
    }
}

duplicate_prims! {
    impl<'a, S, B> InspectEncoding<'a, S, B> for prim
    where
        S: SmallNumber<Big = B>,
        B: BigNumber + From<prim>
    {
        fn decode(self) -> Decoded<S, Cow<'a, B>> {
            #[allow(irrefutable_let_patterns)]
            #[allow(clippy::unnecessary_fallible_conversions)]
            if let Ok(small) = S::try_from(self) {
                Decoded::Small(small)
            } else {
                Decoded::Big(Cow::Owned(B::from(self)))
            }
        }

        fn small(&self) -> Option<S> {
            S::try_from(*self).ok()
        }

        fn with_decoded_ref<T>(&self, f: impl FnOnce(Decoded<S, Cow<B>>) -> T) -> T {
            match S::try_from(*self) {
                Ok(small) => f(Decoded::Small(small)),
                Err(_) => f(Decoded::Big(Cow::Owned(B::from(*self)))),
            }
        }
    }

    impl<'a, S: SmallNumber<Big = B>, B: BigNumber> InspectEncoding<'a, S, B> for &prim
    where
        B: From<prim>,
        S: TryFrom<prim>
    {
        fn decode(self) -> Decoded<S, Cow<'a, B>> {
            #[allow(irrefutable_let_patterns)]
            #[allow(clippy::unnecessary_fallible_conversions)]
            if let Ok(small) = S::try_from(*self) {
                Decoded::Small(small)
            } else {
                Decoded::Big(Cow::Owned(B::from(*self)))
            }
        }

        fn small(&self) -> Option<S> {
            S::try_from(**self).ok()
        }

        fn with_decoded_ref<T>(&self, f: impl FnOnce(Decoded<S, Cow<B>>) -> T) -> T {
            match S::try_from(**self) {
                Ok(small) => f(Decoded::Small(small)),
                Err(_) => f(Decoded::Big(Cow::Owned(B::from(**self)))),
            }
        }
    }
}

/// A decoded big number, which may be either small or big.  The second type parameter, `T`,
/// maybe be an owned value, a reference, a `Cow`, depending on the context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Decoded<S, B> {
    Small(S),
    Big(B),
}

pub trait EncodedRepr<'a, S, B: Clone>: Borrow<B> + Clone
where
    S: SmallNumber<Big = B>,
{
    fn into_owned(self) -> B;
}
