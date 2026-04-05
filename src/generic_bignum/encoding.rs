use std::{
    borrow::{Borrow, Cow},
    rc::Rc,
};

use num_bigint::BigInt;
use quickcheck::Gen;

use crate::{big_number::BigNumber, duplicate_prims};
use crate::{generic_bignum::GenericBigNum, small_num::SmallNumber};

pub trait EncodedBigNum<'a>
where
    Self: Sized + Clone,
    Self::Big: Into<BigInt>,
{
    type Small: SmallNumber<Big = Self::Big>;
    type Big: BigNumber;
    type Repr: EncodedRepr<'a, Self::Small, Self::Big>;

    const ZERO: Self;
    const ONE: Self;

    /// Encodes a value.  Prefer [`Self::from_small`] or [`Self::from_big`] when possible.
    fn from_decoded(enc: Decoded<Self::Small, Cow<'a, Self::Big>>) -> Self {
        match enc {
            Decoded::Small(s) => Self::from_small(s),
            Decoded::Big(b) => Self::from_big_cow(b),
        }
    }

    /// Encodes a small value.
    fn from_small(s: Self::Small) -> Self;

    /// Encodes a big value. Prefer this when the big value is already owned, and [`Self::from_big_cow`] when it is not.
    fn from_big(b: Self::Big) -> Self;

    /// Encodes a big value from a `Cow`.  Prefer [`Self::from_big`] when the value is known to be owned.
    fn from_big_cow(b: Cow<'a, Self::Big>) -> Self;

    /// Updates the encoding in place using the provided function.
    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<Self::Small, Self::Big>));

    /// Decodes the value, preferably without cloning.
    fn decode(self) -> Decoded<Self::Small, Cow<'a, Self::Big>>;

    /// Decodes the value by reference, preferably without cloning.
    fn decode_ref(&self) -> Decoded<Self::Small, Cow<'a, Self::Big>>;

    /// Gets the small value if it is small, or `None` if it is big.
    fn small(&self) -> Option<Self::Small>;

    /// Gets the big value by reference if it is big, or `None` if it is small.
    fn big_ref(&self) -> Option<&'a Self::Big>;

    /// Gets the big value as a `Cow`, cloning if necessary.
    fn big_cow(&self) -> Cow<'a, Self::Big>;

    /// Gets the big value as a `Cow`, cloning if necessary.
    fn into_big_cow(self) -> Cow<'a, Self::Big>;
}

/// A decoded big number, which may be either small or big.  The second type parameter, `T`,
/// maybe be an owned value, a reference, a `Cow`, depending on the context.
pub enum Decoded<S: SmallNumber, T> {
    Small(S),
    Big(T),
}

pub trait EncodedRepr<'a, S, B: Clone>: Borrow<B> + Clone
where
    S: SmallNumber<Big = B>,
{
    fn into_owned(self) -> B;
}

pub trait InspectEncoding<'a, S: SmallNumber, B: BigNumber>: Sized {
    /// Decodes the value, preferably without cloning.
    fn decode(self) -> Decoded<S, Cow<'a, B>>;

    // /// Decodes the value by reference, preferably without cloning.
    // fn decode_ref(&'a self) -> Decoded<S, Cow<'a, B>>;

    /// Gets the small value if it is small, or `None` if it is big.
    fn small(&self) -> Option<S>;

    // /// Gets the big value by reference if it is big, or `None` if it is small.
    // fn big_ref(&'a self) -> Option<&'a B>;

    // /// Gets the big value as a `Cow`, cloning if necessary.
    // fn big_cow(&'a self) -> Cow<'a, B>;

    /// Gets the big value as a `Cow`, cloning if necessary.
    fn into_big_cow(self) -> Cow<'a, B>;
}

impl<'a, E: EncodedBigNum<'a>> InspectEncoding<'a, E::Small, E::Big> for GenericBigNum<'a, E> {
    fn decode(self) -> Decoded<E::Small, Cow<'a, E::Big>> {
        <Self as EncodedBigNum>::decode(self)
    }

    fn small(&self) -> Option<E::Small> {
        <Self as EncodedBigNum>::small(self)
    }

    fn into_big_cow(self) -> Cow<'a, E::Big> {
        <Self as EncodedBigNum>::into_big_cow(self)
    }
}

