use std::fmt::{Binary, Debug, Display, LowerHex, Octal, UpperHex};
use std::hash::Hash;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::str::FromStr;

use num_bigint::{BigUint, ParseBigIntError, RandomBits, ToBigInt, ToBigUint};
use num_integer::{Integer, Roots};
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedEuclid, CheckedMul, CheckedSub, ConstZero, Euclid, FromBytes,
    FromPrimitive, Num, One, Pow, ToBytes, ToPrimitive, Zero,
};
use rand::distributions::uniform::SampleUniform;
use rand::prelude::Distribution;
use serde::{Deserialize, Serialize};

pub trait BigNatural
where
    Self: Binary
        + quickcheck::Arbitrary
        + Binary
        + CheckedAdd
        + CheckedDiv
        + CheckedEuclid
        + CheckedMul
        + CheckedSub
        + Clone
        + ConstZero
        + Debug
        + Default
        + Display
        + Eq
        + Euclid
        + FromBytes
        + FromPrimitive
        + FromStr
        + Hash
        + Integer
        + LowerHex
        + Num<FromStrRadixErr = ParseBigIntError>
        + Octal
        + One
        + Ord
        + PartialEq
        + PartialOrd
        + RefUnwindSafe
        + Roots
        + SampleUniform
        + Send
        + Serialize
        + Sized
        + Sync
        + ToBigInt
        + ToBigUint
        + ToBytes
        + ToPrimitive
        + Unpin
        + UnwindSafe
        + UpperHex
        + Zero,
    for<'a> Self: arbitrary::Arbitrary<'a>,
    for<'de> Self: Deserialize<'de>,
    RandomBits: Distribution<Self>,
    // From
    Self: From<BigUint>
        + From<bool>
        + From<u128>
        + From<u16>
        + From<u32>
        + From<u64>
        + From<u8>
        + From<usize>,
    // TryFrom
    for<'a> u128: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> u16: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> u32: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> u64: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> u8: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> usize: TryFrom<Self> + TryFrom<&'a Self>,
    // Add
    for<'a> Self: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> Self: Add<u128, Output = Self> + Add<&'a u128, Output = Self>,
    for<'a> Self: Add<u16, Output = Self> + Add<&'a u16, Output = Self>,
    for<'a> Self: Add<u32, Output = Self> + Add<&'a u32, Output = Self>,
    for<'a> Self: Add<u64, Output = Self> + Add<&'a u64, Output = Self>,
    for<'a> Self: Add<u8, Output = Self> + Add<&'a u8, Output = Self>,
    for<'a> Self: Add<usize, Output = Self> + Add<&'a usize, Output = Self>,
    for<'a> &'a Self: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a Self: Add<u128, Output = Self> + Add<&'a u128, Output = Self>,
    for<'a> &'a Self: Add<u16, Output = Self> + Add<&'a u16, Output = Self>,
    for<'a> &'a Self: Add<u32, Output = Self> + Add<&'a u32, Output = Self>,
    for<'a> &'a Self: Add<u64, Output = Self> + Add<&'a u64, Output = Self>,
    for<'a> &'a Self: Add<u8, Output = Self> + Add<&'a u8, Output = Self>,
    for<'a> &'a Self: Add<usize, Output = Self> + Add<&'a usize, Output = Self>,
    for<'a> u128: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> u16: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> u32: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> u64: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> u8: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> usize: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a u128: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a u16: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a u32: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a u64: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a u8: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a usize: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> Self: AddAssign<Self>
        + AddAssign<&'a Self>
        + AddAssign<u128>
        + AddAssign<u16>
        + AddAssign<u32>
        + AddAssign<u64>
        + AddAssign<u8>
        + AddAssign<usize>,
    for<'a> Self: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> Self: Add<u128, Output = Self> + Add<&'a u128, Output = Self>,
    for<'a> Self: Add<u16, Output = Self> + Add<&'a u16, Output = Self>,
    for<'a> Self: Add<u32, Output = Self> + Add<&'a u32, Output = Self>,
    for<'a> Self: Add<u64, Output = Self> + Add<&'a u64, Output = Self>,
    for<'a> Self: Add<u8, Output = Self> + Add<&'a u8, Output = Self>,
    for<'a> Self: Add<usize, Output = Self> + Add<&'a usize, Output = Self>,
    for<'a> &'a Self: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a Self: Add<u128, Output = Self> + Add<&'a u128, Output = Self>,
    for<'a> &'a Self: Add<u16, Output = Self> + Add<&'a u16, Output = Self>,
    for<'a> &'a Self: Add<u32, Output = Self> + Add<&'a u32, Output = Self>,
    for<'a> &'a Self: Add<u64, Output = Self> + Add<&'a u64, Output = Self>,
    for<'a> &'a Self: Add<u8, Output = Self> + Add<&'a u8, Output = Self>,
    for<'a> &'a Self: Add<usize, Output = Self> + Add<&'a usize, Output = Self>,
    for<'a> u128: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> u16: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> u32: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> u64: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> u8: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> usize: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a u128: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a u16: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a u32: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a u64: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a u8: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a usize: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> Self: AddAssign<Self>
        + AddAssign<&'a Self>
        + AddAssign<u128>
        + AddAssign<u16>
        + AddAssign<u32>
        + AddAssign<u64>
        + AddAssign<u8>
        + AddAssign<usize>,
    // BitAnd
    for<'a> Self: BitAnd<Self, Output = Self> + BitAnd<&'a Self, Output = Self>,
    for<'a> &'a Self: BitAnd<Self, Output = Self> + BitAnd<&'a Self, Output = Self>,
    for<'a> Self: BitAndAssign<Self> + BitAndAssign<&'a Self>,
    // BitOr
    for<'a> Self: BitOr<Self, Output = Self> + BitOr<&'a Self, Output = Self>,
    for<'a> &'a Self: BitOr<Self, Output = Self> + BitOr<&'a Self, Output = Self>,
    for<'a> Self: BitOrAssign<Self> + BitOrAssign<&'a Self>,
    // BitXor
    for<'a> Self: BitXor<Self, Output = Self> + BitXor<&'a Self, Output = Self>,
    for<'a> &'a Self: BitXor<Self, Output = Self> + BitXor<&'a Self, Output = Self>,
    for<'a> Self: BitXorAssign<Self> + BitXorAssign<&'a Self>,
    // Div
    for<'a> Self: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> Self: Div<u128, Output = Self> + Div<&'a u128, Output = Self>,
    for<'a> Self: Div<u16, Output = Self> + Div<&'a u16, Output = Self>,
    for<'a> Self: Div<u32, Output = Self> + Div<&'a u32, Output = Self>,
    for<'a> Self: Div<u64, Output = Self> + Div<&'a u64, Output = Self>,
    for<'a> Self: Div<u8, Output = Self> + Div<&'a u8, Output = Self>,
    for<'a> Self: Div<usize, Output = Self> + Div<&'a usize, Output = Self>,
    for<'a> &'a Self: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> &'a Self: Div<u128, Output = Self> + Div<&'a u128, Output = Self>,
    for<'a> &'a Self: Div<u16, Output = Self> + Div<&'a u16, Output = Self>,
    for<'a> &'a Self: Div<u32, Output = Self> + Div<&'a u32, Output = Self>,
    for<'a> &'a Self: Div<u64, Output = Self> + Div<&'a u64, Output = Self>,
    for<'a> &'a Self: Div<u8, Output = Self> + Div<&'a u8, Output = Self>,
    for<'a> &'a Self: Div<usize, Output = Self> + Div<&'a usize, Output = Self>,
    for<'a> u128: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> u16: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> u32: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> u64: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> u8: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> usize: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> &'a u128: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> &'a u16: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> &'a u32: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> &'a u64: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> &'a u8: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> &'a usize: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> Self: DivAssign<Self>
        + DivAssign<&'a Self>
        + DivAssign<u128>
        + DivAssign<u16>
        + DivAssign<u32>
        + DivAssign<u64>
        + DivAssign<u8>
        + DivAssign<usize>,
    // Mul
    for<'a> Self: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> Self: Mul<u128, Output = Self> + Mul<&'a u128, Output = Self>,
    for<'a> Self: Mul<u16, Output = Self> + Mul<&'a u16, Output = Self>,
    for<'a> Self: Mul<u32, Output = Self> + Mul<&'a u32, Output = Self>,
    for<'a> Self: Mul<u64, Output = Self> + Mul<&'a u64, Output = Self>,
    for<'a> Self: Mul<u8, Output = Self> + Mul<&'a u8, Output = Self>,
    for<'a> Self: Mul<usize, Output = Self> + Mul<&'a usize, Output = Self>,
    for<'a> &'a Self: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a Self: Mul<u128, Output = Self> + Mul<&'a u128, Output = Self>,
    for<'a> &'a Self: Mul<u16, Output = Self> + Mul<&'a u16, Output = Self>,
    for<'a> &'a Self: Mul<u32, Output = Self> + Mul<&'a u32, Output = Self>,
    for<'a> &'a Self: Mul<u64, Output = Self> + Mul<&'a u64, Output = Self>,
    for<'a> &'a Self: Mul<u8, Output = Self> + Mul<&'a u8, Output = Self>,
    for<'a> u128: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> u16: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> u32: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> u64: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> u8: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> usize: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a Self: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a u128: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a u16: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a u32: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a u64: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a u8: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a usize: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> Self: MulAssign<Self>
        + MulAssign<&'a Self>
        + MulAssign<u128>
        + MulAssign<u16>
        + MulAssign<u32>
        + MulAssign<u64>
        + MulAssign<u8>
        + MulAssign<usize>,
    // Pow
    for<'a> Self: Pow<u128, Output = Self> + Pow<&'a u128, Output = Self>,
    for<'a> Self: Pow<u16, Output = Self> + Pow<&'a u16, Output = Self>,
    for<'a> Self: Pow<u32, Output = Self> + Pow<&'a u32, Output = Self>,
    for<'a> Self: Pow<u64, Output = Self> + Pow<&'a u64, Output = Self>,
    for<'a> Self: Pow<u8, Output = Self> + Pow<&'a u8, Output = Self>,
    for<'a> Self: Pow<usize, Output = Self> + Pow<&'a usize, Output = Self>,
    for<'a> &'a Self: Pow<u128, Output = Self> + Pow<&'a u128, Output = Self>,
    for<'a> &'a Self: Pow<u16, Output = Self> + Pow<&'a u16, Output = Self>,
    for<'a> &'a Self: Pow<u32, Output = Self> + Pow<&'a u32, Output = Self>,
    for<'a> &'a Self: Pow<u64, Output = Self> + Pow<&'a u64, Output = Self>,
    for<'a> &'a Self: Pow<u8, Output = Self> + Pow<&'a u8, Output = Self>,
    for<'a> &'a Self: Pow<usize, Output = Self> + Pow<&'a usize, Output = Self>,
    // Rem
    for<'a> Self: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> Self: Rem<u128, Output = Self> + Rem<&'a u128, Output = Self>,
    for<'a> Self: Rem<u16, Output = Self> + Rem<&'a u16, Output = Self>,
    for<'a> Self: Rem<u32, Output = Self> + Rem<&'a u32, Output = Self>,
    for<'a> Self: Rem<u64, Output = Self> + Rem<&'a u64, Output = Self>,
    for<'a> Self: Rem<u8, Output = Self> + Rem<&'a u8, Output = Self>,
    for<'a> Self: Rem<usize, Output = Self> + Rem<&'a usize, Output = Self>,
    for<'a> &'a Self: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a Self: Rem<u128, Output = Self> + Rem<&'a u128, Output = Self>,
    for<'a> &'a Self: Rem<u16, Output = Self> + Rem<&'a u16, Output = Self>,
    for<'a> &'a Self: Rem<u32, Output = Self> + Rem<&'a u32, Output = Self>,
    for<'a> &'a Self: Rem<u64, Output = Self> + Rem<&'a u64, Output = Self>,
    for<'a> &'a Self: Rem<u8, Output = Self> + Rem<&'a u8, Output = Self>,
    for<'a> &'a Self: Rem<usize, Output = Self> + Rem<&'a usize, Output = Self>,
    for<'a> u128: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> u16: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> u32: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> u64: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> u8: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a Self: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a u128: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a u16: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a u32: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a u64: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a u8: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a usize: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> Self: RemAssign<Self>
        + RemAssign<&'a Self>
        + RemAssign<u128>
        + RemAssign<u16>
        + RemAssign<u32>
        + RemAssign<u64>
        + RemAssign<u8>
        + RemAssign<usize>,
    // Shl
    for<'a> Self: Shl<i128, Output = Self> + Shl<&'a i128, Output = Self>,
    for<'a> Self: Shl<i16, Output = Self> + Shl<&'a i16, Output = Self>,
    for<'a> Self: Shl<i32, Output = Self> + Shl<&'a i32, Output = Self>,
    for<'a> Self: Shl<i64, Output = Self> + Shl<&'a i64, Output = Self>,
    for<'a> Self: Shl<i8, Output = Self> + Shl<&'a i8, Output = Self>,
    for<'a> Self: Shl<isize, Output = Self> + Shl<&'a isize, Output = Self>,
    for<'a> &'a Self: Shl<i128, Output = Self>
        + Shl<i16, Output = Self>
        + Shl<i32, Output = Self>
        + Shl<i64, Output = Self>
        + Shl<i8, Output = Self>
        + Shl<isize, Output = Self>
        + Shl<u128, Output = Self>
        + Shl<u16, Output = Self>
        + Shl<u32, Output = Self>
        + Shl<u64, Output = Self>
        + Shl<u8, Output = Self>
        + Shl<usize, Output = Self>
        + Shl<&'a i128, Output = Self>
        + Shl<&'a i16, Output = Self>
        + Shl<&'a i32, Output = Self>
        + Shl<&'a i64, Output = Self>
        + Shl<&'a i8, Output = Self>
        + Shl<&'a isize, Output = Self>
        + Shl<&'a u128, Output = Self>
        + Shl<&'a u16, Output = Self>
        + Shl<&'a u32, Output = Self>
        + Shl<&'a u64, Output = Self>
        + Shl<&'a u8, Output = Self>
        + Shl<&'a usize, Output = Self>,
    for<'a> Self: ShlAssign<i128>
        + ShlAssign<i16>
        + ShlAssign<i32>
        + ShlAssign<i64>
        + ShlAssign<i8>
        + ShlAssign<isize>
        + ShlAssign<u128>
        + ShlAssign<u16>
        + ShlAssign<u32>
        + ShlAssign<u64>
        + ShlAssign<u8>
        + ShlAssign<usize>
        + ShlAssign<&'a i128>
        + ShlAssign<&'a i16>
        + ShlAssign<&'a i32>
        + ShlAssign<&'a i64>
        + ShlAssign<&'a i8>
        + ShlAssign<&'a isize>
        + ShlAssign<&'a u128>
        + ShlAssign<&'a u16>
        + ShlAssign<&'a u32>
        + ShlAssign<&'a u64>
        + ShlAssign<&'a u8>
        + ShlAssign<&'a usize>,
    // Shr
    for<'a> Self: Shr<i128, Output = Self> + Shr<&'a i128, Output = Self>,
    for<'a> Self: Shr<i16, Output = Self> + Shr<&'a i16, Output = Self>,
    for<'a> Self: Shr<i32, Output = Self> + Shr<&'a i32, Output = Self>,
    for<'a> Self: Shr<i64, Output = Self> + Shr<&'a i64, Output = Self>,
    for<'a> Self: Shr<i8, Output = Self> + Shr<&'a i8, Output = Self>,
    for<'a> Self: Shr<isize, Output = Self> + Shr<&'a isize, Output = Self>,
    for<'a> &'a Self: Shr<i128, Output = Self>
        + Shr<i16, Output = Self>
        + Shr<i32, Output = Self>
        + Shr<i64, Output = Self>
        + Shr<i8, Output = Self>
        + Shr<isize, Output = Self>
        + Shr<u128, Output = Self>
        + Shr<u16, Output = Self>
        + Shr<u32, Output = Self>
        + Shr<u64, Output = Self>
        + Shr<u8, Output = Self>
        + Shr<usize, Output = Self>
        + Shr<&'a i128, Output = Self>
        + Shr<&'a i16, Output = Self>
        + Shr<&'a i32, Output = Self>
        + Shr<&'a i64, Output = Self>
        + Shr<&'a i8, Output = Self>
        + Shr<&'a isize, Output = Self>
        + Shr<&'a u128, Output = Self>
        + Shr<&'a u16, Output = Self>
        + Shr<&'a u32, Output = Self>
        + Shr<&'a u64, Output = Self>
        + Shr<&'a u8, Output = Self>
        + Shr<&'a usize, Output = Self>,
    for<'a> Self: ShrAssign<i128>
        + ShrAssign<i16>
        + ShrAssign<i32>
        + ShrAssign<i64>
        + ShrAssign<i8>
        + ShrAssign<isize>
        + ShrAssign<u128>
        + ShrAssign<u16>
        + ShrAssign<u32>
        + ShrAssign<u64>
        + ShrAssign<u8>
        + ShrAssign<usize>
        + ShrAssign<&'a i128>
        + ShrAssign<&'a i16>
        + ShrAssign<&'a i32>
        + ShrAssign<&'a i64>
        + ShrAssign<&'a i8>
        + ShrAssign<&'a isize>
        + ShrAssign<&'a u128>
        + ShrAssign<&'a u16>
        + ShrAssign<&'a u32>
        + ShrAssign<&'a u64>
        + ShrAssign<&'a u8>
        + ShrAssign<&'a usize>,
    // Sub
    for<'a> Self: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> Self: Sub<u128, Output = Self> + Sub<&'a u128, Output = Self>,
    for<'a> Self: Sub<u16, Output = Self> + Sub<&'a u16, Output = Self>,
    for<'a> Self: Sub<u32, Output = Self> + Sub<&'a u32, Output = Self>,
    for<'a> Self: Sub<u64, Output = Self> + Sub<&'a u64, Output = Self>,
    for<'a> Self: Sub<u8, Output = Self> + Sub<&'a u8, Output = Self>,
    for<'a> Self: Sub<usize, Output = Self> + Sub<&'a usize, Output = Self>,
    for<'a> &'a Self: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> &'a Self: Sub<u128, Output = Self> + Sub<&'a u128, Output = Self>,
    for<'a> &'a Self: Sub<u16, Output = Self> + Sub<&'a u16, Output = Self>,
    for<'a> &'a Self: Sub<u32, Output = Self> + Sub<&'a u32, Output = Self>,
    for<'a> &'a Self: Sub<u64, Output = Self> + Sub<&'a u64, Output = Self>,
    for<'a> &'a Self: Sub<u8, Output = Self> + Sub<&'a u8, Output = Self>,
    for<'a> &'a Self: Sub<usize, Output = Self> + Sub<&'a usize, Output = Self>,
    for<'a> u128: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> u16: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> u32: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> u64: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> u8: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> usize: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> &'a u128: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> &'a u16: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> &'a u32: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> &'a u64: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> &'a u8: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> &'a usize: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> Self: SubAssign<Self>
        + SubAssign<&'a Self>
        + SubAssign<u128>
        + SubAssign<u16>
        + SubAssign<u32>
        + SubAssign<u64>
        + SubAssign<u8>
        + SubAssign<usize>,
{
    /// A constant `BigNatural` with value 0, useful for static initialization.
    const ZERO: Self;

    /// Creates and initializes a [`BigNatural`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    fn new(digits: Vec<u32>) -> Self;

    /// Creates and initializes a [`BigNatural`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    fn from_slice(slice: &[u32]) -> Self;

    /// Assign a value to a [`BigNatural`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    fn assign_from_slice(&mut self, slice: &[u32]);

    /// Creates and initializes a [`BigNatural`].
    ///
    /// The bytes are in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigUint;
    ///
    /// assert_eq!(BigUint::from_bytes_be(b"A"),
    ///            BigUint::parse_bytes(b"65", 10).unwrap());
    /// assert_eq!(BigUint::from_bytes_be(b"AA"),
    ///            BigUint::parse_bytes(b"16705", 10).unwrap());
    /// assert_eq!(BigUint::from_bytes_be(b"AB"),
    ///            BigUint::parse_bytes(b"16706", 10).unwrap());
    /// assert_eq!(BigUint::from_bytes_be(b"Hello world!"),
    ///            BigUint::parse_bytes(b"22405534230753963835153736737", 10).unwrap());
    /// ```
    fn from_bytes_be(bytes: &[u8]) -> Self;

    /// Creates and initializes a [`BigNatural`].
    ///
    /// The bytes are in little-endian byte order.
    fn from_bytes_le(bytes: &[u8]) -> Self;

    /// Creates and initializes a [`BigNatural`]. The input slice must contain
    /// ascii/utf8 characters in [0-9a-zA-Z].
    /// `radix` must be in the range `2...36`.
    ///
    /// The function `from_str_radix` from the `Num` trait provides the same logic
    /// for `&str` buffers.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigUint, ToBigUint};
    ///
    /// assert_eq!(BigUint::parse_bytes(b"1234", 10), ToBigUint::to_biguint(&1234));
    /// assert_eq!(BigUint::parse_bytes(b"ABCD", 16), ToBigUint::to_biguint(&0xABCD));
    /// assert_eq!(BigUint::parse_bytes(b"G", 16), None);
    /// ```
    fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self>;

    /// Creates and initializes a [`BigNatural`]. Each `u8` of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in big-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigUint};
    ///
    /// let inbase190 = &[15, 33, 125, 12, 14];
    /// let a = BigUint::from_radix_be(inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_be(190), inbase190);
    /// ```
    fn from_radix_be(buf: &[u8], radix: u32) -> Option<Self>;

    /// Creates and initializes a [`BigNatural`]. Each `u8` of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in little-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigUint};
    ///
    /// let inbase190 = &[14, 12, 125, 33, 15];
    /// let a = BigUint::from_radix_be(inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_be(190), inbase190);
    /// ```
    fn from_radix_le(buf: &[u8], radix: u32) -> Option<Self>;

    /// Returns the byte representation of the [`BigNatural`] in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigUint;
    ///
    /// let i = BigUint::parse_bytes(b"1125", 10).unwrap();
    /// assert_eq!(i.to_bytes_be(), vec![4, 101]);
    /// ```
    fn to_bytes_be(&self) -> Vec<u8> {
        let mut v = self.to_bytes_le();
        v.reverse();
        v
    }

    /// Returns the byte representation of the [`BigNatural`] in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigUint;
    ///
    /// let i = BigUint::parse_bytes(b"1125", 10).unwrap();
    /// assert_eq!(i.to_bytes_le(), vec![101, 4]);
    /// ```
    fn to_bytes_le(&self) -> Vec<u8>;

    /// Returns the `u32` digits representation of the [`BigNatural`] ordered least significant digit
    /// first.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigUint;
    ///
    /// assert_eq!(BigUint::from(1125u32).to_u32_digits(), vec![1125]);
    /// assert_eq!(BigUint::from(4294967295u32).to_u32_digits(), vec![4294967295]);
    /// assert_eq!(BigUint::from(4294967296u64).to_u32_digits(), vec![0, 1]);
    /// assert_eq!(BigUint::from(112500000000u64).to_u32_digits(), vec![830850304, 26]);
    /// ```
    fn to_u32_digits(&self) -> Vec<u32> {
        self.iter_u32_digits().collect()
    }

    /// Returns the `u64` digits representation of the [`BigNatural`] ordered least significant digit
    /// first.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigUint;
    ///
    /// assert_eq!(BigUint::from(1125u32).to_u64_digits(), vec![1125]);
    /// assert_eq!(BigUint::from(4294967295u32).to_u64_digits(), vec![4294967295]);
    /// assert_eq!(BigUint::from(4294967296u64).to_u64_digits(), vec![4294967296]);
    /// assert_eq!(BigUint::from(112500000000u64).to_u64_digits(), vec![112500000000]);
    /// assert_eq!(BigUint::from(1u128 << 64).to_u64_digits(), vec![0, 1]);
    /// ```
    fn to_u64_digits(&self) -> Vec<u64> {
        self.iter_u64_digits().collect()
    }

    /// Returns an iterator of `u32` digits representation of the [`BigNatural`] ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigUint;
    ///
    /// assert_eq!(BigUint::from(1125u32).iter_u32_digits().collect::<Vec<u32>>(), vec![1125]);
    /// assert_eq!(BigUint::from(4294967295u32).iter_u32_digits().collect::<Vec<u32>>(), vec![4294967295]);
    /// assert_eq!(BigUint::from(4294967296u64).iter_u32_digits().collect::<Vec<u32>>(), vec![0, 1]);
    /// assert_eq!(BigUint::from(112500000000u64).iter_u32_digits().collect::<Vec<u32>>(), vec![830850304, 26]);
    /// ```
    fn iter_u32_digits(
        &self,
    ) -> impl DoubleEndedIterator<Item = u32> + ExactSizeIterator<Item = u32> + '_;

    /// Returns an iterator of `u64` digits representation of the [`BigNatural`] ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigUint;
    ///
    /// assert_eq!(BigUint::from(1125u32).iter_u64_digits().collect::<Vec<u64>>(), vec![1125]);
    /// assert_eq!(BigUint::from(4294967295u32).iter_u64_digits().collect::<Vec<u64>>(), vec![4294967295]);
    /// assert_eq!(BigUint::from(4294967296u64).iter_u64_digits().collect::<Vec<u64>>(), vec![4294967296]);
    /// assert_eq!(BigUint::from(112500000000u64).iter_u64_digits().collect::<Vec<u64>>(), vec![112500000000]);
    /// assert_eq!(BigUint::from(1u128 << 64).iter_u64_digits().collect::<Vec<u64>>(), vec![0, 1]);
    /// ```
    fn iter_u64_digits(
        &self,
    ) -> impl DoubleEndedIterator<Item = u64> + ExactSizeIterator<Item = u64> + '_;

    /// Returns the integer formatted as a string in the given radix.
    /// `radix` must be in the range `2...36`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigUint;
    ///
    /// let i = BigUint::parse_bytes(b"ff", 16).unwrap();
    /// assert_eq!(i.to_str_radix(16), "ff");
    /// ```
    fn to_str_radix(&self, radix: u32) -> String;

    /// Returns the integer in the requested base in big-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based `u8` number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigUint;
    ///
    /// assert_eq!(BigUint::from(0xFFFFu64).to_radix_be(159),
    ///            vec![2, 94, 27]);
    /// // 0xFFFF = 65535 = 2*(159^2) + 94*159 + 27
    /// ```
    fn to_radix_be(&self, radix: u32) -> Vec<u8>;

    /// Returns the integer in the requested base in little-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based u8 number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigUint;
    ///
    /// assert_eq!(BigUint::from(0xFFFFu64).to_radix_le(159),
    ///            vec![27, 94, 2]);
    /// // 0xFFFF = 65535 = 27 + 94*159 + 2*(159^2)
    /// ```
    fn to_radix_le(&self, radix: u32) -> Vec<u8>;

    /// Determines the fewest bits necessary to express the [`BigNatural`].
    fn bits(&self) -> u64;

    /// Returns `self ^ exponent`.
    fn pow(&self, exponent: u32) -> Self {
        Pow::pow(self, exponent)
    }

    /// Returns `(self ^ exponent) % modulus`.
    ///
    /// Panics if the modulus is zero.
    fn modpow(&self, exponent: &Self, modulus: &Self) -> Self;

    /// Returns the modular multiplicative inverse if it exists, otherwise `None`.
    ///
    /// This solves for `x` in the interval `[0, modulus)` such that `self * x ≡ 1 (mod modulus)`.
    /// The solution exists if and only if `gcd(self, modulus) == 1`.
    ///
    /// ```
    /// use num_bigint::BigUint;
    /// use num_traits::{One, Zero};
    ///
    /// let m = BigUint::from(383_u32);
    ///
    /// // Trivial cases
    /// assert_eq!(BigUint::zero().modinv(&m), None);
    /// assert_eq!(BigUint::one().modinv(&m), Some(BigUint::one()));
    /// let neg1 = &m - 1u32;
    /// assert_eq!(neg1.modinv(&m), Some(neg1));
    ///
    /// let a = BigUint::from(271_u32);
    /// let x = a.modinv(&m).unwrap();
    /// assert_eq!(x, BigUint::from(106_u32));
    /// assert_eq!(x.modinv(&m).unwrap(), a);
    /// assert!((a * x % m).is_one());
    /// ```
    fn modinv(&self, modulus: &Self) -> Option<Self>;

    /// Returns the truncated principal square root of `self` --
    /// see [Roots::sqrt](https://docs.rs/num-integer/0.1/num_integer/trait.Roots.html#method.sqrt)
    fn sqrt(&self) -> Self {
        Roots::sqrt(self)
    }

    /// Returns the truncated principal cube root of `self` --
    /// see [Roots::cbrt](https://docs.rs/num-integer/0.1/num_integer/trait.Roots.html#method.cbrt).
    fn cbrt(&self) -> Self {
        Roots::cbrt(self)
    }

    /// Returns the truncated principal `n`th root of `self` --
    /// see [Roots::nth_root](https://docs.rs/num-integer/0.1/num_integer/trait.Roots.html#tymethod.nth_root).
    fn nth_root(&self, n: u32) -> Self {
        Roots::nth_root(self, n)
    }

    /// Returns the number of least-significant bits that are zero,
    /// or `None` if the entire number is zero.
    fn trailing_zeros(&self) -> Option<u64>;

    /// Returns the number of least-significant bits that are ones.
    fn trailing_ones(&self) -> u64;

    /// Returns the number of one bits.
    fn count_ones(&self) -> u64;

    /// Returns whether the bit in the given position is set
    fn bit(&self, bit: u64) -> bool;

    /// Sets or clears the bit in the given position
    ///
    /// Note that setting a bit greater than the current bit length, a reallocation may be needed
    /// to store the new digits
    fn set_bit(&mut self, bit: u64, value: bool);
}

