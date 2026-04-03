use std::{
    fmt::{Debug, Display},
    ops::{Add, BitAnd, BitOr, Shl, Shr},
};

use num_bigint::{BigInt, BigUint};
use num_integer::Roots;
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedSub, Num, One, PrimInt, ToPrimitive,
    Unsigned, Zero,
};
use paste::paste;

use crate::{BigNumber, duplicate_arith_ops};

/// A trait implemented by primitive integer types that can be used as the
/// "small" part of a big integer encoding.
pub trait SmallNumber:
    Copy
    + Debug
    + Display
    + Eq
    + Num
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + CheckedAdd
    + CheckedDiv
    + CheckedMul
    + CheckedRem
    + CheckedSub
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + From<u8>
    + From<bool>
    + Into<BigInt>
    + One
    + Ord
    + PrimInt
    + Roots
    + ToPrimitive
    + TryFrom<i8>
    + TryFrom<i16>
    + TryFrom<i32>
    + TryFrom<i64>
    + TryFrom<i128>
    + TryFrom<isize>
    + TryFrom<u8>
    + TryFrom<u16>
    + TryFrom<u32>
    + TryFrom<u64>
    + TryFrom<u128>
    + TryFrom<usize>
    + TryFrom<Self::Unsigned>
    + TryInto<BigUint>
    + Zero
where
    for<'a> &'a Self: Add<&'a Self, Output = Self>,
    for<'a> &'a Self: Add<&'a u128, Output = Self>,
    for<'a> &'a Self: Add<&'a u16, Output = Self>,
    for<'a> &'a Self: Add<&'a u32, Output = Self>,
    for<'a> &'a Self: Add<&'a u64, Output = Self>,
    for<'a> &'a Self: Add<&'a u8, Output = Self>,
    for<'a> &'a Self: Add<&'a usize, Output = Self>,
    for<'a> &'a Self: Add<Self, Output = Self>,
    for<'a> &'a Self: Add<u128, Output = Self>,
    for<'a> &'a Self: Add<u16, Output = Self>,
    for<'a> &'a Self: Add<u32, Output = Self>,
    for<'a> &'a Self: Add<u64, Output = Self>,
    for<'a> &'a Self: Add<u8, Output = Self>,
{
    type Big: BigNumber;
    type Unsigned: SmallNumber + Unsigned;

    const BITS: u32;

    fn try_from_unsigned(u: Self::Unsigned) -> Option<Self> {
        Self::try_from(u).ok()
    }

    /// Calls the primitive's `unsigned_abs` method, which returns the unsigned
    /// absolute value of the number.  For unsigned types, this is just the
    /// identity function.
    fn unsigned_abs(self) -> Self::Unsigned;

    /// Calls the primitive's `overflowing_pow` method.
    fn overflowing_pow(self, exp: u32) -> (Self, bool);

    fn from_bytes_be(bytes: &[u8]) -> Option<Self> {
        // Ideally this would be implemented using `from_be_bytes` on the
        // primitive types, but that requires an array of a specific size, and
        // we need to support any size up to the size of `Self`.
        //
        // It would be possible to copy the bytes into a fixed-size array and
        // then call `from_be_bytes`, but with the current Rust compiler (1.95),
        // it's not possible to create an array of size `size_of::<Self>()`.
        if size_of::<Self>() < bytes.len() {
            return None;
        }
        let mut result = Self::zero();
        for &byte in bytes {
            result = (result << 8u32) | <Self as From<u8>>::from(byte);
        }
        Some(result)
    }

    fn from_bytes_le(bytes: &[u8]) -> Option<Self> {
        // See note in `from_bytes_be`.
        if size_of::<Self>() < bytes.len() {
            return None;
        }
        let mut result = Self::zero();
        for &byte in bytes.iter().rev() {
            result = (result << 8u32) | <Self as From<u8>>::from(byte);
        }
        Some(result)
    }

    duplicate_arith_ops!(paste! {
        fn [<op_fn _bigint_left>](lhs: Self, rhs: Self::Big) -> Self::Big;
        fn [<op_fn _bigint_right>](lhs: Self::Big, rhs: Self) -> Self::Big;
        fn [<op_fn _bigint_ref_left>](lhs: Self, rhs: &Self::Big) -> Self::Big;
        fn [<op_fn _bigint_ref_right>](lhs: &Self::Big, rhs: Self) -> Self::Big;
    });
}

macro_rules! impl_arith_ops {
    () => {
        duplicate_arith_ops!(paste! {
            fn [<op_fn _bigint_left>](lhs: Self, rhs: Self::Big) -> Self::Big {
                std::ops::op_trait::op_fn(lhs, rhs)
            }
            fn [<op_fn _bigint_right>](lhs: Self::Big, rhs: Self) -> Self::Big {
                std::ops::op_trait::op_fn(lhs, rhs)
            }
            fn [<op_fn _bigint_ref_left>](lhs: Self, rhs: &Self::Big) -> Self::Big {
                std::ops::op_trait::op_fn(lhs, rhs)
            }
            fn [<op_fn _bigint_ref_right>](lhs: &Self::Big, rhs: Self) -> Self::Big {
                std::ops::op_trait::op_fn(lhs, rhs)
            }
        });
    };
}

macro_rules! impl_small_num {
    ($signed:ty, $unsigned:ty) => {
        impl SmallNumber for $signed {
            type Big = BigInt;
            type Unsigned = $unsigned;
            const BITS: u32 = <$signed>::BITS;

            fn unsigned_abs(self) -> Self::Unsigned {
                self.unsigned_abs()
            }

            fn overflowing_pow(self, exp: u32) -> (Self, bool) {
                self.overflowing_pow(exp)
            }

            impl_arith_ops!();
        }

        impl SmallNumber for $unsigned {
            type Big = BigUint;
            type Unsigned = $unsigned;
            const BITS: u32 = <$unsigned>::BITS;

            fn unsigned_abs(self) -> Self::Unsigned {
                self
            }

            fn overflowing_pow(self, exp: u32) -> (Self, bool) {
                self.overflowing_pow(exp)
            }

            impl_arith_ops!();
        }
    };
}

impl_small_num!(i128, u128);
impl_small_num!(isize, usize);

#[test]
fn test_bytes_to_uint_be() {
    assert_eq!(SmallNumber::from_bytes_be(&[0x00, 0x01]), Some(0x01usize));
    assert_eq!(SmallNumber::from_bytes_be(&[0x01, 0x00]), Some(0x0100usize));
    assert_eq!(SmallNumber::from_bytes_be(&[0x12, 0x34]), Some(0x1234usize));
    assert_eq!(
        SmallNumber::from_bytes_be(&[0xFF; size_of::<usize>()]),
        Some(usize::MAX)
    );
    assert_eq!(
        SmallNumber::from_bytes_be(&[0xFF; size_of::<usize>() + 1]),
        None::<usize>
    );
}

#[test]
fn test_bytes_to_uint_le() {
    assert_eq!(SmallNumber::from_bytes_le(&[0x01, 0x00]), Some(0x01usize));
    assert_eq!(SmallNumber::from_bytes_le(&[0x00, 0x01]), Some(0x0100usize));
    assert_eq!(SmallNumber::from_bytes_le(&[0x34, 0x12]), Some(0x1234usize));
    assert_eq!(
        SmallNumber::from_bytes_le(&[0xFF; size_of::<usize>()]),
        Some(usize::MAX)
    );
    assert_eq!(
        SmallNumber::from_bytes_le(&[0xFF; size_of::<usize>() + 1]),
        None::<usize>
    );
}
