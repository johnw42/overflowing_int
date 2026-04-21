use std::{
    fmt::{Binary, Debug, Display, LowerHex, Octal, UpperHex},
    hash::Hash,
    ops::{BitAnd, BitOr, Shl, Shr},
};

use num_bigint::{BigInt, BigUint, ToBigUint};
use num_integer::Roots;
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedSub, ConstOne, ConstZero, Num, One,
    PrimInt, ToBytes, ToPrimitive, Unsigned, Zero,
};
use paste::paste;

use crate::{
    big_number::BigNumber,
    bounds::{ArbitraryBounds, QuickcheckBounds},
    duplicate_arith_ops,
};

/// A trait implemented by primitive integer types that can be used as the
/// "small" part of a big integer encoding.
pub trait SmallNumber:
    Copy
    + Debug
    + Display
    + Eq
    + Num
    + ArbitraryBounds
    + Binary
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + CheckedAdd
    + CheckedDiv
    + CheckedMul
    + CheckedRem
    + CheckedSub
    + ConstOne
    + ConstZero
    + LowerHex
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + From<u8>
    + From<bool>
    + Hash
    + Into<BigInt>
    + Into<Self::Big>
    + Octal
    + One
    + Ord
    + PrimInt
    + Roots
    + ToBigUint
    + ToBytes
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
    + TryFrom<Self::Wide>
    + for<'a> TryFrom<&'a Self::Big>
    + TryInto<BigUint>
    + TryInto<u32>
    + TryInto<Self::Wide>
    + QuickcheckBounds
    + UpperHex
    + Zero
{
    type Big: BigNumber;
    type Unsigned: SmallNumber<Big = BigUint> + Unsigned;
    type Wide;

    const BITS: u32;
    const MIN: Self;
    const MINUS_ONE: Self;

    fn widen(self) -> Self::Wide {
        self.try_into()
            .ok()
            .expect("widening conversion should never fail")
    }

    fn try_from_unsigned(u: Self::Unsigned) -> Option<Self> {
        Self::try_from(u).ok()
    }

    fn to_big(self) -> Self::Big {
        self.into()
    }

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
        let mut result = Self::Unsigned::zero();
        for &byte in bytes {
            result = (result << 8u32) | <Self::Unsigned as From<u8>>::from(byte);
        }
        Self::try_from(result).ok()
    }

    fn from_bytes_le(bytes: &[u8]) -> Option<Self> {
        // See note in `from_bytes_be`.
        if size_of::<Self>() < bytes.len() {
            return None;
        }
        let mut result = Self::Unsigned::zero();
        for &byte in bytes.iter().rev() {
            result = (result << 8u32) | <Self::Unsigned as From<u8>>::from(byte);
        }
        Self::try_from(result).ok()
    }

    fn try_to_unsigned(self) -> Option<Self::Unsigned>;

    /// Calls the primitive's `unsigned_abs` method, which returns the unsigned
    /// absolute value of the number.  For unsigned types, this is just the
    /// identity function.
    fn unsigned_abs(self) -> Self::Unsigned;

    /// Computes the absolute value of the number if it can be represented.
    fn try_abs(self) -> Option<Self>;

    /// Computes self to the power of `exp`, if it can be represented.
    fn try_pow(self, exp: u32) -> Option<Self>;

    /// Computes the negation of the number if it can be represented.
    fn try_neg(self) -> Option<Self>;

    /// Returns 0, 1, or -1, whichever has the same sign as `self`.
    fn signum(self) -> Self;

    duplicate_arith_ops!(paste! {
        fn [<op_fn _small_big>](lhs: Self, rhs: Self::Big) -> Self::Big;
        fn [<op_fn _small_big_ref>](lhs: Self, rhs: &Self::Big) -> Self::Big;
        fn [<op_fn _big_small>](lhs: Self::Big, rhs: Self) -> Self::Big;
        fn [<op_fn _big_ref_small>](lhs: &Self::Big, rhs: Self) -> Self::Big;
        fn [<op_fn _assign_small>](lhs: &mut Self::Big, rhs: Self);
    });
}

macro_rules! impl_binary_ops {
    () => {
        duplicate_arith_ops!(paste! {
            fn [<op_fn _small_big>](lhs: Self, rhs: Self::Big) -> Self::Big {
                std::ops::OpTrait::op_fn(lhs, rhs)
            }
            fn [<op_fn _small_big_ref>](lhs: Self, rhs: &Self::Big) -> Self::Big {
                std::ops::OpTrait::op_fn(lhs, rhs)
            }
            fn [<op_fn _big_small>](lhs: Self::Big, rhs: Self) -> Self::Big {
                std::ops::OpTrait::op_fn(lhs, rhs)
            }
            fn [<op_fn _big_ref_small>](lhs: &Self::Big, rhs: Self) -> Self::Big {
                std::ops::OpTrait::op_fn(lhs, rhs)
            }
            fn [<op_fn _assign_small>](lhs: &mut Self::Big, rhs: Self) {
                std::ops::[<OpTrait  Assign>]::[<op_fn _assign>](lhs, rhs)
            }
        });
    };
}

macro_rules! impl_small_num {
    ($signed:ty, $unsigned:ty) => {
        impl SmallNumber for $signed {
            type Big = BigInt;
            type Unsigned = $unsigned;
            type Wide = i128;

            const BITS: u32 = <$signed>::BITS;
            const MIN: Self = <$signed>::MIN;
            const MINUS_ONE: Self = -1;

            fn try_to_unsigned(self) -> Option<Self::Unsigned> {
                Self::Unsigned::try_from(self).ok()
            }

            fn unsigned_abs(self) -> Self::Unsigned {
                self.unsigned_abs()
            }

            fn try_abs(self) -> Option<Self> {
                if let (abs, false) = self.overflowing_abs() {
                    Some(abs)
                } else {
                    None
                }
            }

            fn try_neg(self) -> Option<Self> {
                if let (neg, false) = self.overflowing_neg() {
                    Some(neg)
                } else {
                    None
                }
            }

            fn try_pow(self, exp: u32) -> Option<Self> {
                if let (pow, false) = self.overflowing_pow(exp) {
                    Some(pow)
                } else {
                    None
                }
            }

            fn signum(self) -> Self {
                self.signum()
            }

            impl_binary_ops!();
        }

        impl SmallNumber for $unsigned {
            type Big = BigUint;
            type Unsigned = $unsigned;
            type Wide = u128;

            const BITS: u32 = <$unsigned>::BITS;
            const MIN: Self = <$unsigned>::MIN;
            const MINUS_ONE: Self = <$unsigned>::MAX;

            fn try_to_unsigned(self) -> Option<Self::Unsigned> {
                Some(self)
            }

            fn unsigned_abs(self) -> Self::Unsigned {
                self
            }

            fn try_abs(self) -> Option<Self> {
                Some(self)
            }

            fn try_neg(self) -> Option<Self> {
                if self == 0 { Some(0) } else { None }
            }

            fn try_pow(self, exp: u32) -> Option<Self> {
                if let (pow, false) = self.overflowing_pow(exp) {
                    Some(pow)
                } else {
                    None
                }
            }

            fn signum(self) -> Self {
                if self == 0 { 0 } else { 1 }
            }

            impl_binary_ops!();
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
