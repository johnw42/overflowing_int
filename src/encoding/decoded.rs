use crate::encoding::Decode;
use crate::encoding::Decoded;
use crate::encoding::Encoding;
use crate::num_traits::small_number::SmallNumber;
use std::borrow::Cow;

pub type DecodedEncoding<S> = Decoded<S, <S as SmallNumber>::Big>;

impl<'enc, S> Decode<'enc, S> for DecodedEncoding<S>
where
    S: SmallNumber,
{
    fn into_decoded(self) -> Decoded<S, Cow<'enc, S::Big>> {
        match self {
            Decoded::Small(s) => Decoded::Small(s),
            Decoded::Big(b) => Decoded::Big(Cow::Owned(b)),
        }
    }

    fn decode<'a>(&'a self) -> Decoded<S, Cow<'a, <S as SmallNumber>::Big>> {
        match self {
            Decoded::Small(s) => Decoded::Small(*s),
            Decoded::Big(b) => Decoded::Big(Cow::Borrowed(b)),
        }
    }

    fn big_cow<'a>(&'a self) -> Cow<'a, <S as SmallNumber>::Big> {
        match self {
            Decoded::Small(s) => Cow::Owned(S::to_big(*s)),
            Decoded::Big(b) => Cow::Borrowed(b),
        }
    }
}

impl<'enc, S> Encoding<'enc> for DecodedEncoding<S>
where
    S: SmallNumber,
{
    type Small = S;
    type Big = S::Big;
    type Unsigned = DecodedEncoding<S::Unsigned>;
    type Static = Self;

    const ZERO: Self = Decoded::Small(S::ZERO);

    fn from_small(s: S) -> Self {
        Decoded::Small(s)
    }

    fn from_big(b: S::Big) -> Self {
        let mut this = Decoded::Big(b);
        if let Decoded::Big(big) = &this
            && let Some(small) = S::try_from(big).ok()
        {
            this = Decoded::Small(small);
        }
        this
    }

    fn from_big_ref(b: &'enc S::Big) -> Self {
        Self::from_big(b.clone())
    }

    fn into_static(self) -> Self::Static {
        self
    }

    fn decode_mut(&mut self) -> Decoded<S, &mut <S as SmallNumber>::Big> {
        match self {
            Decoded::Small(s) => Decoded::Small(*s),
            Decoded::Big(b) => Decoded::Big(b),
        }
    }
}

#[cfg(any(test, feature = "quickcheck"))]
impl<S> quickcheck::Arbitrary for DecodedEncoding<S>
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
impl<S> arbitrary::Arbitrary<'_> for DecodedEncoding<S>
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
