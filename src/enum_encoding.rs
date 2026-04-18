use crate::encoding::Decode;
use crate::encoding::Decoded;
use crate::encoding::Encode;
use crate::encoding::Encoding;
use crate::small_num::SmallNumber;
use std::borrow::Cow;
use std::fmt::Debug;
use std::marker::PhantomData;

/// A wrapper type around `Encoding` that maintains the the invariant that
/// values that can be represented as `SmallInt` or `SmallUint` are always
/// stored as such, and only values that cannot be represented as `SmallInt` or
/// `SmallUint` are stored as `BigInt` or `BigUint`.  This type, in turn, is the
/// content of `CBigInt` and `CBigUint`, which implement high-level operations
/// and traits.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumEncoding<'a, S: SmallNumber>(Decoded<S, S::Big>, PhantomData<&'a ()>);

impl<'a, S: SmallNumber> EnumEncoding<'a, S> {
    fn normalize(&mut self) {
        if let Decoded::Big(big) = &self.0
            && let Some(small) = S::try_from(big).ok()
        {
            self.0 = Decoded::Small(small);
        };
    }
}

impl<'a, S> Decode<'a, S> for EnumEncoding<'a, S>
where
    S: SmallNumber,
{
    fn into_decoded(self) -> Decoded<S, Cow<'a, S::Big>> {
        match self.0 {
            Decoded::Small(s) => Decoded::Small(s),
            Decoded::Big(b) => Decoded::Big(Cow::Owned(b)),
        }
    }

    fn decode<'b>(&'b self) -> Decoded<S, Cow<'b, <S as SmallNumber>::Big>> {
        match &self.0 {
            Decoded::Small(s) => Decoded::Small(*s),
            Decoded::Big(b) => Decoded::Big(Cow::Borrowed(b)),
        }
    }

    fn big_cow<'b>(&'b self) -> Cow<'b, <S as SmallNumber>::Big> {
        match &self.0 {
            Decoded::Small(s) => Cow::Owned(S::to_big(*s)),
            Decoded::Big(b) => Cow::Borrowed(b),
        }
    }
}

impl<'a, S> Encode<'a, S> for EnumEncoding<'a, S>
where
    S: SmallNumber,
{
    fn from_small(s: S) -> Self {
        Self(Decoded::Small(s), PhantomData)
    }

    fn from_big_cow(b: Cow<'a, S::Big>) -> Self {
        let mut r = Self(Decoded::Big(b.into_owned()), PhantomData);
        r.normalize();
        r
    }
}

impl<'a, S: SmallNumber> Encoding<'a> for EnumEncoding<'a, S> {
    type Small = S;
    type Big = S::Big;
    type Unsigned = EnumEncoding<'a, S::Unsigned>;
    type Static = EnumEncoding<'static, S>;
    type WithLifetime<'b>
        = EnumEncoding<'b, S>
    where
        'a: 'b;

    const ZERO: Self = Self(Decoded::Small(S::ZERO), PhantomData);

    fn borrow<'b>(&'b self) -> Self::WithLifetime<'b>
    where
        'a: 'b,
    {
        match &self.0 {
            Decoded::Small(s) => EnumEncoding(Decoded::Small(*s), PhantomData),
            Decoded::Big(b) => EnumEncoding(Decoded::Big(b.clone()), PhantomData),
        }
    }

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

    fn into_static(self) -> Self::Static {
        match self.0 {
            Decoded::Small(s) => EnumEncoding(Decoded::Small(s), PhantomData),
            Decoded::Big(b) => EnumEncoding(Decoded::Big(b), PhantomData),
        }
    }
}

#[cfg(any(test, feature = "quickcheck"))]
impl<S: SmallNumber> quickcheck::Arbitrary for EnumEncoding<'static, S> {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if bool::arbitrary(g) {
            Self(
                Decoded::Small(<S as quickcheck::Arbitrary>::arbitrary(g)),
                PhantomData,
            )
        } else {
            Self(
                Decoded::Big(<S::Big as quickcheck::Arbitrary>::arbitrary(g)),
                PhantomData,
            )
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<S: SmallNumber> arbitrary::Arbitrary<'_> for EnumEncoding<'static, S> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        if bool::arbitrary(u)? {
            Ok(Self(
                Decoded::Small(<S as arbitrary::Arbitrary>::arbitrary(u)?),
                PhantomData,
            ))
        } else {
            Ok(Self(
                Decoded::Big(<S::Big as arbitrary::Arbitrary>::arbitrary(u)?),
                PhantomData,
            ))
        }
    }
}
