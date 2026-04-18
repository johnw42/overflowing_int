use crate::encoding::{Decode, Decoded, Encode, Encoding};
use crate::small_num::SmallNumber;
use num_traits::ConstZero as _;
use std::hash::Hash;
use std::marker::PhantomData;
use std::{borrow::Cow, fmt::Debug};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct IdentityEncoding<'enc, S>(Cow<'enc, S::Big>, PhantomData<&'enc ()>)
where
    S: SmallNumber;

impl<'enc, S> Decode<'enc, S> for IdentityEncoding<'enc, S>
where
    S: SmallNumber,
{
    #[inline]
    fn into_decoded(self) -> Decoded<S, Cow<'enc, S::Big>> {
        Decoded::Big(Cow::Owned(self.0.into_owned()))
    }

    #[inline]
    fn decode<'a>(&'a self) -> Decoded<S, Cow<'a, <S as SmallNumber>::Big>> {
        Decoded::Big(Cow::Borrowed(self.0.as_ref()))
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
    fn big_cow<'a>(&'a self) -> Cow<'a, <S as SmallNumber>::Big> {
        Cow::Borrowed(self.0.as_ref())
    }
}

impl<'enc, S> Encode<'enc, S> for IdentityEncoding<'enc, S>
where
    S: SmallNumber,
{
    #[inline]
    fn from_small(s: S) -> Self {
        Self(Cow::Owned(s.to_big()), PhantomData)
    }

    #[inline]
    fn from_big_cow(b: Cow<'enc, S::Big>) -> Self {
        Self(b, PhantomData)
    }

    #[inline]
    fn from_big(b: <S as SmallNumber>::Big) -> Self {
        Self(Cow::Owned(b), PhantomData)
    }

    #[inline]
    fn from_decoded(enc: Decoded<S, Cow<'enc, <S as SmallNumber>::Big>>) -> Self {
        match enc {
            Decoded::Small(_) => unreachable!(),
            Decoded::Big(b) => Self::from_big_cow(b),
        }
    }
}

impl<'enc, S> Encoding<'enc> for IdentityEncoding<'enc, S>
where
    S: SmallNumber,
{
    type Small = S;
    type Big = S::Big;
    type Unsigned = IdentityEncoding<'enc, S::Unsigned>;
    type Static = IdentityEncoding<'static, S>;
    type WithLifetime<'a>
        = IdentityEncoding<'a, S>
    where
        Self: 'a,
        'enc: 'a;

    const ZERO: Self = Self(Cow::Owned(S::Big::ZERO), PhantomData);

    #[inline]
    fn borrow<'a>(&'a self) -> Self::WithLifetime<'a>
    where
        Self: 'a,
        'enc: 'a,
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

impl<'enc, S> Debug for IdentityEncoding<'enc, S>
where
    S: SmallNumber,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[cfg(any(test, feature = "quickcheck"))]
impl<S> quickcheck::Arbitrary for IdentityEncoding<'static, S>
where
    S: SmallNumber,
{
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self(Cow::Owned(S::Big::arbitrary(g)), PhantomData)
    }
}

#[cfg(feature = "arbitrary")]
impl<'enc, S> arbitrary::Arbitrary<'enc> for IdentityEncoding<'enc, S>
where
    S: SmallNumber,
{
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(Self(Cow::Owned(S::Big::arbitrary(u)?), PhantomData))
    }
}
