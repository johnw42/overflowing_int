use std::fmt::Debug;
use std::hash::Hash;
use std::mem::ManuallyDrop;
use std::rc::Rc;

use num_bigint::{BigInt, BigUint};

use crate::rc_bigint::shifted::Shifted;
use crate::rc_bigint::small_num::{SmallInt, SmallNum, SmallUint};

/// A wrapper type around `Encoding` that maintains the the invariant that
/// values that can be represented as `SmallInt` or `SmallUint` are always
/// stored as such, and only values that cannot be represented as `SmallInt` or
/// `SmallUint` are stored as `BigInt` or `BigUint`.  This type, in turn, is the
/// content of `CBigInt` and `CBigUint`, which implement high-level operations
/// and traits.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Encoded<S, T>(EncodedRepr<S, T>)
where
    S: SmallNum;

impl<S, T> Encoded<S, T>
where
    S: SmallNum,
    T: From<S>,
{
    pub const fn from_small(s: S) -> Self {
        Self(EncodedRepr {
            small: Shifted::try_new(s).unwrap(),
        })
    }

    pub fn decode_ref(&self) -> RefEncoding<S, T> {
        unsafe {
            if let Some(small) = self.0.small.validate() {
                RefEncoding::Small(small)
            } else {
                RefEncoding::Big(&self.0.big)
            }
        }
    }

    pub fn decode(self) -> Encoding<S, T> {
        self.0.clone().into()
    }

    pub fn small(&self) -> Option<S> {
        self.0.small()
    }

    pub fn big_ref(&self) -> Option<&T> {
        self.0.big()
    }
}

impl<S, T> From<EncodedRepr<S, T>> for Encoded<S, T>
where
    S: SmallNum,
{
    fn from(value: EncodedRepr<S, T>) -> Self {
        Encoded(value)
    }
}

impl<S, T> From<Encoding<S, T>> for Encoded<S, T>
where
    S: SmallNum,
    T: From<S>,
{
    fn from(value: Encoding<S, T>) -> Self {
        Encoded(value.into())
    }
}

pub enum Encoding<S, T> {
    Small(S),
    Big(Rc<T>),
}

pub enum RefEncoding<'a, S, T> {
    Small(S),
    Big(&'a T),
}

union EncodedRepr<S: SmallNum, T> {
    small: Shifted<S>,
    big: ManuallyDrop<Rc<T>>,
}

const _: () = {
    assert!(align_of::<Rc<BigInt>>() > 1);
    assert!(align_of::<Rc<BigUint>>() > 1);
    assert!(size_of::<Rc<BigInt>>() == size_of::<SmallInt>());
    assert!(size_of::<Rc<BigUint>>() == size_of::<SmallUint>());
};

impl<S, T> EncodedRepr<S, T>
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

impl<S, T> Clone for EncodedRepr<S, T>
where
    S: SmallNum,
{
    fn clone(&self) -> Self {
        unsafe {
            if self.small.validate().is_some() {
                EncodedRepr { small: self.small }
            } else {
                EncodedRepr {
                    big: ManuallyDrop::new(Rc::clone(&self.big)),
                }
            }
        }
    }
}

impl<S, T> Drop for EncodedRepr<S, T>
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

impl<S, T> From<EncodedRepr<S, T>> for Encoding<S, T>
where
    S: SmallNum,
{
    fn from(mut value: EncodedRepr<S, T>) -> Self {
        unsafe {
            if let Some(small) = value.small.validate() {
                Encoding::Small(small)
            } else {
                Encoding::Big(ManuallyDrop::take(&mut value.big))
            }
        }
    }
}

impl<S, T> From<Encoding<S, T>> for EncodedRepr<S, T>
where
    S: SmallNum,
    T: From<S>,
{
    fn from(value: Encoding<S, T>) -> Self {
        match value {
            Encoding::Small(s) => {
                if let Some(small) = Shifted::try_new(s) {
                    EncodedRepr { small }
                } else {
                    EncodedRepr {
                        big: ManuallyDrop::new(Rc::new(T::from(s))),
                    }
                }
            }
            Encoding::Big(b) => EncodedRepr {
                big: ManuallyDrop::new(b),
            },
        }
    }
}

impl<S, T> Debug for EncodedRepr<S, T>
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

impl<S, T> Hash for EncodedRepr<S, T>
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

impl<S, T> PartialEq for EncodedRepr<S, T>
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

impl<S, T> Eq for EncodedRepr<S, T>
where
    S: SmallNum + Eq,
    T: Eq,
{
}
