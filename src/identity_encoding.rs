use crate::encoding::{Decode, Decoded, Encode, Encoding, EncodingKind};
use crate::small_num::SmallNumber;
use num_traits::ConstZero as _;
use std::hash::Hash;
use std::{borrow::Cow, fmt::Debug};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct IdentityEncoding<S>(S::Big)
where
    S: SmallNumber;

impl<S> Decode<'static, S> for IdentityEncoding<S>
where
    S: SmallNumber,
{
    #[inline]
    fn kind() -> EncodingKind {
        EncodingKind::Trivial
    }

    #[inline]
    fn decode(self) -> Decoded<S, Cow<'static, S::Big>> {
        Decoded::Big(Cow::Owned(self.0))
    }

    #[inline]
    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T {
        f(Decoded::Big(Cow::Borrowed(&self.0)))
    }

    #[inline]
    fn owns_bignum(&self) -> bool {
        true
    }

    #[inline]
    fn small(&self) -> Option<S> {
        None
    }

    #[inline]
    fn into_big(self) -> S::Big {
        self.0
    }

    #[inline]
    fn with_big_cow<T>(&self, f: impl FnOnce(Cow<<S as SmallNumber>::Big>) -> T) -> T {
        f(Cow::Borrowed(&self.0))
    }
}

impl<S> Encode<'static, S> for IdentityEncoding<S>
where
    S: SmallNumber,
{
    #[inline]
    fn from_small(s: S) -> Self {
        Self(s.to_big())
    }

    #[inline]
    fn from_big_cow(b: Cow<'static, S::Big>) -> Self {
        Self(b.into_owned())
    }

    #[inline]
    fn from_big(b: <S as SmallNumber>::Big) -> Self {
        Self(b)
    }

    #[inline]
    fn from_decoded(enc: Decoded<S, Cow<'static, <S as SmallNumber>::Big>>) -> Self {
        match enc {
            Decoded::Small(_) => unreachable!(),
            Decoded::Big(b) => Self::from_big_cow(b),
        }
    }
}

impl<S> Encoding<'static> for IdentityEncoding<S>
where
    S: SmallNumber,
{
    type Small = S;
    type Big = S::Big;
    type Unsigned = IdentityEncoding<S::Unsigned>;
    type Static = IdentityEncoding<S>;

    const ZERO: Self = Self(S::Big::ZERO);

    #[inline]
    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<Self::Small, Cow<Self::Big>>)) {
        let mut decoded = Decoded::Big(Cow::Borrowed(&self.0));
        f(&mut decoded);
        *self = match decoded {
            Decoded::Small(_) => unreachable!(),
            Decoded::Big(b) => Self::from_big(b.into_owned()),
        };
    }

    #[inline]
    fn into_static(self) -> Self::Static {
        self
    }
}

impl<S> Debug for IdentityEncoding<S>
where
    S: SmallNumber,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[cfg(any(test, feature = "quickcheck"))]
impl<S: SmallNumber> quickcheck::Arbitrary for IdentityEncoding<S> {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self(S::Big::arbitrary(g))
    }
}

#[cfg(feature = "arbitrary")]
impl<S: SmallNumber> arbitrary::Arbitrary<'_> for IdentityEncoding<S> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(Self(S::Big::arbitrary(u)?))
    }
}
