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
    pub const ZERO: Self = Self(S::ONE);

    /// Creates a new `Shifted` value from a small number, if it can be represented as such.
    pub fn try_new(s: S) -> Option<Self> {
        let shifted = s << 1u32;
        let unshifted = shifted >> 1u32;
        if unshifted == s {
            Some(Self(shifted | S::ONE))
        } else {
            None
        }
    }

    /// Validates that the value is a valid `Shifted` value, and returns the
    /// original small number if it is.  The only way a shifted number can be
    /// invalid is through the use of unsafe operations.
    pub fn validate(self) -> Option<S> {
        if self.0 & S::ONE == S::ONE {
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
        Self::ZERO
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
