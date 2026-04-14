use crate::encoding::{Decode, Decoded, Encode, Encoding, EncodingKind};
use crate::small_num::SmallNumber;
use num_traits::ConstZero as _;
use std::hash::Hash;
use std::marker::PhantomData;
use std::{borrow::Cow, fmt::Debug};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct IdentityEncoding<'a, S>(Cow<'a, S::Big>, PhantomData<&'a ()>)
where
    S: SmallNumber;

impl<'a, S> Decode<'a, S> for IdentityEncoding<'a, S>
where
    S: SmallNumber,
{
    #[inline]
    fn kind() -> EncodingKind {
        EncodingKind::Cow
    }

    #[inline]
    fn decode(self) -> Decoded<S, Cow<'a, S::Big>> {
        Decoded::Big(Cow::Owned(self.0.into_owned()))
    }

    #[inline]
    fn decode_ref<'b>(&'b self) -> Decoded<S, Cow<'b, <S as SmallNumber>::Big>> {
        Decoded::Big(Cow::Borrowed(self.0.as_ref()))
    }

    #[inline]
    fn with_decoded<'b, T>(&'b self, f: impl FnOnce(Decoded<S, Cow<'b, S::Big>>) -> T) -> T {
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
        self.0.into_owned()
    }

    #[inline]
    fn with_big_cow<T>(&self, f: impl FnOnce(Cow<<S as SmallNumber>::Big>) -> T) -> T {
        f(Cow::Borrowed(&self.0))
    }
}

impl<'a, S> Encode<'a, S> for IdentityEncoding<'a, S>
where
    S: SmallNumber,
{
    #[inline]
    fn from_small(s: S) -> Self {
        Self(Cow::Owned(s.to_big()), PhantomData)
    }

    #[inline]
    fn from_big_cow(b: Cow<'a, S::Big>) -> Self {
        Self(b, PhantomData)
    }

    #[inline]
    fn from_big(b: <S as SmallNumber>::Big) -> Self {
        Self(Cow::Owned(b), PhantomData)
    }

    #[inline]
    fn from_decoded(enc: Decoded<S, Cow<'a, <S as SmallNumber>::Big>>) -> Self {
        match enc {
            Decoded::Small(_) => unreachable!(),
            Decoded::Big(b) => Self::from_big_cow(b),
        }
    }
}

impl<'a, S> Encoding<'a> for IdentityEncoding<'a, S>
where
    S: SmallNumber,
{
    type Small = S;
    type Big = S::Big;
    type Unsigned = IdentityEncoding<'a, S::Unsigned>;
    type Static = IdentityEncoding<'static, S>;
    type WithLifetime<'b>
        = IdentityEncoding<'b, S>
    where
        Self: 'b,
        'a: 'b;

    const ZERO: Self = Self(Cow::Owned(S::Big::ZERO), PhantomData);

    #[inline]
    fn borrow<'b>(&'b self) -> Self::WithLifetime<'b>
    where
        Self: 'b,
        'a: 'b,
    {
        Self(self.0.clone(), PhantomData)
    }

    #[inline]
    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<Self::Small, Cow<Self::Big>>)) {
        let mut decoded = Decoded::Big(Cow::Borrowed(self.0.as_ref()));
        f(&mut decoded);
        *self = match decoded {
            Decoded::Small(_) => unreachable!(),
            Decoded::Big(b) => Self::from_big(b.into_owned()),
        };
    }

    #[inline]
    fn into_static(self) -> IdentityEncoding<'static, S> {
        IdentityEncoding(Cow::Owned(self.0.into_owned()), PhantomData)
    }
}

impl<'a, S> Debug for IdentityEncoding<'a, S>
where
    S: SmallNumber,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[cfg(any(test, feature = "quickcheck"))]
impl<S: SmallNumber> quickcheck::Arbitrary for IdentityEncoding<'static, S> {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self(Cow::Owned(S::Big::arbitrary(g)), PhantomData)
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, S: SmallNumber> arbitrary::Arbitrary<'a> for IdentityEncoding<'a, S> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(Self(Cow::Owned(S::Big::arbitrary(u)?), PhantomData))
    }
}
