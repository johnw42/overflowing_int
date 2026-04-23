use crate::encoding::Decode;
use crate::encoding::Decoded;
use crate::encoding::Encoding;
use crate::small_num::SmallNumber;
use std::borrow::Cow;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumEncoding<S>(Decoded<S, S::Big>)
where
    S: SmallNumber;

impl<'enc, S> Decode<'enc, S> for EnumEncoding<S>
where
    S: SmallNumber,
{
    fn into_decoded(self) -> Decoded<S, Cow<'enc, S::Big>> {
        match self.0 {
            Decoded::Small(s) => Decoded::Small(s),
            Decoded::Big(b) => Decoded::Big(Cow::Owned(b)),
        }
    }

    fn decode<'a>(&'a self) -> Decoded<S, Cow<'a, <S as SmallNumber>::Big>> {
        match &self.0 {
            Decoded::Small(s) => Decoded::Small(*s),
            Decoded::Big(b) => Decoded::Big(Cow::Borrowed(b)),
        }
    }

    fn big_cow<'a>(&'a self) -> Cow<'a, <S as SmallNumber>::Big> {
        match &self.0 {
            Decoded::Small(s) => Cow::Owned(S::to_big(*s)),
            Decoded::Big(b) => Cow::Borrowed(b),
        }
    }
}

impl<'enc, S> Encoding<'enc> for EnumEncoding<S>
where
    S: SmallNumber,
{
    type Small = S;
    type Big = S::Big;
    type Unsigned = EnumEncoding<S::Unsigned>;
    type Owned = Self;
    type Borrowed<'a> = Self;

    const ZERO: Self = Self(Decoded::Small(S::ZERO));

    fn from_small(s: S) -> Self {
        Self(Decoded::Small(s))
    }

    fn from_big(b: S::Big) -> Self {
        let mut r = Self(Decoded::Big(b));
        if let Decoded::Big(big) = &r.0
            && let Some(small) = S::try_from(big).ok()
        {
            r.0 = Decoded::Small(small);
        }
        r
    }

    fn from_big_ref(b: &'enc S::Big) -> Self {
        Self::from_big(b.clone())
    }

    fn borrow<'a>(&'a self) -> Self::Borrowed<'a> {
        self.clone()
    }

    fn into_owned(self) -> Self::Owned {
        self
    }

    fn decode_mut(&mut self) -> Decoded<S, &mut <S as SmallNumber>::Big> {
        match &mut self.0 {
            Decoded::Small(s) => Decoded::Small(*s),
            Decoded::Big(b) => Decoded::Big(b),
        }
    }
}

#[cfg(any(test, feature = "quickcheck"))]
impl<S> quickcheck::Arbitrary for EnumEncoding<S>
where
    S: SmallNumber,
{
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if bool::arbitrary(g) {
            Self::from_small(<S as quickcheck::Arbitrary>::arbitrary(g))
        } else {
            Self::from_big(<S::Big as quickcheck::Arbitrary>::arbitrary(g))
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<S> arbitrary::Arbitrary<'_> for EnumEncoding<S>
where
    S: SmallNumber,
{
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(if bool::arbitrary(u)? {
            Self::from_small(<S as arbitrary::Arbitrary>::arbitrary(u)?)
        } else {
            Self::from_big(<S::Big as arbitrary::Arbitrary>::arbitrary(u)?)
        })
    }
}