impl BigNatural for BigUint {
    const ZERO: Self = Self::ZERO;

    fn new(digits: Vec<u32>) -> Self {
        BigUint::new(digits)
    }

    fn from_slice(slice: &[u32]) -> Self {
        Self::from_slice(slice)
    }

    fn assign_from_slice(&mut self, slice: &[u32]) {
        self.assign_from_slice(slice)
    }

    fn from_bytes_be(bytes: &[u8]) -> Self {
        Self::from_bytes_be(bytes)
    }

    fn from_bytes_le(bytes: &[u8]) -> Self {
        Self::from_bytes_le(bytes)
    }

    fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self> {
        Self::parse_bytes(buf, radix)
    }

    fn from_radix_be(buf: &[u8], radix: u32) -> Option<Self> {
        Self::from_radix_be(buf, radix)
    }

    fn from_radix_le(buf: &[u8], radix: u32) -> Option<Self> {
        Self::from_radix_le(buf, radix)
    }

    fn to_bytes_be(&self) -> Vec<u8> {
        self.to_bytes_be()
    }

    fn to_bytes_le(&self) -> Vec<u8> {
        self.to_bytes_le()
    }

    fn to_u32_digits(&self) -> Vec<u32> {
        self.to_u32_digits()
    }

    fn to_u64_digits(&self) -> Vec<u64> {
        self.to_u64_digits()
    }

