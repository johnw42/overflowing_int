/// Checked and pseudo-checked operations with the same names as the regular trait methods.
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedShl, CheckedShr, CheckedSub,
};
use std::ops::{BitAnd, BitOr, BitXor};

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
pub fn shl<T: CheckedShl>(lhs: T, rhs: u32) -> Option<T> {
    lhs.checked_shl(rhs)
}
pub fn shr<T: CheckedShr>(lhs: T, rhs: u32) -> Option<T> {
    lhs.checked_shr(rhs)
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
