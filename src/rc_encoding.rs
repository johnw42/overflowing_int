use crate::encoding::{Decode, Decoded, Encode, Encoding};
use crate::rc_encoding::shifted::Shifted;
use crate::small_num::SmallNumber;
use num_bigint::{BigInt, BigUint};
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::rc::Rc;
use std::{borrow::Cow, fmt::Debug};

const _: () = {
    assert!(align_of::<Rc<BigInt>>() > 1);
    assert!(align_of::<Rc<BigUint>>() > 1);
    assert!(size_of::<Rc<BigInt>>() == size_of::<isize>());
    assert!(size_of::<Rc<BigUint>>() == size_of::<usize>());
};

mod shifted {
    use crate::small_num::SmallNumber;

    // A number that is stored shifted left by one bit, with the least significant
    // bit set to 1.  This allows us to distinguish between small numbers (which
    // have the least significant bit set to 1) and pointers to big numbers (which
    // have the least significant bit set to 0).  This is used in `RcEncoded` to
    // store small numbers without heap allocation, while still allowing us to store
    // big numbers on the heap and reference them with a pointer.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Shifted<S>(S);

    impl<S> Shifted<S>
    where
        S: SmallNumber,
    {
        /// Creates a new `Shifted` value from a small number, if it can be represented as such.
        pub fn try_new(s: S) -> Option<Self> {
            let shifted = s << 1u32;
            let unshifted = shifted >> 1u32;
            if unshifted == s {
                Some(Self(shifted | S::one()))
            } else {
                None
            }
        }

        /// Validates that the value is a valid `Shifted` value, and returns the
        /// original small number if it is.  The only way a shifted number can be
        /// invalid is through the use of unsafe operations.
        pub fn validate(self) -> Option<S> {
            if self.0 & S::one() == S::one() {
                Some(self.0 >> 1u32)
            } else {
                None
            }
        }
    }

    impl<S> Default for Shifted<S>
    where
        S: SmallNumber,
    {
        fn default() -> Self {
            Self(S::one())
        }
    }

    #[cfg(any(test, feature = "quickcheck"))]
    impl<S> quickcheck::Arbitrary for Shifted<S>
    where
        S: SmallNumber,
    {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Shifted(<S as quickcheck::Arbitrary>::arbitrary(g) >> 1u32)
        }
    }

    #[cfg(feature = "arbitrary")]
    impl<S> arbitrary::Arbitrary<'_> for Shifted<S>
    where
        S: SmallNumber,
    {
        fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
            Ok(Shifted(<S as arbitrary::Arbitrary>::arbitrary(u)? >> 1u32))
        }
    }
}

/// An encoding that uses `Rc` for big values, and a small value with the LSB
/// set to 1 for small values.  This encoding is used for `RcBigInt` and
/// `RcBigUint`.
#[derive(Clone)]
pub struct RcEncoding<S>(RcEncodedRepr<S>, PhantomData<()>)
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

    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T {
        unsafe {
            if let Some(s) = self.0.small.validate() {
                f(Decoded::Small(s))
            } else {
                f(Decoded::Big(Cow::Borrowed(&*self.0.big)))
            }
        }
    }

    fn owns_bignum(&self) -> bool {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(_) => false,
            Decoded::Big(_) => true,
        })
    }
}

impl<S> Encode<'static, S> for RcEncoding<S>
where
    S: SmallNumber,
{
    fn from_small(s: S) -> Self {
        if let Some(shifted) = Shifted::try_new(s) {
            Self(RcEncodedRepr { small: shifted }, PhantomData)
        } else {
            let r = Self::from_big(s.to_big());
            unsafe {
                debug_assert!(r.0.small.validate().is_none());
            }
            r
        }
    }

    fn from_big_cow(b: Cow<'static, S::Big>) -> Self {
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
            PhantomData,
        )
    }
}

impl<S> Encoding<'static> for RcEncoding<S>
where
    S: SmallNumber<Unsigned = usize>,
{
    type Small = S;
    type Big = S::Big;
    type Unsigned = RcEncoding<S::Unsigned>;
    type Static = RcEncoding<S>;

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

    fn into_static(self) -> Self::Static {
        RcEncoding(self.0, PhantomData)
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

#[cfg(any(test, feature = "quickcheck"))]
impl<S: SmallNumber> quickcheck::Arbitrary for RcEncoding<S> {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if bool::arbitrary(g) {
            Self(
                RcEncodedRepr {
                    small: Shifted::<S>::arbitrary(g),
                },
                PhantomData,
            )
        } else {
            Self(
                RcEncodedRepr {
                    big: ManuallyDrop::new(Rc::new(S::Big::arbitrary(g))),
                },
                PhantomData,
            )
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<S: SmallNumber> arbitrary::Arbitrary<'_> for RcEncoding<S> {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok(
            self,
            if bool::arbitrary(u)? {
                RcEncodedRepr {
                    small: <Shifted<S> as arbitrary::Arbitrary>::arbitrary(u)?,
                }
            } else {
                RcEncodedRepr {
                    big: ManuallyDrop::new(Rc::new(S::Big::arbitrary(u)?)),
                }
            },
            PhantomData,
        )
    }
}
