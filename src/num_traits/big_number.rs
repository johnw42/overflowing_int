//! This file defines the [`BigNumber`] trait, which is a common supertrait of
//! [`BigInt`] and [`BigUint`].
//!
//! Unlike the types themselves, the trait exposes all available operators as
//! methods with systematic names that are used in [`crate::num_ops`].

use num_bigint::{BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint};
use num_integer::{Integer, Roots};
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedEuclid, CheckedMul, CheckedSub, ConstZero, Euclid, FromBytes,
    FromPrimitive, Num, One, ToBytes, ToPrimitive, Zero,
};

use std::fmt::{Binary, Debug, Display, LowerHex, Octal, UpperHex};
use std::hash::Hash;
use std::iter::FusedIterator;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::str::FromStr;

use crate::num_traits::bounds::{ArbitraryBounds, QuickcheckBounds, RandBounds, SerdeBounds};
use crate::{
    duplicate_arith_ops, duplicate_bit_ops, duplicate_iprims, duplicate_prims, duplicate_shift_ops,
    duplicate_uprims,
};

use paste::paste;

/// A trait covering iterators over the digits of a big number, in a particular
/// base.  The digits are ordered least-significant first.
pub trait BigNumberDigits<'a, T>:
    DoubleEndedIterator<Item = T> + ExactSizeIterator<Item = T> + FusedIterator<Item = T> + 'a
{
}

impl<'a, T, I> BigNumberDigits<'a, T> for I where
    I: DoubleEndedIterator<Item = T> + ExactSizeIterator<Item = T> + FusedIterator<Item = T> + 'a
{
}

macro_rules! declare_binary_ops {
    ($op_fn:ident, $lhs:ty, $rhs:ty) => {
        paste! {
            fn [<$op_fn _ $lhs:lower _and_ $rhs:lower>](lhs: $lhs, rhs: $rhs) -> Self;
            fn [<$op_fn _ $lhs:lower _and_ref_ $rhs:lower>](lhs: $lhs, rhs: &$rhs) -> Self;
            fn [<$op_fn _ref_ $lhs:lower _and_ $rhs:lower>](lhs: &$lhs, rhs: $rhs) -> Self;
            fn [<$op_fn _ref_ $lhs:lower _and_ref_ $rhs:lower>](lhs: &$lhs, rhs: &$rhs) -> Self;
        }
    };
}
macro_rules! declare_binary_assign_op {
    ($op_fn:ident, $rhs:ty) => {
        paste! {
            fn [<$op_fn _assign_ $rhs:lower>](&mut self, rhs: $rhs);
        }
    };
}
macro_rules! declare_binary_assign_ref_op {
    ($op_fn:ident) => {
        paste! {
            fn [<$op_fn _assign_ref_self>](&mut self, rhs: &Self);
        }
    };
}

macro_rules! impl_binary_ops {
    ($op_fn:ident, $lhs:ty, $rhs:ty) => {
        paste! {
            #[inline]
            fn [<$op_fn _ $lhs:lower _and_ $rhs:lower>](lhs: $lhs, rhs: $rhs) -> Self {
                lhs.$op_fn(rhs)
            }

            #[inline]
            fn [<$op_fn _ $lhs:lower _and_ref_ $rhs:lower>](lhs: $lhs, rhs: &$rhs) -> Self {
                lhs.$op_fn(rhs)
            }

            #[inline]
            fn [<$op_fn _ref_ $lhs:lower _and_ $rhs:lower>](lhs: &$lhs, rhs: $rhs) -> Self {
                lhs.$op_fn(rhs)
            }

            #[inline]
            fn [<$op_fn _ref_ $lhs:lower _and_ref_ $rhs:lower>](lhs: &$lhs, rhs: &$rhs) -> Self {
                lhs.$op_fn(rhs)
            }
        }
    };
}
macro_rules! impl_binary_assign_op {
    ($op_fn:ident, $rhs:ty) => {
        paste! {
            #[inline]
            fn [<$op_fn _assign_ $rhs:lower>](&mut self, rhs: $rhs) {
                self.[<$op_fn _assign>](rhs);
            }
        }
    };
}
macro_rules! impl_binary_assign_ref_op {
    ($op_fn:ident) => {
        paste! {
            #[inline]
            fn [<$op_fn _assign_ref_self>](&mut self, rhs: &Self) {
                self.[<$op_fn _assign>](rhs);
            }
        }
    };
}

