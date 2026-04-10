use crate::encoding::{Decode, Decoded, Encode, Encoding};
use crate::small_num::SmallNumber;
use std::hash::Hash;
use std::{borrow::Cow, fmt::Debug};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct TrivialEncoding<S>(S::Big)
where
    S: SmallNumber;

impl<S> Decode<'static, S> for TrivialEncoding<S>
where
    S: SmallNumber,
{
    fn decode(self) -> Decoded<S, Cow<'static, S::Big>> {
        Decoded::Big(Cow::Owned(self.0))
    }

    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T {
        f(Decoded::Big(Cow::Borrowed(&self.0)))
    }

    fn owns_bignum(&self) -> bool {
        true
    }
}

impl<S> Encode<'static, S> for TrivialEncoding<S>
where
    S: SmallNumber,
{
    fn from_small(s: S) -> Self {
        Self(s.to_big())
    }

    fn from_big_cow(b: Cow<'static, S::Big>) -> Self {
        Self(b.into_owned())
    }
}

impl<S> Encoding<'static> for TrivialEncoding<S>
where
    S: SmallNumber,
{
    type Small = S;
    type Big = S::Big;
    type Unsigned = TrivialEncoding<S::Unsigned>;
    type Static = TrivialEncoding<S>;

    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<Self::Small, Cow<Self::Big>>)) {
        let mut decoded = Decoded::Big(Cow::Borrowed(&self.0));
        f(&mut decoded);
        *self = match decoded {
            Decoded::Small(s) => Self::from_small(s),
            Decoded::Big(b) => Self::from_big(b.into_owned()),
        };
    }

    fn into_static(self) -> Self::Static {
        self
    }
}

impl<S> Debug for TrivialEncoding<S>
where
    S: SmallNumber,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[cfg(any(test, feature = "quickcheck"))]
impl<S: SmallNumber> quickcheck::Arbitrary for TrivialEncoding<S> {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self(S::Big::arbitrary(g))
    }
}

#[cfg(feature = "arbitrary")]
impl<S: SmallNumber> arbitrary::Arbitrary<'_> for TrivialEncoding<S> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(Self(S::Big::arbitrary(u)?))
    }
}