impl<'a, E: EncodedBigNum<'a>> InspectEncoding<'a, E::Small, E::Big> for &GenericBigNum<'a, E> {
    fn decode(self) -> Decoded<E::Small, Cow<'a, E::Big>> {
        <GenericBigNum<'a, E> as EncodedBigNum>::decode_ref(self)
    }

    fn small(&self) -> Option<E::Small> {
        <GenericBigNum<'a, E> as EncodedBigNum>::small(self)
    }

    fn into_big_cow(self) -> Cow<'a, E::Big> {
        <GenericBigNum<'a, E> as EncodedBigNum>::big_cow(self)
    }
}

duplicate_prims! {
    impl<'a, S: SmallNumber, B: BigNumber> InspectEncoding<'a, S, B> for prim
    where
        B: From<Self>
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

        // fn decode_ref(&self) -> Decoded<S, Cow<'a, B>> {
        //     #[allow(irrefutable_let_patterns)]
        //     #[allow(clippy::unnecessary_fallible_conversions)]
        //     if let Ok(small) = S::try_from(*self) {
        //         Decoded::Small(small)
        //     } else {
        //         Decoded::Big(Cow::Owned(B::from(*self)))
        //     }
        // }

        fn small(&self) -> Option<S> {
            S::try_from(*self).ok()
        }

        // fn big_ref(&self) -> Option<&'a B> {
        //     None
        // }

        // fn big_cow(&self) -> Cow<'a, B> {
        //     Cow::Owned(B::from(*self))
        // }

        fn into_big_cow(self) -> Cow<'a, B> {
            Cow::Owned(B::from(self))
        }
    }

    impl<'a, S: SmallNumber, B: BigNumber> InspectEncoding<'a, S, B> for &prim
    where
        B: From<Self> + From<S>,
        S: TryFrom<Self> + TryFrom<B>,
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

        // fn decode_ref(&self) -> Decoded<S, Cow<'a, B>> {
        //     #[allow(irrefutable_let_patterns)]
        //     #[allow(clippy::unnecessary_fallible_conversions)]
        //     if let Ok(small) = S::try_from(*self) {
        //         Decoded::Small(small)
        //     } else {
        //         Decoded::Big(Cow::Owned(B::from(*self)))
        //     }
        // }

        fn small(&self) -> Option<S> {
            S::try_from(*self).ok()
        }

        // fn big_ref(&self) -> Option<&'a B> {
        //     None
        // }

        // fn big_cow(&'a self) -> Cow<'a, B> {
        //     Cow::Owned(B::from(*self))
        // }

        fn into_big_cow(self) -> Cow<'a, B> {
            Cow::Owned(B::from(self))
        }
    }

}

impl<'a, S, B> InspectEncoding<'a, S, B> for Cow<'a, B>
where
    S: SmallNumber<Big = B>,
    B: BigNumber,
{
    fn decode(self) -> Decoded<S, Cow<'a, B>> {
        Decoded::Big(self)
    }

    // fn decode_ref(&'a self) -> Decoded<S, Cow<'a, B>> {
    //     Decoded::Big(Cow::Borrowed(self.borrow()))
    // }

    fn small(&self) -> Option<S> {
        None
    }

    // fn big_ref(&'a self) -> Option<&'a B> {
    //     Some(self.borrow())
    // }

    // fn big_cow(&'a self) -> Cow<'a, B> {
    //     Cow::Borrowed(self.borrow())
    // }

    fn into_big_cow(self) -> Cow<'a, B> {
        self
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

    // fn decode_ref(&'a self) -> Decoded<S, Cow<'a, B>> {
    //     Decoded::Big(Cow::Borrowed(self.borrow()))
    // }

    fn small(&self) -> Option<S> {
        None
    }

    // fn big_ref(&'a self) -> Option<&'a B> {
    //     Some(self.borrow())
    // }

    // fn big_cow(&'a self) -> Cow<'a, B> {
    //     Cow::Borrowed(self.borrow())
    // }

    fn into_big_cow(self) -> Cow<'a, B> {
        match Rc::try_unwrap(self) {
            Ok(b) => Cow::Owned(b),
            Err(rc) => Cow::Owned((*rc).clone()),
        }
    }
}
