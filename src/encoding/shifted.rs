use std::ops::{BitAnd, BitOr, Shl, Shr};

use num_traits::ConstOne;

/// Trait for types `T` that can be stored as a `Shifted<T>`.  Implemented for
/// all types that satisfy the necessary bounds.
pub trait Shiftable:
    Copy
    + ConstOne
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + PartialEq
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
{
}

impl<T> Shiftable for T where
    T: Copy
        + ConstOne
        + Shl<u32, Output = T>
        + Shr<u32, Output = T>
        + PartialEq
        + BitAnd<Output = T>
        + BitOr<Output = T>
{
}

// A number that is stored shifted left by one bit, with the least significant
// bit set to 1.  This allows us to distinguish between small numbers (which
// have the least significant bit set to 1) and pointers to big numbers (which
// have the least significant bit set to 0).  This is used in `RcEndcoding` to
// store small numbers without heap allocation, while still allowing us to store
// big numbers on the heap and reference them with a pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shifted<T>(T);

impl<T> Shifted<T>
where
    T: Shiftable,
{
    /// The zero value, which is store as the underlying types's `ONE` value.
    pub const ZERO: Self = Self(T::ONE);

    /// Creates a new `Shifted` value from a number, if it can be represented as such.
    pub fn new(s: T) -> Option<Self> {
        let shifted = s << 1u32;
        let unshifted = shifted >> 1u32;
        if unshifted == s {
            Some(Self(shifted | T::ONE))
        } else {
            None
        }
    }

    /// Validates that the value is a valid `Shifted` value, and returns the
    /// original number if it is.  The only way a shifted number can be
    /// invalid is through the use of unsafe operations.
    pub fn validate(self) -> Option<T> {
        if self.0 & T::ONE == T::ONE {
            Some(self.0 >> 1u32)
        } else {
            None
        }
    }
}

impl<T> Default for Shifted<T>
where
    T: Shiftable,
{
    fn default() -> Self {
        Self::ZERO
    }
}
