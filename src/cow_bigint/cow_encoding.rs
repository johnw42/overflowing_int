use std::borrow::Cow;
use std::fmt::Debug;

use num_bigint::BigInt;
use num_bigint::BigUint;

use duplicate::duplicate;

use crate::big_number::BigNumber;
use crate::generic_bignum::encoding::Decoded;
use crate::generic_bignum::encoding::EncodedBigNum;
use crate::generic_bignum::encoding::InspectEncoding;
use crate::small_num::SmallNumber;
use crate::{duplicate_prims, duplicate_uprims};

/// A wrapper type around `Encoding` that maintains the the invariant that
/// values that can be represented as `SmallInt` or `SmallUint` are always
/// stored as such, and only values that cannot be represented as `SmallInt` or
/// `SmallUint` are stored as `BigInt` or `BigUint`.  This type, in turn, is the
/// content of `CBigInt` and `CBigUint`, which implement high-level operations
/// and traits.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CowEncoding<'a, S: SmallNumber<Big = T>, T: BigNumber>(Decoded<S, Cow<'a, T>>);

impl<'a, S: SmallNumber<Big = T>, T: BigNumber> CowEncoding<'a, S, T> {
    fn normalize(&mut self) {
        if let Decoded::Big(big) = &self.0
            && let Some(small) = S::try_from(big.as_ref()).ok()
        {
            self.0 = Decoded::Small(small);
        }
    }
}

impl<'a, S, B> InspectEncoding<'a, S, B> for CowEncoding<'a, S, B>
where
    S: SmallNumber<Big = B>,
    B: BigNumber,
{
    fn decode(self) -> Decoded<S, Cow<'a, B>> {
        self.0
    }

    fn small(&self) -> Option<S> {
        match self.0 {
            Decoded::Small(s) => Some(s),
            Decoded::Big(_) => None,
        }
    }

    fn into_big_cow(self) -> Cow<'a, B> {
        match self.0 {
            Decoded::Small(s) => Cow::Owned(s.to_big()),
            Decoded::Big(b) => b,
        }
    }

    fn with_decoded_ref<T>(&self, f: impl FnOnce(Decoded<S, Cow<B>>) -> T) -> T {
        match &self.0 {
            Decoded::Small(s) => f(Decoded::Small(*s)),
            Decoded::Big(b) => f(Decoded::Big(Cow::Borrowed(b.as_ref()))),
        }
    }
}

impl<'a, S: SmallNumber<Big = B>, B: BigNumber> EncodedBigNum<'a> for CowEncoding<'a, S, B>
where
    B: Into<BigInt>,
{
    type Small = S;
    type Big = B;

    fn from_small(s: Self::Small) -> Self {
        Self(Decoded::Small(s))
    }

    fn from_big_cow(b: Cow<'a, Self::Big>) -> Self {
        Self(Decoded::Big(b))
    }

    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<Self::Small, Cow<'a, Self::Big>>)) {
        f(&mut self.0);
        self.normalize();
    }
}