/// Common trait implemented by both [`BigInt`] and [`BigUint`].
pub trait BigNumber
where
    Self: Binary,
    Self: CheckedAdd,
    Self: CheckedDiv,
    Self: CheckedEuclid,
    Self: CheckedMul,
    Self: CheckedSub,
    Self: Clone,
    Self: ConstZero,
    Self: Debug,
    Self: Default,
    Self: Display,
    Self: Eq,
    Self: Euclid,
    Self: FromBytes,
    Self: FromPrimitive,
    Self: FromStr,
    Self: Hash,
    Self: Integer,
    Self: Into<BigInt>,
    Self: LowerHex,
    Self: Num<FromStrRadixErr = ParseBigIntError>,
    Self: Octal,
    Self: One,
    Self: Ord,
    Self: PartialEq,
    Self: PartialOrd,
    Self: RefUnwindSafe,
    Self: Roots,
    Self: RandBounds,
    Self: Send,
    Self: Sync,
    Self: ToBigInt,
    Self: ToBigUint,
    Self: ToBytes,
    Self: ToPrimitive,
    Self: Unpin,
    Self: UnwindSafe,
    Self: UpperHex,
    Self: Zero,
    Self: QuickcheckBounds,
    Self: ArbitraryBounds,
    Self: SerdeBounds,
    // From bounds
    Self: From<BigUint>,
    Self: From<bool>,
    Self: From<u128>,
    Self: From<u16>,
    Self: From<u32>,
    Self: From<u64>,
    Self: From<u8>,
    Self: From<usize>,
    // TryInto bounds
    Self: TryInto<u128>,
    Self: TryInto<u16>,
    Self: TryInto<u32>,
    Self: TryInto<u64>,
    Self: TryInto<u8>,
    Self: TryInto<usize>,
    Self: 'static,
{
    duplicate_arith_ops! {
        paste! {
            declare_binary_ops!(op_fn, Self, Self);
            declare_binary_assign_op!(op_fn, Self);
            declare_binary_assign_ref_op!(op_fn);
        }
        duplicate_uprims! { paste! {
            declare_binary_ops!(op_fn, Self, prim);
            declare_binary_ops!(op_fn, prim, Self);
            declare_binary_assign_op!(op_fn, prim);
        } }
    }
    duplicate_shift_ops! {
        duplicate_prims! { paste! {
            declare_binary_ops!(op_fn, Self, prim);
            declare_binary_assign_op!(op_fn, prim);
        } }
    }
    duplicate_bit_ops! {
        paste! {
            declare_binary_ops!(op_fn, Self, Self);
            declare_binary_assign_op!(op_fn, Self);
            declare_binary_assign_ref_op!(op_fn);
        }
    }

    /// Returns true if the value is -1.  This is used to optimize exponentiation by small unsigned integers.
    fn is_minus_one(&self) -> bool;

    /// Returns true if the number type is signed.
    fn is_signed() -> bool;

    fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self>;
    fn iter_u32_digits(&self) -> impl BigNumberDigits<'_, u32>;
    fn iter_u64_digits(&self) -> impl BigNumberDigits<'_, u64>;
    fn to_str_radix(&self, radix: u32) -> String;
    fn bits(&self) -> u64;
    fn checked_add(&self, v: &Self) -> Option<Self>;
    fn checked_sub(&self, v: &Self) -> Option<Self>;
    fn checked_mul(&self, v: &Self) -> Option<Self>;
    fn checked_div(&self, v: &Self) -> Option<Self>;
    fn pow(&self, exponent: u32) -> Self;
    fn modpow(&self, exponent: &Self, modulus: &Self) -> Self;
    fn modinv(&self, modulus: &Self) -> Option<Self>;
    fn trailing_zeros(&self) -> Option<u64>;
    fn bit(&self, bit: u64) -> bool;
    fn set_bit(&mut self, bit: u64, value: bool);
}

