/// Checked and pseudo-checked operations with the same names as the regular trait methods.
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedSub};
use std::mem::size_of;
use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr};

pub fn add<T: CheckedAdd>(lhs: T, rhs: T) -> Option<T> {
    lhs.checked_add(&rhs)
}
pub fn sub<T: CheckedSub>(lhs: T, rhs: T) -> Option<T> {
    lhs.checked_sub(&rhs)
}
pub fn mul<T: CheckedMul>(lhs: T, rhs: T) -> Option<T> {
    lhs.checked_mul(&rhs)
}
pub fn div<T: CheckedDiv>(lhs: T, rhs: T) -> Option<T> {
    lhs.checked_div(&rhs)
}
pub fn rem<T: CheckedRem>(lhs: T, rhs: T) -> Option<T> {
    lhs.checked_rem(&rhs)
}
pub fn bitand<T: BitAnd<Output = T>>(lhs: T, rhs: T) -> Option<T> {
    Some(lhs & rhs)
}
pub fn bitor<T: BitOr<Output = T>>(lhs: T, rhs: T) -> Option<T> {
    Some(lhs | rhs)
}
pub fn bitxor<T: BitXor<Output = T>>(lhs: T, rhs: T) -> Option<T> {
    Some(lhs ^ rhs)
}

pub fn shl<T>(lhs: T, rhs: u32) -> Option<T>
where
    T: Shl<u32, Output = T>,
    T: Shr<u32, Output = T>,
    T: Eq,
    T: Copy,
{
    if (rhs as usize) <= 8 * size_of::<T>() {
        let shifted = lhs << rhs;
        let unshifted = shifted >> rhs;
        if unshifted == lhs {
            Some(shifted)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn shr<T>(lhs: T, rhs: u32) -> Option<T>
where
    T: Shr<u32, Output = T>,
{
    if (rhs as usize) <= 8 * size_of::<T>() {
        Some(lhs >> rhs)
    } else {
        None
    }
}
