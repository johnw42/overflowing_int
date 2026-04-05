use crate::{rc_bignum::shifted, small_num::SmallNumber};

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
    pub const fn try_new(s: S) -> Option<Self> {
        let shifted = s << 1;
        let unshifted = shifted >> 1;
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
            Some(self.0 >> 1)
        } else {
            None
        }
    }
}