/// Trait adding extra methods to `BigInt`.  This is needed to give systematic names
/// to operators definte by other trait.
pub trait BigSigned: BigNumber
where
    // From bounds
    Self: From<BigInt>,
    Self: From<i128>,
    Self: From<i16>,
    Self: From<i32>,
    Self: From<i64>,
    Self: From<i8>,
    Self: From<isize>,
    // TryInto bounds
    Self: TryInto<i128>,
    Self: TryInto<i16>,
    Self: TryInto<i32>,
    Self: TryInto<i64>,
    Self: TryInto<i8>,
    Self: TryInto<isize>,
{
    duplicate_arith_ops! { duplicate_iprims! { paste! {
        declare_binary_ops!(op_fn, Self, prim);
        declare_binary_ops!(op_fn, prim, Self);
        declare_binary_assign_op!(op_fn, prim);
    } } }
}

macro_rules! impl_big_number_body {
    () => {
        duplicate_arith_ops! {
            paste! {
                impl_binary_ops!(op_fn, Self, Self);
                impl_binary_assign_op!(op_fn, Self);
                impl_binary_assign_ref_op!(op_fn);
            }
            duplicate_uprims! { paste! {
                impl_binary_ops!(op_fn, Self, prim);
                impl_binary_ops!(op_fn, prim, Self);
                impl_binary_assign_op!(op_fn, prim);
            }}
        }
        duplicate_shift_ops! {
            duplicate_prims! { paste! {
                impl_binary_ops!(op_fn, Self, prim);
                impl_binary_assign_op!(op_fn, prim);
            }}
        }
        duplicate_bit_ops! {
            paste! {
                impl_binary_ops!(op_fn, Self, Self);
                impl_binary_assign_op!(op_fn, Self);
                impl_binary_assign_ref_op!(op_fn);
            }
        }

        #[inline]
        fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self> {
            Self::parse_bytes(buf, radix)
        }

        #[inline]
        fn iter_u32_digits(&self) -> impl BigNumberDigits<'_, u32> {
            self.iter_u32_digits()
        }

        #[inline]
        fn iter_u64_digits(&self) -> impl BigNumberDigits<'_, u64> {
            self.iter_u64_digits()
        }

        #[inline]
        fn to_str_radix(&self, radix: u32) -> String {
            self.to_str_radix(radix)
        }

        #[inline]
        fn bits(&self) -> u64 {
            self.bits()
        }

        #[inline]
        fn checked_add(&self, v: &Self) -> Option<Self> {
            CheckedAdd::checked_add(self, v)
        }

        #[inline]
        fn checked_sub(&self, v: &Self) -> Option<Self> {
            CheckedSub::checked_sub(self, v)
        }

        #[inline]
        fn checked_mul(&self, v: &Self) -> Option<Self> {
            CheckedMul::checked_mul(self, v)
        }

        #[inline]
        fn checked_div(&self, v: &Self) -> Option<Self> {
            CheckedDiv::checked_div(self, v)
        }

        #[inline]
        fn pow(&self, exponent: u32) -> Self {
            self.pow(exponent)
        }

        #[inline]
        fn modpow(&self, exponent: &Self, modulus: &Self) -> Self {
            self.modpow(exponent, modulus)
        }

        #[inline]
        fn modinv(&self, modulus: &Self) -> Option<Self> {
            self.modinv(modulus)
        }

        #[inline]
        fn trailing_zeros(&self) -> Option<u64> {
            self.trailing_zeros()
        }

        #[inline]
        fn bit(&self, bit: u64) -> bool {
            self.bit(bit)
        }

        #[inline]
        fn set_bit(&mut self, bit: u64, value: bool) {
            self.set_bit(bit, value)
        }
    };
}

impl BigNumber for BigInt {
    impl_big_number_body!();

    #[inline]
    fn is_minus_one(&self) -> bool {
        self.sign() == Sign::Minus && self.magnitude().is_one()
    }

    #[inline]
    fn is_signed() -> bool {
        true
    }
}

impl BigNumber for BigUint {
    impl_big_number_body!();

    #[inline]
    fn is_minus_one(&self) -> bool {
        false
    }

    #[inline]
    fn is_signed() -> bool {
        false
    }
}

impl BigSigned for BigInt {
    duplicate_arith_ops! { duplicate_iprims! { paste! {
        impl_binary_ops!(op_fn, Self, prim);
        impl_binary_ops!(op_fn, prim, Self);
        impl_binary_assign_op!(op_fn, prim);
    } } }
}
