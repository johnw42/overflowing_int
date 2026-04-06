use crate::generic_bignum::encoding::Decode;
use crate::generic_bignum::encoding::Decoded;
use crate::generic_bignum::encoding::Encoding;
use crate::small_num::SmallNumber;
use std::borrow::Cow;
use std::fmt::Debug;

/// A wrapper type around `Encoding` that maintains the the invariant that
/// values that can be represented as `SmallInt` or `SmallUint` are always
/// stored as such, and only values that cannot be represented as `SmallInt` or
/// `SmallUint` are stored as `BigInt` or `BigUint`.  This type, in turn, is the
/// content of `CBigInt` and `CBigUint`, which implement high-level operations
/// and traits.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CowEncoding<'a, S: SmallNumber>(Decoded<S, Cow<'a, S::Big>>);

impl<'a, S: SmallNumber> CowEncoding<'a, S> {
    fn normalize(&mut self) {
        if let Decoded::Big(big) = &self.0
            && let Some(small) = S::try_from(big.as_ref()).ok()
        {
            self.0 = Decoded::Small(small);
        };
    }
}

impl<'a, S> Decode<'a, S> for CowEncoding<'a, S>
where
    S: SmallNumber,
{
    fn decode(self) -> Decoded<S, Cow<'a, S::Big>> {
        self.0
    }

    fn small(&self) -> Option<S> {
        match self.0 {
            Decoded::Small(s) => Some(s),
            Decoded::Big(_) => None,
        }
    }

    fn into_big_cow(self) -> Cow<'a, S::Big> {
        match self.0 {
            Decoded::Small(s) => Cow::Owned(s.to_big()),
            Decoded::Big(b) => b,
        }
    }

    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T {
        match &self.0 {
            Decoded::Small(s) => f(Decoded::Small(*s)),
            Decoded::Big(b) => f(Decoded::Big(Cow::Borrowed(b.as_ref()))),
        }
    }
}

impl<'a, S: SmallNumber> Encoding<'a> for CowEncoding<'a, S> {
    type Small = S;
    type Big = S::Big;

    fn from_small(s: Self::Small) -> Self {
        Self(Decoded::Small(s))
    }

    fn from_big_cow(b: Cow<'a, Self::Big>) -> Self {
        let mut r = Self(Decoded::Big(b));
        r.normalize();
        r
    }

    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<Self::Small, Cow<'a, Self::Big>>)) {
        f(&mut self.0);
        self.normalize();
    }
}

impl<S: SmallNumber> quickcheck::Arbitrary for CowEncoding<'static, S> {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if bool::arbitrary(g) {
            Self(Decoded::Small(<S as quickcheck::Arbitrary>::arbitrary(g)))
        } else {
            Self(Decoded::Big(Cow::Owned(
                <S::Big as quickcheck::Arbitrary>::arbitrary(g),
            )))
        }
    }
}

impl<S: SmallNumber> arbitrary::Arbitrary<'_> for CowEncoding<'static, S> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        if bool::arbitrary(u)? {
            Ok(Self(Decoded::Small(
                <S as arbitrary::Arbitrary>::arbitrary(u)?,
            )))
        } else {
            Ok(Self(Decoded::Big(Cow::Owned(
                <S::Big as arbitrary::Arbitrary>::arbitrary(u)?,
            ))))
        }
    }
}
