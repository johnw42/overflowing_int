use crate::encoding::Decode;
use crate::encoding::Decoded;
use crate::encoding::Encoding;
use crate::encoding::OwnedEncoding;
use crate::num_traits::small_number::SmallNumber;
use std::borrow::Cow;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CowEncoding<'enc, S>(Decoded<S, Cow<'enc, S::Big>>)
where
    S: SmallNumber;

impl<'enc, S> CowEncoding<'enc, S>
where
    S: SmallNumber,
{
    fn normalize(&mut self) {
        if let Decoded::Big(big) = &self.0
            && let Some(small) = S::try_from(big.as_ref()).ok()
        {
            *self = Self(Decoded::Small(small));
        }
    }
}

impl<'enc, S> Decode<'enc, S> for CowEncoding<'enc, S>
where
    S: SmallNumber,
{
    fn into_decoded(self) -> Decoded<S, Cow<'enc, S::Big>> {
        self.0
    }

    fn decode<'a>(&'a self) -> Decoded<S, Cow<'a, <S as SmallNumber>::Big>> {
        match &self.0 {
            Decoded::Small(s) => Decoded::Small(*s),
            Decoded::Big(b) => Decoded::Big(Cow::Borrowed(b.as_ref())),
        }
    }

    fn big_cow<'a>(&'a self) -> Cow<'a, <S as SmallNumber>::Big> {
        match &self.0 {
            Decoded::Small(s) => Cow::Owned(S::to_big(*s)),
            Decoded::Big(b) => Cow::Borrowed(b.as_ref()),
        }
    }
}

impl<'enc, S> Encoding<'enc> for CowEncoding<'enc, S>
where
    S: SmallNumber,
{
    type Small = S;
    type Big = S::Big;
    type Unsigned = CowEncoding<'enc, S::Unsigned>;
    type Owned = CowEncoding<'enc, S>;
    type Borrowed<'a>
        = CowEncoding<'a, S>
    where
        Self: 'a;

    const ZERO: Self = Self(Decoded::Small(S::ZERO));

    fn from_small(s: S) -> Self::Owned {
        CowEncoding(Decoded::Small(s))
    }

    fn from_big(b: S::Big) -> Self::Owned {
        let mut this = CowEncoding(Decoded::Big(Cow::Owned(b)));
        this.normalize();
        this
    }

    fn from_big_ref(b: &'enc S::Big) -> Self {
        let mut this = CowEncoding(Decoded::Big(Cow::Borrowed(b)));
        this.normalize();
        this
    }

    fn into_owned(self) -> Self::Owned {
        match self.0 {
            Decoded::Small(s) => CowEncoding(Decoded::Small(s)),
            Decoded::Big(b) => CowEncoding::from_big(b.into_owned()),
        }
    }

    fn borrow<'a>(&'a self) -> Self::Borrowed<'a> {
        match &self.0 {
            Decoded::Small(s) => CowEncoding(Decoded::Small(*s)),
            Decoded::Big(b) => CowEncoding(Decoded::Big(Cow::Borrowed(b.as_ref()))),
        }
    }
}

impl<'enc, S> OwnedEncoding<'enc> for CowEncoding<'enc, S>
where
    S: SmallNumber,
{
    fn decode_mut(&mut self) -> Decoded<S, &mut S::Big> {
        match &mut self.0 {
            Decoded::Small(s) => Decoded::Small(*s),
            Decoded::Big(b) => Decoded::Big(b.to_mut()),
        }
    }
}

#[cfg(any(test, feature = "quickcheck"))]
impl<S> quickcheck::Arbitrary for CowEncoding<'static, S>
where
    S: SmallNumber,
{
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if bool::arbitrary(g) {
            Self(Decoded::Small(<S as quickcheck::Arbitrary>::arbitrary(g)))
        } else {
            Self::from_big(<S::Big as quickcheck::Arbitrary>::arbitrary(g))
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<'enc, S> arbitrary::Arbitrary<'enc> for CowEncoding<'enc, S>
where
    S: SmallNumber,
{
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        if bool::arbitrary(u)? {
            Ok(Self(Decoded::Small(
                <S as arbitrary::Arbitrary>::arbitrary(u)?,
            )))
        } else {
            Ok(Self::from_big(<S::Big as arbitrary::Arbitrary>::arbitrary(
                u,
            )?))
        }
    }
}
