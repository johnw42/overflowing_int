use std::hash::Hash;
use std::mem::ManuallyDrop;
use std::rc::Rc;
use std::{borrow::Cow, fmt::Debug};

use crate::rc_bignum::shifted::Shifted;

/// A wrapper type around `Encoding` that maintains the the invariant that
/// values that can be represented as `SmallInt` or `SmallUint` are always
/// stored as such, and only values that cannot be represented as `SmallInt` or
/// `SmallUint` are stored as `BigInt` or `BigUint`.  This type, in turn, is the
/// content of `CBigInt` and `CBigUint`, which implement high-level operations
/// and traits.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RcEncoded<S, T>(RcEncodedRepr<S, T>)
where
    S: SmallNum;

impl<S, T> EncodedBigNum<'static> for RcEncoded<S, T>
where
    S: SmallNum,
    T: BigNumber,
{
    type Small = S;
    type Big = T;
    type BigEncoding = Rc<T>;

    fn from_small(s: S) -> Self {
        Self(RcEncodedRepr {
            small: Shifted::try_new(s).unwrap(),
        })
    }

    fn decode(self) -> Encoding<Self::Small, Self::BigEncoding> {
        self.0.clone().into()
    }

    fn decode_ref(&self) -> Encoding<Self::Small, &Self::Big> {
        if let Some(small) = self.0.small.validate() {
            Encoding::Small(small)
        } else {
            Encoding::Big(&self.0.big)
        }
    }

    fn small(&self) -> Option<<Self as EncodedBigNum>::Small> {
        self.0.small()
    }

    fn big_ref(&self) -> Option<&<Self as EncodedBigNum>::Big> {
        self.0.big()
    }

    fn big_cow<'b>(&'b self) -> Cow<'b, <Self as EncodedBigNum>::Big> {
        if let Some(small) = self.0.small() {
            Cow::Owned(<Self as EncodedBigNum>::Big::from(small))
        } else {
            Cow::Borrowed(self.0.big())
        }
    }
}

union RcEncodedRepr<S: SmallNum, T> {
    small: Shifted<S>,
    big: ManuallyDrop<Rc<T>>,
}

const _: () = {
    assert!(align_of::<Rc<BigInt>>() > 1);
    assert!(align_of::<Rc<BigUint>>() > 1);
    assert!(size_of::<Rc<BigInt>>() == size_of::<SmallInt>());
    assert!(size_of::<Rc<BigUint>>() == size_of::<SmallUint>());
};

impl<S, T> RcEncodedRepr<S, T>
where
    S: SmallNum,
{
    pub fn small(&self) -> Option<S> {
        unsafe { self.small.validate() }
    }

    pub fn big(&self) -> Option<&T> {
        unsafe {
            if self.small.validate().is_none() {
                Some(&*self.big)
            } else {
                None
            }
        }
    }
}

impl<S, T> Clone for RcEncodedRepr<S, T>
where
    S: SmallNum,
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

impl<S, T> Drop for RcEncodedRepr<S, T>
where
    S: SmallNum,
{
    fn drop(&mut self) {
        unsafe {
            if self.small.validate().is_none() {
                ManuallyDrop::drop(&mut self.big);
            }
        }
    }
}

impl<S, T> From<RcEncodedRepr<S, T>> for Encoding<S, T>
where
    S: SmallNum,
{
    fn from(mut value: RcEncodedRepr<S, T>) -> Self {
        unsafe {
            if let Some(small) = value.small.validate() {
                Encoding::Small(small)
            } else {
                Encoding::Big(ManuallyDrop::take(&mut value.big))
            }
        }
    }
}

impl<S, T> From<Encoding<S, T>> for RcEncodedRepr<S, T>
where
    S: SmallNum,
    T: From<S>,
{
    fn from(value: Encoding<S, T>) -> Self {
        match value {
            Encoding::Small(s) => {
                if let Some(small) = Shifted::try_new(s) {
                    RcEncodedRepr { small }
                } else {
                    RcEncodedRepr {
                        big: ManuallyDrop::new(Rc::new(T::from(s))),
                    }
                }
            }
            Encoding::Big(b) => RcEncodedRepr {
                big: ManuallyDrop::new(b),
            },
        }
    }
}

impl<S, T> Debug for RcEncodedRepr<S, T>
where
    S: SmallNum + Debug,
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.clone().into() {
            Encoding::Small(s) => s.fmt(f),
            Encoding::Big(b) => b.fmt(f),
        }
    }
}

impl<S, T> Hash for RcEncodedRepr<S, T>
where
    S: SmallNum + Hash,
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self.clone().into() {
            Encoding::Small(s) => s.hash(state),
            Encoding::Big(b) => b.hash(state),
        }
    }
}

impl<S, T> PartialEq for RcEncodedRepr<S, T>
where
    S: SmallNum + PartialEq,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self.clone().into(), other.clone().into()) {
            (Encoding::Small(s1), Encoding::Small(s2)) => s1 == s2,
            (Encoding::Big(b1), Encoding::Big(b2)) => b1 == b2,
            _ => false,
        }
    }
}

impl<S, T> Eq for RcEncodedRepr<S, T>
where
    S: SmallNum + Eq,
    T: Eq,
{
}
