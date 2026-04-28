use crate::encoding::int_or_ptr::{IntOrPtr, IntOrPtrData};
use crate::encoding::{Decode, Decoded, Encoding};
use crate::num_traits::small_number::SmallNumber;
use num_bigint::{BigInt, BigUint};
use std::hash::Hash;
use std::sync::Arc;
use std::{borrow::Cow, fmt::Debug};

const _: () = {
    assert!(align_of::<Arc<BigInt>>() > 1);
    assert!(align_of::<Arc<BigUint>>() > 1);
    assert!(size_of::<Arc<BigInt>>() == size_of::<i64>());
    assert!(size_of::<Arc<BigUint>>() == size_of::<u64>());
};

/// An encoding that uses `Arc` for big values, and a small value with the LSB
/// set to 1 for small values.  This encoding is used for `ArcBigInt` and
/// `ArcBigUint`.
#[derive(Clone)]
pub struct ArcEncoding<S>(IntOrPtrData<S, S::Big, Arc<S::Big>>)
where
    S: SmallNumber;

impl<'enc, S> Decode<'enc, S> for ArcEncoding<S>
where
    S: SmallNumber,
{
    fn into_decoded(self) -> Decoded<S, Cow<'static, S::Big>> {
        match self.0.into_inner() {
            IntOrPtr::Int(s) => Decoded::Small(s),
            IntOrPtr::Ptr(b) => Decoded::Big(Cow::Owned(Arc::unwrap_or_clone(b))),
        }
    }

    fn decode<'a>(&'a self) -> Decoded<S, Cow<'a, <S as SmallNumber>::Big>> {
        match self.0.get() {
            IntOrPtr::Int(s) => Decoded::Small(s),
            IntOrPtr::Ptr(b) => Decoded::Big(Cow::Borrowed(b)),
        }
    }
}

impl<'enc, S> Encoding<'enc> for ArcEncoding<S>
where
    S: SmallNumber,
{
    type Small = S;
    type Big = S::Big;
    type Unsigned = ArcEncoding<S::Unsigned>;
    type Static = Self;

    const ZERO: Self = Self(IntOrPtrData::ZERO);

    fn from_small(s: S) -> Self {
        Self(match IntOrPtrData::new_int(s) {
            Some(int) => int,
            None => IntOrPtrData::new_ptr(Arc::new(s.to_big())),
        })
    }

    fn from_big(b: S::Big) -> Self {
        Self(match S::try_from(&b).ok() {
            Some(small) if let Some(int) = IntOrPtrData::new_int(small) => int,
            _ => IntOrPtrData::new_ptr(Arc::new(b)),
        })
    }

    fn from_big_ref(b: &'enc S::Big) -> Self {
        Self(match S::try_from(b).ok() {
            Some(small) if let Some(int) = IntOrPtrData::new_int(small) => int,
            _ => IntOrPtrData::new_ptr(Arc::new(b.clone())),
        })
    }

    fn into_static(self) -> Self::Static {
        self
    }

    fn decode_mut(&mut self) -> Decoded<S, &mut S::Big> {
        match self.0.get_mut() {
            IntOrPtr::Int(s) => Decoded::Small(s),
            IntOrPtr::Ptr(b) => Decoded::Big(Arc::make_mut(b)),
        }
    }
}

impl<S> Debug for ArcEncoding<S>
where
    S: SmallNumber,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.decode() {
            Decoded::Small(s) => write!(f, "Small({})", s),
            Decoded::Big(b) => write!(f, "Big({})", b.as_ref()),
        }
    }
}

impl<S> Hash for ArcEncoding<S>
where
    S: SmallNumber,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self.decode() {
            Decoded::Small(s) => s.hash(state),
            Decoded::Big(b) => b.hash(state),
        }
    }
}

impl<S> PartialEq for ArcEncoding<S>
where
    S: SmallNumber,
{
    fn eq(&self, other: &Self) -> bool {
        match (self.decode(), other.decode()) {
            (Decoded::Small(s1), Decoded::Small(s2)) => s1 == s2,
            (Decoded::Big(b1), Decoded::Big(b2)) => b1 == b2,
            _ => false,
        }
    }
}

impl<S> Eq for ArcEncoding<S> where S: SmallNumber {}

#[cfg(any(test, feature = "quickcheck"))]
impl<S> quickcheck::Arbitrary for ArcEncoding<S>
where
    S: SmallNumber,
{
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if bool::arbitrary(g) {
            Self::from_small(<S as quickcheck::Arbitrary>::arbitrary(g) >> 1u32)
        } else {
            Self::from_big(S::Big::arbitrary(g))
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<S> arbitrary::Arbitrary<'_> for ArcEncoding<S>
where
    S: SmallNumber,
{
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(if bool::arbitrary(u)? {
            Self::from_small(<S as arbitrary::Arbitrary>::arbitrary(u)? >> 1u32)
        } else {
            Self::from_big(S::Big::arbitrary(u)?)
        })
    }
}
