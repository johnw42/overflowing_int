use crate::encoding::{Decode, Decoded, Encode, Encoding};
use crate::shifted::Shifted;
use crate::small_num::SmallNumber;
use num_bigint::{BigInt, BigUint};
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::sync::Arc;
use std::{borrow::Cow, fmt::Debug};

const _: () = {
    assert!(align_of::<Arc<BigInt>>() > 1);
    assert!(align_of::<Arc<BigUint>>() > 1);
    assert!(size_of::<Arc<BigInt>>() == size_of::<isize>());
    assert!(size_of::<Arc<BigUint>>() == size_of::<usize>());
};

/// An encoding that uses `Arc` for big values, and a small value with the LSB
/// set to 1 for small values.  This encoding is used for `ArcBigInt` and
/// `ArcBigUint`.
#[derive(Clone)]
pub struct ArcEncoding<'a, S>(ArcEncodedRepr<S>, PhantomData<&'a ()>)
where
    S: SmallNumber;

impl<'a, S> ArcEncoding<'a, S>
where
    S: SmallNumber,
{
    #[allow(unused)]
    fn from_shifted(shifted: Shifted<S>) -> Self {
        Self(ArcEncodedRepr { small: shifted }, PhantomData)
    }
}

impl<'a, S> Decode<'a, S> for ArcEncoding<'a, S>
where
    S: SmallNumber,
{
    fn into_decoded(mut self) -> Decoded<S, Cow<'static, S::Big>> {
        unsafe {
            if let Some(s) = self.0.small.validate() {
                Decoded::Small(s)
            } else {
                let taken = ManuallyDrop::take(&mut self.0.big);
                // Reset the encoding to a valid small so the Drop implementation doesn't try to drop the big value again.
                self.0.small = Shifted::default();
                Decoded::Big(Cow::Owned(Arc::unwrap_or_clone(taken)))
            }
        }
    }

    fn decode<'b>(&'b self) -> Decoded<S, Cow<'b, <S as SmallNumber>::Big>> {
        unsafe {
            if let Some(s) = self.0.small.validate() {
                Decoded::Small(s)
            } else {
                Decoded::Big(Cow::Borrowed(Arc::as_ref(&self.0.big)))
            }
        }
    }
}

impl<'a, S> Encode<'a, S> for ArcEncoding<'a, S>
where
    S: SmallNumber,
{
    fn from_small(s: S) -> Self {
        if let Some(shifted) = Shifted::try_new(s) {
            Self(ArcEncodedRepr { small: shifted }, PhantomData)
        } else {
            let r = Self::from_big(s.to_big());
            unsafe {
                debug_assert!(r.0.small.validate().is_none());
            }
            r
        }
    }

    fn from_big_cow(b: Cow<'a, S::Big>) -> Self {
        Self(
            if let Some(small) = S::try_from(b.as_ref()).ok()
                && let Some(shifted) = Shifted::try_new(small)
            {
                ArcEncodedRepr { small: shifted }
            } else {
                ArcEncodedRepr {
                    big: ManuallyDrop::new(Arc::new(b.into_owned())),
                }
            },
            PhantomData,
        )
    }
}

impl<'a, S> Encoding<'a> for ArcEncoding<'a, S>
where
    S: SmallNumber,
{
    type Small = S;
    type Big = S::Big;
    type Unsigned = ArcEncoding<'a, S::Unsigned>;
    type Static = ArcEncoding<'static, S>;
    type WithLifetime<'b>
        = ArcEncoding<'b, S>
    where
        Self: 'b,
        'a: 'b;

    const ZERO: Self = Self(
        ArcEncodedRepr {
            small: Shifted::ZERO,
        },
        PhantomData,
    );

    fn borrow<'b>(&'b self) -> Self::WithLifetime<'b>
    where
        Self: 'b,
        'a: 'b,
    {
        unsafe {
            if self.0.small.validate().is_some() {
                ArcEncoding(
                    ArcEncodedRepr {
                        small: self.0.small,
                    },
                    PhantomData,
                )
            } else {
                ArcEncoding(
                    ArcEncodedRepr {
                        big: ManuallyDrop::new(Arc::clone(&self.0.big)),
                    },
                    PhantomData,
                )
            }
        }
    }

    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<Self::Small, Cow<Self::Big>>)) {
        let mut decoded = unsafe {
            if let Some(s) = self.0.small.validate() {
                Decoded::Small(s)
            } else {
                Decoded::Big(Cow::Borrowed(Arc::as_ref(&self.0.big)))
            }
        };
        f(&mut decoded);
        *self = match decoded {
            Decoded::Small(s) => Self::from_small(s),
            Decoded::Big(b) => Self::from_big(b.into_owned()),
        };
    }

    fn into_static(self) -> Self::Static {
        ArcEncoding(self.0, PhantomData)
    }
}

union ArcEncodedRepr<S: SmallNumber> {
    small: Shifted<S>,
    big: ManuallyDrop<Arc<S::Big>>,
}

impl<S> Clone for ArcEncodedRepr<S>
where
    S: SmallNumber,
{
    fn clone(&self) -> Self {
        unsafe {
            if self.small.validate().is_some() {
                ArcEncodedRepr { small: self.small }
            } else {
                ArcEncodedRepr {
                    big: ManuallyDrop::new(Arc::clone(&self.big)),
                }
            }
        }
    }
}

impl<S> Drop for ArcEncodedRepr<S>
where
    S: SmallNumber,
{
    fn drop(&mut self) {
        unsafe {
            if self.small.validate().is_none() {
                ManuallyDrop::drop(&mut self.big);
            }
        }
    }
}

impl<'a, S> Debug for ArcEncoding<'a, S>
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

impl<'a, S> Hash for ArcEncoding<'a, S>
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

impl<'a, S> PartialEq for ArcEncoding<'a, S>
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

impl<'a, S> Eq for ArcEncoding<'a, S> where S: SmallNumber {}

#[cfg(any(test, feature = "quickcheck"))]
impl<S: SmallNumber> quickcheck::Arbitrary for ArcEncoding<'static, S> {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if bool::arbitrary(g) {
            Self::from_shifted(Shifted::<S>::arbitrary(g))
        } else {
            Self::from_big(S::Big::arbitrary(g))
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, S: SmallNumber> arbitrary::Arbitrary<'a> for ArcEncoding<'a, S> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(if bool::arbitrary(u)? {
            Self::from_shifted(Shifted::<S>::arbitrary(u)?)
        } else {
            Self::from_big(S::Big::arbitrary(u)?)
        })
    }
}
