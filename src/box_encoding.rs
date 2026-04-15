use crate::encoding::{Decode, Decoded, Encode, Encoding};
use crate::shifted::Shifted;
use crate::small_num::SmallNumber;
use num_bigint::{BigInt, BigUint};
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::{borrow::Cow, fmt::Debug};

const _: () = {
    assert!(align_of::<Box<BigInt>>() > 1);
    assert!(align_of::<Box<BigUint>>() > 1);
    assert!(size_of::<Box<BigInt>>() == size_of::<isize>());
    assert!(size_of::<Box<BigUint>>() == size_of::<usize>());
};

/// An encoding that uses `Box` for big values, and a small value with the LSB
/// set to 1 for small values.  This encoding is used for `BoxBigInt` and
/// `BoxBigUint`.
#[derive(Clone)]
pub struct BoxEncoding<'a, S>(BoxEncodedRepr<S>, PhantomData<&'a ()>)
where
    S: SmallNumber;

impl<'a, S> BoxEncoding<'a, S>
where
    S: SmallNumber,
{
    #[allow(unused)]
    fn from_shifted(shifted: Shifted<S>) -> Self {
        Self(BoxEncodedRepr { small: shifted }, PhantomData)
    }
}

impl<'a, S> Decode<'a, S> for BoxEncoding<'a, S>
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
                Decoded::Big(Cow::Owned(*taken))
            }
        }
    }

    fn decode<'b>(&'b self) -> Decoded<S, Cow<'b, <S as SmallNumber>::Big>> {
        unsafe {
            if let Some(s) = self.0.small.validate() {
                Decoded::Small(s)
            } else {
                Decoded::Big(Cow::Borrowed(Box::as_ref(&self.0.big)))
            }
        }
    }
}

impl<'a, S> Encode<'a, S> for BoxEncoding<'a, S>
where
    S: SmallNumber,
{
    fn from_small(s: S) -> Self {
        if let Some(shifted) = Shifted::try_new(s) {
            Self(BoxEncodedRepr { small: shifted }, PhantomData)
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
                BoxEncodedRepr { small: shifted }
            } else {
                BoxEncodedRepr {
                    big: ManuallyDrop::new(Box::new(b.into_owned())),
                }
            },
            PhantomData,
        )
    }
}

impl<'a, S> Encoding<'a> for BoxEncoding<'a, S>
where
    S: SmallNumber,
{
    type Small = S;
    type Big = S::Big;
    type Unsigned = BoxEncoding<'a, S::Unsigned>;
    type Static = BoxEncoding<'static, S>;
    type WithLifetime<'b>
        = BoxEncoding<'b, S>
    where
        'a: 'b;

    const ZERO: Self = Self(
        BoxEncodedRepr {
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
                BoxEncoding(
                    BoxEncodedRepr {
                        small: self.0.small,
                    },
                    PhantomData,
                )
            } else {
                BoxEncoding(
                    BoxEncodedRepr {
                        big: ManuallyDrop::new(Box::clone(&self.0.big)),
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
                Decoded::Big(Cow::Borrowed(Box::as_ref(&self.0.big)))
            }
        };
        f(&mut decoded);
        *self = match decoded {
            Decoded::Small(s) => Self::from_small(s),
            Decoded::Big(b) => Self::from_big(b.into_owned()),
        };
    }

    fn into_static(self) -> Self::Static {
        BoxEncoding(self.0, PhantomData)
    }
}

union BoxEncodedRepr<S: SmallNumber> {
    small: Shifted<S>,
    big: ManuallyDrop<Box<S::Big>>,
}

impl<S> Clone for BoxEncodedRepr<S>
where
    S: SmallNumber,
{
    fn clone(&self) -> Self {
        unsafe {
            if self.small.validate().is_some() {
                BoxEncodedRepr { small: self.small }
            } else {
                BoxEncodedRepr {
                    big: ManuallyDrop::new(Box::clone(&self.big)),
                }
            }
        }
    }
}

impl<S> Drop for BoxEncodedRepr<S>
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

impl<'a, S> Debug for BoxEncoding<'a, S>
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

impl<'a, S> Hash for BoxEncoding<'a, S>
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

impl<'a, S> PartialEq for BoxEncoding<'a, S>
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

impl<'a, S> Eq for BoxEncoding<'a, S> where S: SmallNumber {}

#[cfg(any(test, feature = "quickcheck"))]
impl<S: SmallNumber> quickcheck::Arbitrary for BoxEncoding<'static, S> {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if bool::arbitrary(g) {
            Self::from_shifted(Shifted::<S>::arbitrary(g))
        } else {
            Self::from_big(S::Big::arbitrary(g))
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, S: SmallNumber> arbitrary::Arbitrary<'a> for BoxEncoding<'a, S> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(if bool::arbitrary(u)? {
            Self::from_shifted(Shifted::<S>::arbitrary(u)?)
        } else {
            Self::from_big(S::Big::arbitrary(u)?)
        })
    }
}
