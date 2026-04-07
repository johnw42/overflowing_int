use crate::generic_bignum::encoding::{Decode, Decoded, Encoding};
use crate::rc_encoding::shifted::Shifted;
use crate::small_num::SmallNumber;
use num_bigint::{BigInt, BigUint};
use std::hash::Hash;
use std::mem::ManuallyDrop;
use std::rc::Rc;
use std::{borrow::Cow, fmt::Debug};

mod shifted;

const _: () = {
    assert!(align_of::<Rc<BigInt>>() > 1);
    assert!(align_of::<Rc<BigUint>>() > 1);
    assert!(size_of::<Rc<BigInt>>() == size_of::<isize>());
    assert!(size_of::<Rc<BigUint>>() == size_of::<usize>());
};

/// An encoding that uses `Rc` for big values, and a small value with the LSB
/// set to 1 for small values.  This encoding is used for `RcBigInt` and
/// `RcBigUint`.
#[derive(Clone)]
pub struct RcEncoding<S>(RcEncodedRepr<S>)
where
    S: SmallNumber;

impl<S> Decode<'static, S> for RcEncoding<S>
where
    S: SmallNumber,
{
    fn decode(mut self) -> Decoded<S, Cow<'static, S::Big>> {
        unsafe {
            if let Some(s) = self.0.small.validate() {
                Decoded::Small(s)
            } else {
                let taken = ManuallyDrop::take(&mut self.0.big);
                // Reset the encoding to a valid small so the Drop implementation doesn't try to drop the big value again.
                self.0.small = Shifted::default();
                Decoded::Big(Cow::Owned(Rc::unwrap_or_clone(taken)))
            }
        }
    }

    fn small(&self) -> Option<S> {
        unsafe { self.0.small.validate() }
    }

    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T {
        unsafe {
            if let Some(s) = self.0.small.validate() {
                f(Decoded::Small(s))
            } else {
                f(Decoded::Big(Cow::Borrowed(&*self.0.big)))
            }
        }
    }
}

impl<S> Encoding<'static> for RcEncoding<S>
where
    S: SmallNumber,
{
    type Small = S;
    type Big = S::Big;
    type Unsigned = RcEncoding<S::Unsigned>;

    fn from_small(s: S) -> Self {
        if let Some(shifted) = Shifted::try_new(s) {
            Self(RcEncodedRepr { small: shifted })
        } else {
            let r = Self::from_big(s.to_big());
            unsafe {
                debug_assert!(r.0.small.validate().is_none());
            }
            r
        }
    }

    fn from_big_cow(b: Cow<'static, Self::Big>) -> Self {
        Self(
            if let Some(small) = S::try_from(b.as_ref()).ok()
                && let Some(shifted) = Shifted::try_new(small)
            {
                RcEncodedRepr { small: shifted }
            } else {
                RcEncodedRepr {
                    big: ManuallyDrop::new(Rc::new(b.into_owned())),
                }
            },
        )
    }

    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<Self::Small, Cow<Self::Big>>)) {
        let mut decoded = unsafe {
            if let Some(s) = self.0.small.validate() {
                Decoded::Small(s)
            } else {
                Decoded::Big(Cow::Borrowed(Rc::as_ref(&self.0.big)))
            }
        };
        f(&mut decoded);
        *self = match decoded {
            Decoded::Small(s) => Self::from_small(s),
            Decoded::Big(b) => Self::from_big(b.into_owned()),
        };
    }
}

union RcEncodedRepr<S: SmallNumber> {
    small: Shifted<S>,
    big: ManuallyDrop<Rc<S::Big>>,
}

impl<S> Clone for RcEncodedRepr<S>
where
    S: SmallNumber,
{
    fn clone(&self) -> Self {
        unsafe {
            if self.small.validate().is_some() {
                RcEncodedRepr { small: self.small }
            } else {
                RcEncodedRepr {
                    big: ManuallyDrop::new(Rc::clone(&self.big)),
                }
            }
        }
    }
}

impl<S> Drop for RcEncodedRepr<S>
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

impl<S> Debug for RcEncoding<S>
where
    S: SmallNumber,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => write!(f, "Small({})", s),
            Decoded::Big(b) => write!(f, "Big({})", b.as_ref()),
        })
    }
}

impl<S> Hash for RcEncoding<S>
where
    S: SmallNumber,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => s.hash(state),
            Decoded::Big(b) => b.hash(state),
        });
    }
}

impl<S> PartialEq for RcEncoding<S>
where
    S: SmallNumber,
{
    fn eq(&self, other: &Self) -> bool {
        self.with_decoded(|lhs| {
            other.with_decoded(|rhs| match (lhs, rhs) {
                (Decoded::Small(s1), Decoded::Small(s2)) => s1 == s2,
                (Decoded::Big(b1), Decoded::Big(b2)) => b1 == b2,
                _ => false,
            })
        })
    }
}

impl<S> Eq for RcEncoding<S> where S: SmallNumber {}
