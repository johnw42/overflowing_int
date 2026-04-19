use crate::cow_encoding::CowEncoding;
use crate::encoding::Decode;
use crate::encoding::Decoded;
use crate::encoding::Encode;
use crate::encoding::Encoding;
use crate::encoding::EncodingMut;
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
pub struct EnumEncoding<S>(Decoded<S, S::Big>)
where
    S: SmallNumber;

impl<S> EnumEncoding<S>
where
    S: SmallNumber,
{
    fn normalize(&mut self) {
        if let Decoded::Big(big) = &self.0
            && let Some(small) = S::try_from(big).ok()
        {
            self.0 = Decoded::Small(small);
        };
    }
}

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

impl<'enc, S> Encode<'enc, S> for EnumEncoding<S>
where
    S: SmallNumber,
{
    fn from_small(s: S) -> Self {
        Self(Decoded::Small(s))
    }

    fn from_big(b: S::Big) -> Self {
        let mut r = Self(Decoded::Big(b));
        r.normalize();
        r
    }

    fn from_big_ref(b: &'enc S::Big) -> Self {
        let mut r = Self(Decoded::Big(b.clone()));
        r.normalize();
        r
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
    type Borrowed<'a> = CowEncoding<'a, S>;

    const ZERO: Self = Self(Decoded::Small(S::ZERO));

    fn borrow<'a>(&'a self) -> Self::Borrowed<'a>
    where
        Self: 'a,
    {
        match &self.0 {
            Decoded::Small(s) => CowEncoding::from_small(*s),
            Decoded::Big(b) => CowEncoding::from_big_ref(b),
        }
    }

    fn into_owned(self) -> Self::Owned {
        self
    }
}

impl<'enc, S> EncodingMut<'enc> for EnumEncoding<S>
where
    S: SmallNumber,
{
    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<Self::Small, Cow<Self::Big>>)) {
        let mut swapped = Decoded::Small(S::ZERO);
        std::mem::swap(&mut self.0, &mut swapped);
        let mut encoding = match swapped {
            Decoded::Small(s) => Decoded::Small(s),
            Decoded::Big(b) => Decoded::Big(Cow::Owned(b)),
        };
        f(&mut encoding);
        self.0 = match encoding {
            Decoded::Small(s) => Decoded::Small(s),
            Decoded::Big(b) => Decoded::Big(b.into_owned()),
        };
        self.normalize();
    }
}

#[cfg(any(test, feature = "quickcheck"))]
impl<S> quickcheck::Arbitrary for EnumEncoding<S>
where
    S: SmallNumber,
{
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if bool::arbitrary(g) {
            Self(Decoded::Small(<S as quickcheck::Arbitrary>::arbitrary(g)))
        } else {
            Self(Decoded::Big(<S::Big as quickcheck::Arbitrary>::arbitrary(
                g,
            )))
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<S> arbitrary::Arbitrary<'_> for EnumEncoding<S>
where
    S: SmallNumber,
{
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        if bool::arbitrary(u)? {
            Ok(Self(Decoded::Small(
                <S as arbitrary::Arbitrary>::arbitrary(u)?,
            )))
        } else {
            Ok(Self(Decoded::Big(
                <S::Big as arbitrary::Arbitrary>::arbitrary(u)?,
            )))
        }
    }
}
