use std::hash::Hash;
use std::mem::ManuallyDrop;
use std::rc::Rc;
use std::{borrow::Cow, fmt::Debug};

use num_bigint::{BigInt, BigUint};
use serde::de;

use crate::big_number::BigNumber;
use crate::generic_bignum::encoding::{Decoded, EncodedBigNum, InspectEncoding};
use crate::rc_encoding::shifted::Shifted;
use crate::small_num::SmallNumber;

mod shifted;

#[derive(Clone)]
pub struct RcEncoding<S, B>(RcEncodedRepr<S, B>)
where
    S: SmallNumber<Big = B>,
    B: BigNumber;

impl<S, B> InspectEncoding<'static, S, B> for RcEncoding<S, B>
where
    S: SmallNumber<Big = B>,
    B: BigNumber,
{
    fn decode(mut self) -> Decoded<S, Cow<'static, B>> {
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

    fn with_decoded_ref<T>(&self, f: impl FnOnce(Decoded<S, Cow<B>>) -> T) -> T {
        unsafe {
            if let Some(s) = self.0.small.validate() {
                f(Decoded::Small(s))
            } else {
                f(Decoded::Big(Cow::Borrowed(&*self.0.big)))
            }
        }
    }
}

impl<S, B> EncodedBigNum<'static> for RcEncoding<S, B>
where
    S: SmallNumber<Big = B>,
    B: BigNumber,
    BigInt: From<B>,
{
    type Small = S;
    type Big = B;

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
        Self(RcEncodedRepr {
            big: ManuallyDrop::new(Rc::new(b.into_owned())),
        })
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

union RcEncodedRepr<S: SmallNumber, T> {
    small: Shifted<S>,
    big: ManuallyDrop<Rc<T>>,
}

const _: () = {
    assert!(align_of::<Rc<BigInt>>() > 1);
    assert!(align_of::<Rc<BigUint>>() > 1);
    assert!(size_of::<Rc<BigInt>>() == size_of::<isize>());
    assert!(size_of::<Rc<BigUint>>() == size_of::<usize>());
};

impl<S, T> Clone for RcEncodedRepr<S, T>
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

impl<S, B> Drop for RcEncodedRepr<S, B>
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

impl<S, B> Debug for RcEncoding<S, B>
where
    S: SmallNumber<Big = B>,
    B: BigNumber,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.with_decoded_ref(|decoded| match decoded {
            Decoded::Small(s) => write!(f, "Small({})", s),
            Decoded::Big(b) => write!(f, "Big({})", b.as_ref()),
        })
    }
}

impl<S, B> Hash for RcEncoding<S, B>
where
    S: SmallNumber<Big = B>,
    B: BigNumber,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.with_decoded_ref(|decoded| match decoded {
            Decoded::Small(s) => s.hash(state),
            Decoded::Big(b) => b.hash(state),
        });
    }
}

impl<S, B> PartialEq for RcEncoding<S, B>
where
    S: SmallNumber<Big = B>,
    B: BigNumber,
{
    fn eq(&self, other: &Self) -> bool {
        self.with_decoded_ref(|lhs| {
            other.with_decoded_ref(|rhs| match (lhs, rhs) {
                (Decoded::Small(s1), Decoded::Small(s2)) => s1 == s2,
                (Decoded::Big(b1), Decoded::Big(b2)) => b1 == b2,
                _ => false,
            })
        })
    }
}

impl<S, B> Eq for RcEncoding<S, B>
where
    S: SmallNumber<Big = B>,
    B: BigNumber,
{
}