    fn iter_u32_digits(
        &self,
    ) -> impl DoubleEndedIterator<Item = u32> + ExactSizeIterator<Item = u32> + '_ {
        self.iter_u32_digits()
    }

    fn iter_u64_digits(
        &self,
    ) -> impl DoubleEndedIterator<Item = u64> + ExactSizeIterator<Item = u64> + '_ {
        self.iter_u64_digits()
    }

    fn to_str_radix(&self, radix: u32) -> String {
        self.to_str_radix(radix)
    }

    fn to_radix_be(&self, radix: u32) -> Vec<u8> {
        self.to_radix_be(radix)
    }

    fn to_radix_le(&self, radix: u32) -> Vec<u8> {
        self.to_radix_le(radix)
    }

    fn bits(&self) -> u64 {
        self.bits()
    }

    fn pow(&self, exponent: u32) -> Self {
        self.pow(exponent)
    }

    fn modpow(&self, exponent: &Self, modulus: &Self) -> Self {
        self.modpow(exponent, modulus)
    }

    fn modinv(&self, modulus: &Self) -> Option<Self> {
        self.modinv(modulus)
    }

    fn sqrt(&self) -> Self {
        self.sqrt()
    }

    fn cbrt(&self) -> Self {
        self.cbrt()
    }

    fn nth_root(&self, n: u32) -> Self {
        self.nth_root(n)
    }

    fn trailing_zeros(&self) -> Option<u64> {
        self.trailing_zeros()
    }

    fn bit(&self, bit: u64) -> bool {
        self.bit(bit)
    }

    fn set_bit(&mut self, bit: u64, value: bool) {
        self.set_bit(bit, value)
    }

    fn trailing_ones(&self) -> u64 {
        self.trailing_ones()
    }

    fn count_ones(&self) -> u64 {
        self.count_ones()
    }
}
