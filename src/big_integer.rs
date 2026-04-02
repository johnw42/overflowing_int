use std::fmt::{Binary, Debug, Display, LowerHex, Octal, UpperHex};
use std::hash::Hash;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::str::FromStr;

use num_bigint::{BigInt, BigUint, ParseBigIntError, RandomBits, Sign, ToBigInt, ToBigUint};
use num_integer::{Integer, Roots};
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedEuclid, CheckedMul, CheckedSub, ConstZero, Euclid, FromBytes,
    FromPrimitive, Num, One, Pow, Signed, ToBytes, ToPrimitive, Zero,
};
use rand::distributions::uniform::SampleUniform;
use rand::prelude::Distribution;
use serde::{Deserialize, Serialize};

use crate::CBigInt;

pub trait BigInteger
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
        + Neg
        + Not
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
        + Signed
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
    Self: arbitrary::Arbitrary<'static>,
    for<'de> Self: Deserialize<'de>,
    RandomBits: Distribution<Self>,
    // From
    Self: From<BigInt>
        + From<BigUint>
        + From<bool>
        + From<i128>
        + From<i16>
        + From<i32>
        + From<i64>
        + From<i8>
        + From<isize>
        + From<u128>
        + From<u16>
        + From<u32>
        + From<u64>
        + From<u8>
        + From<usize>,
    // TryFrom
    for<'a> i128: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> i16: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> i32: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> i64: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> i8: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> isize: TryFrom<Self> + TryFrom<&'a Self>,
    // Add
    for<'a> Self: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> Self: Add<i128, Output = Self> + Add<&'a i128, Output = Self>,
    for<'a> Self: Add<i16, Output = Self> + Add<&'a i16, Output = Self>,
    for<'a> Self: Add<i32, Output = Self> + Add<&'a i32, Output = Self>,
    for<'a> Self: Add<i64, Output = Self> + Add<&'a i64, Output = Self>,
    for<'a> Self: Add<i8, Output = Self> + Add<&'a i8, Output = Self>,
    for<'a> Self: Add<isize, Output = Self> + Add<&'a isize, Output = Self>,
    for<'a> &'a Self: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a Self: Add<i128, Output = Self> + Add<&'a i128, Output = Self>,
    for<'a> &'a Self: Add<i16, Output = Self> + Add<&'a i16, Output = Self>,
    for<'a> &'a Self: Add<i32, Output = Self> + Add<&'a i32, Output = Self>,
    for<'a> &'a Self: Add<i64, Output = Self> + Add<&'a i64, Output = Self>,
    for<'a> &'a Self: Add<i8, Output = Self> + Add<&'a i8, Output = Self>,
    for<'a> &'a Self: Add<isize, Output = Self> + Add<&'a isize, Output = Self>,
    for<'a> i128: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> i16: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> i32: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> i64: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> i8: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> isize: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a i128: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a i16: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a i32: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a i64: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a i8: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a isize: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> Self: AddAssign<Self>
        + AddAssign<&'a Self>
        + AddAssign<i128>
        + AddAssign<i16>
        + AddAssign<i32>
        + AddAssign<i64>
        + AddAssign<i8>
        + AddAssign<isize>,
    for<'a> Self: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> Self: Add<i128, Output = Self> + Add<&'a i128, Output = Self>,
    for<'a> Self: Add<i16, Output = Self> + Add<&'a i16, Output = Self>,
    for<'a> Self: Add<i32, Output = Self> + Add<&'a i32, Output = Self>,
    for<'a> Self: Add<i64, Output = Self> + Add<&'a i64, Output = Self>,
    for<'a> Self: Add<i8, Output = Self> + Add<&'a i8, Output = Self>,
    for<'a> Self: Add<isize, Output = Self> + Add<&'a isize, Output = Self>,
    for<'a> &'a Self: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a Self: Add<i128, Output = Self> + Add<&'a i128, Output = Self>,
    for<'a> &'a Self: Add<i16, Output = Self> + Add<&'a i16, Output = Self>,
    for<'a> &'a Self: Add<i32, Output = Self> + Add<&'a i32, Output = Self>,
    for<'a> &'a Self: Add<i64, Output = Self> + Add<&'a i64, Output = Self>,
    for<'a> &'a Self: Add<i8, Output = Self> + Add<&'a i8, Output = Self>,
    for<'a> &'a Self: Add<isize, Output = Self> + Add<&'a isize, Output = Self>,
    for<'a> i128: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> i16: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> i32: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> i64: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> i8: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> isize: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a i128: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a i16: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a i32: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a i64: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a i8: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> &'a isize: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> Self: AddAssign<Self>
        + AddAssign<&'a Self>
        + AddAssign<i128>
        + AddAssign<i16>
        + AddAssign<i32>
        + AddAssign<i64>
        + AddAssign<i8>
        + AddAssign<isize>,
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
    for<'a> Self: Div<i128, Output = Self> + Div<&'a i128, Output = Self>,
    for<'a> Self: Div<i16, Output = Self> + Div<&'a i16, Output = Self>,
    for<'a> Self: Div<i32, Output = Self> + Div<&'a i32, Output = Self>,
    for<'a> Self: Div<i64, Output = Self> + Div<&'a i64, Output = Self>,
    for<'a> Self: Div<i8, Output = Self> + Div<&'a i8, Output = Self>,
    for<'a> Self: Div<isize, Output = Self> + Div<&'a isize, Output = Self>,
    for<'a> &'a Self: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> &'a Self: Div<i128, Output = Self> + Div<&'a i128, Output = Self>,
    for<'a> &'a Self: Div<i16, Output = Self> + Div<&'a i16, Output = Self>,
    for<'a> &'a Self: Div<i32, Output = Self> + Div<&'a i32, Output = Self>,
    for<'a> &'a Self: Div<i64, Output = Self> + Div<&'a i64, Output = Self>,
    for<'a> &'a Self: Div<i8, Output = Self> + Div<&'a i8, Output = Self>,
    for<'a> &'a Self: Div<isize, Output = Self> + Div<&'a isize, Output = Self>,
    for<'a> i128: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> i16: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> i32: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> i64: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> i8: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> isize: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> &'a i128: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> &'a i16: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> &'a i32: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> &'a i64: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> &'a i8: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> &'a isize: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> Self: DivAssign<Self>
        + DivAssign<&'a Self>
        + DivAssign<i128>
        + DivAssign<i16>
        + DivAssign<i32>
        + DivAssign<i64>
        + DivAssign<i8>
        + DivAssign<isize>,
    // Mul
    for<'a> Self: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> Self: Mul<i128, Output = Self> + Mul<&'a i128, Output = Self>,
    for<'a> Self: Mul<i16, Output = Self> + Mul<&'a i16, Output = Self>,
    for<'a> Self: Mul<i32, Output = Self> + Mul<&'a i32, Output = Self>,
    for<'a> Self: Mul<i64, Output = Self> + Mul<&'a i64, Output = Self>,
    for<'a> Self: Mul<i8, Output = Self> + Mul<&'a i8, Output = Self>,
    for<'a> Self: Mul<isize, Output = Self> + Mul<&'a isize, Output = Self>,
    for<'a> &'a Self: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a Self: Mul<i128, Output = Self> + Mul<&'a i128, Output = Self>,
    for<'a> &'a Self: Mul<i16, Output = Self> + Mul<&'a i16, Output = Self>,
    for<'a> &'a Self: Mul<i32, Output = Self> + Mul<&'a i32, Output = Self>,
    for<'a> &'a Self: Mul<i64, Output = Self> + Mul<&'a i64, Output = Self>,
    for<'a> &'a Self: Mul<i8, Output = Self> + Mul<&'a i8, Output = Self>,
    for<'a> i128: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> i16: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> i32: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> i64: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> i8: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> isize: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a Self: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a i128: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a i16: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a i32: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a i64: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a i8: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> &'a isize: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    for<'a> Self: MulAssign<Self>
        + MulAssign<&'a Self>
        + MulAssign<i128>
        + MulAssign<i16>
        + MulAssign<i32>
        + MulAssign<i64>
        + MulAssign<i8>
        + MulAssign<isize>,
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
    for<'a> Self: Rem<i128, Output = Self> + Rem<&'a i128, Output = Self>,
    for<'a> Self: Rem<i16, Output = Self> + Rem<&'a i16, Output = Self>,
    for<'a> Self: Rem<i32, Output = Self> + Rem<&'a i32, Output = Self>,
    for<'a> Self: Rem<i64, Output = Self> + Rem<&'a i64, Output = Self>,
    for<'a> Self: Rem<i8, Output = Self> + Rem<&'a i8, Output = Self>,
    for<'a> Self: Rem<isize, Output = Self> + Rem<&'a isize, Output = Self>,
    for<'a> &'a Self: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a Self: Rem<i128, Output = Self> + Rem<&'a i128, Output = Self>,
    for<'a> &'a Self: Rem<i16, Output = Self> + Rem<&'a i16, Output = Self>,
    for<'a> &'a Self: Rem<i32, Output = Self> + Rem<&'a i32, Output = Self>,
    for<'a> &'a Self: Rem<i64, Output = Self> + Rem<&'a i64, Output = Self>,
    for<'a> &'a Self: Rem<i8, Output = Self> + Rem<&'a i8, Output = Self>,
    for<'a> &'a Self: Rem<isize, Output = Self> + Rem<&'a isize, Output = Self>,
    for<'a> i128: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> i16: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> i32: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> i64: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> i8: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a Self: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a i128: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a i16: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a i32: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a i64: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a i8: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> &'a isize: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> Self: RemAssign<Self>
        + RemAssign<&'a Self>
        + RemAssign<i128>
        + RemAssign<i16>
        + RemAssign<i32>
        + RemAssign<i64>
        + RemAssign<i8>
        + RemAssign<isize>,
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
    for<'a> Self: Sub<i128, Output = Self> + Sub<&'a i128, Output = Self>,
    for<'a> Self: Sub<i16, Output = Self> + Sub<&'a i16, Output = Self>,
    for<'a> Self: Sub<i32, Output = Self> + Sub<&'a i32, Output = Self>,
    for<'a> Self: Sub<i64, Output = Self> + Sub<&'a i64, Output = Self>,
    for<'a> Self: Sub<i8, Output = Self> + Sub<&'a i8, Output = Self>,
    for<'a> Self: Sub<isize, Output = Self> + Sub<&'a isize, Output = Self>,
    for<'a> &'a Self: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> &'a Self: Sub<i128, Output = Self> + Sub<&'a i128, Output = Self>,
    for<'a> &'a Self: Sub<i16, Output = Self> + Sub<&'a i16, Output = Self>,
    for<'a> &'a Self: Sub<i32, Output = Self> + Sub<&'a i32, Output = Self>,
    for<'a> &'a Self: Sub<i64, Output = Self> + Sub<&'a i64, Output = Self>,
    for<'a> &'a Self: Sub<i8, Output = Self> + Sub<&'a i8, Output = Self>,
    for<'a> &'a Self: Sub<isize, Output = Self> + Sub<&'a isize, Output = Self>,
    for<'a> i128: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> i16: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> i32: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> i64: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> i8: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> isize: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> &'a i128: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> &'a i16: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> &'a i32: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> &'a i64: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> &'a i8: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> &'a isize: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> Self: SubAssign<Self>
        + SubAssign<&'a Self>
        + SubAssign<i128>
        + SubAssign<i16>
        + SubAssign<i32>
        + SubAssign<i64>
        + SubAssign<i8>
        + SubAssign<isize>,
{
    /// Creates and initializes a [`BigInteger`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    fn new(sign: Sign, digits: Vec<u32>) -> Self;

    /// Creates and initializes a [`BigInteger`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    fn from_biguint(sign: Sign, data: BigUint) -> Self;

    /// Creates and initializes a [`BigInteger`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    fn from_slice(sign: Sign, slice: &[u32]) -> Self;

    /// Reinitializes a [`BigInteger`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    fn assign_from_slice(&mut self, sign: Sign, slice: &[u32]);

    /// Creates and initializes a [`BigInteger`].
    ///
    /// The bytes are in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, Sign};
    ///
    /// assert_eq!(BigInt::from_bytes_be(Sign::Plus, b"A"),
    ///            BigInt::parse_bytes(b"65", 10).unwrap());
    /// assert_eq!(BigInt::from_bytes_be(Sign::Plus, b"AA"),
    ///            BigInt::parse_bytes(b"16705", 10).unwrap());
    /// assert_eq!(BigInt::from_bytes_be(Sign::Plus, b"AB"),
    ///            BigInt::parse_bytes(b"16706", 10).unwrap());
    /// assert_eq!(BigInt::from_bytes_be(Sign::Plus, b"Hello world!"),
    ///            BigInt::parse_bytes(b"22405534230753963835153736737", 10).unwrap());
    /// ```
    fn from_bytes_be(sign: Sign, bytes: &[u8]) -> Self;

    /// Creates and initializes a [`BigInteger`].
    ///
    /// The bytes are in little-endian byte order.
    fn from_bytes_le(sign: Sign, bytes: &[u8]) -> Self;

    /// Creates and initializes a [`BigInteger`] from an array of bytes in
    /// two's complement binary representation.
    ///
    /// The digits are in big-endian base 2<sup>8</sup>.
    fn from_signed_bytes_be(digits: &[u8]) -> Self;

    /// Creates and initializes a [`BigInteger`] from an array of bytes in two's complement.
    ///
    /// The digits are in little-endian base 2<sup>8</sup>.
    fn from_signed_bytes_le(digits: &[u8]) -> Self;

    /// Creates and initializes a [`BigInteger`].
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, ToBigInt};
    ///
    /// assert_eq!(BigInt::parse_bytes(b"1234", 10), ToBigInt::to_bigint(&1234));
    /// assert_eq!(BigInt::parse_bytes(b"ABCD", 16), ToBigInt::to_bigint(&0xABCD));
    /// assert_eq!(BigInt::parse_bytes(b"G", 16), None);
    /// ```
    fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self>;

    /// Creates and initializes a [`BigInteger`]. Each `u8` of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in big-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, Sign};
    ///
    /// let inbase190 = vec![15, 33, 125, 12, 14];
    /// let a = BigInt::from_radix_be(Sign::Minus, &inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_be(190), (Sign:: Minus, inbase190));
    /// ```
    fn from_radix_be(sign: Sign, buf: &[u8], radix: u32) -> Option<Self>;

    /// Creates and initializes a [`BigInteger`]. Each `u8` of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in little-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, Sign};
    ///
    /// let inbase190 = vec![14, 12, 125, 33, 15];
    /// let a = BigInt::from_radix_be(Sign::Minus, &inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_be(190), (Sign::Minus, inbase190));
    /// ```
    fn from_radix_le(sign: Sign, buf: &[u8], radix: u32) -> Option<Self>;

    /// Returns the sign and the byte representation of the [`BigInteger`] in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{ToBigInt, Sign};
    ///
    /// let i = -1125.to_bigint().unwrap();
    /// assert_eq!(i.to_bytes_be(), (Sign::Minus, vec![4, 101]));
    /// ```
    fn to_bytes_be(&self) -> (Sign, Vec<u8>);

    /// Returns the sign and the byte representation of the [`BigInteger`] in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{ToBigInt, Sign};
    ///
    /// let i = -1125.to_bigint().unwrap();
    /// assert_eq!(i.to_bytes_le(), (Sign::Minus, vec![101, 4]));
    /// ```
    fn to_bytes_le(&self) -> (Sign, Vec<u8>);

    /// Returns the sign and the `u32` digits representation of the [`BigInteger`] ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, Sign};
    ///
    /// assert_eq!(BigInt::from(-1125).to_u32_digits(), (Sign::Minus, vec![1125]));
    /// assert_eq!(BigInt::from(4294967295u32).to_u32_digits(), (Sign::Plus, vec![4294967295]));
    /// assert_eq!(BigInt::from(4294967296u64).to_u32_digits(), (Sign::Plus, vec![0, 1]));
    /// assert_eq!(BigInt::from(-112500000000i64).to_u32_digits(), (Sign::Minus, vec![830850304, 26]));
    /// assert_eq!(BigInt::from(112500000000i64).to_u32_digits(), (Sign::Plus, vec![830850304, 26]));
    /// ```
    fn to_u32_digits(&self) -> (Sign, Vec<u32>);

    /// Returns the sign and the `u64` digits representation of the [`BigInteger`] ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, Sign};
    ///
    /// assert_eq!(BigInt::from(-1125).to_u64_digits(), (Sign::Minus, vec![1125]));
    /// assert_eq!(BigInt::from(4294967295u32).to_u64_digits(), (Sign::Plus, vec![4294967295]));
    /// assert_eq!(BigInt::from(4294967296u64).to_u64_digits(), (Sign::Plus, vec![4294967296]));
    /// assert_eq!(BigInt::from(-112500000000i64).to_u64_digits(), (Sign::Minus, vec![112500000000]));
    /// assert_eq!(BigInt::from(112500000000i64).to_u64_digits(), (Sign::Plus, vec![112500000000]));
    /// assert_eq!(BigInt::from(1u128 << 64).to_u64_digits(), (Sign::Plus, vec![0, 1]));
    /// ```
    fn to_u64_digits(&self) -> (Sign, Vec<u64>);

    /// Returns an iterator of `u32` digits representation of the [`BigInteger`] ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigInt;
    ///
    /// assert_eq!(BigInt::from(-1125).iter_u32_digits().collect::<Vec<u32>>(), vec![1125]);
    /// assert_eq!(BigInt::from(4294967295u32).iter_u32_digits().collect::<Vec<u32>>(), vec![4294967295]);
    /// assert_eq!(BigInt::from(4294967296u64).iter_u32_digits().collect::<Vec<u32>>(), vec![0, 1]);
    /// assert_eq!(BigInt::from(-112500000000i64).iter_u32_digits().collect::<Vec<u32>>(), vec![830850304, 26]);
    /// assert_eq!(BigInt::from(112500000000i64).iter_u32_digits().collect::<Vec<u32>>(), vec![830850304, 26]);
    /// ```
    fn iter_u32_digits(
        &self,
    ) -> impl DoubleEndedIterator<Item = u32> + ExactSizeIterator<Item = u32> + '_;

    /// Returns an iterator of `u64` digits representation of the [`BigInteger`] ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigInt;
    ///
    /// assert_eq!(BigInt::from(-1125).iter_u64_digits().collect::<Vec<u64>>(), vec![1125u64]);
    /// assert_eq!(BigInt::from(4294967295u32).iter_u64_digits().collect::<Vec<u64>>(), vec![4294967295u64]);
    /// assert_eq!(BigInt::from(4294967296u64).iter_u64_digits().collect::<Vec<u64>>(), vec![4294967296u64]);
    /// assert_eq!(BigInt::from(-112500000000i64).iter_u64_digits().collect::<Vec<u64>>(), vec![112500000000u64]);
    /// assert_eq!(BigInt::from(112500000000i64).iter_u64_digits().collect::<Vec<u64>>(), vec![112500000000u64]);
    /// assert_eq!(BigInt::from(1u128 << 64).iter_u64_digits().collect::<Vec<u64>>(), vec![0, 1]);
    /// ```
    fn iter_u64_digits(
        &self,
    ) -> impl DoubleEndedIterator<Item = u64> + ExactSizeIterator<Item = u64> + '_;

    /// Returns the two's-complement byte representation of the [`BigInteger`] in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::ToBigInt;
    ///
    /// let i = -1125.to_bigint().unwrap();
    /// assert_eq!(i.to_signed_bytes_be(), vec![251, 155]);
    /// ```
    fn to_signed_bytes_be(&self) -> Vec<u8>;

    /// Returns the two's-complement byte representation of the [`BigInteger`] in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::ToBigInt;
    ///
    /// let i = -1125.to_bigint().unwrap();
    /// assert_eq!(i.to_signed_bytes_le(), vec![155, 251]);
    /// ```
    fn to_signed_bytes_le(&self) -> Vec<u8>;

    /// Returns the integer formatted as a string in the given radix.
    /// `radix` must be in the range `2...36`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigInt;
    ///
    /// let i = BigInt::parse_bytes(b"ff", 16).unwrap();
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
    /// use num_bigint::{BigInt, Sign};
    ///
    /// assert_eq!(BigInt::from(-0xFFFFi64).to_radix_be(159),
    ///            (Sign::Minus, vec![2, 94, 27]));
    /// // 0xFFFF = 65535 = 2*(159^2) + 94*159 + 27
    /// ```
    fn to_radix_be(&self, radix: u32) -> (Sign, Vec<u8>);

    /// Returns the integer in the requested base in little-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based `u8` number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, Sign};
    ///
    /// assert_eq!(BigInt::from(-0xFFFFi64).to_radix_le(159),
    ///            (Sign::Minus, vec![27, 94, 2]));
    /// // 0xFFFF = 65535 = 27 + 94*159 + 2*(159^2)
    /// ```
    fn to_radix_le(&self, radix: u32) -> (Sign, Vec<u8>);

    /// Returns the sign of the [`BigInteger`] as a [`Sign`].
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, Sign};
    ///
    /// assert_eq!(BigInt::from(1234).sign(), Sign::Plus);
    /// assert_eq!(BigInt::from(-4321).sign(), Sign::Minus);
    /// assert_eq!(BigInt::ZERO.sign(), Sign::NoSign);
    /// ```
    fn sign(&self) -> Sign;

    /// Convert this [`BigInteger`] into its [`Sign`] and [`BigUint`] magnitude,
    /// the reverse of [`BigInt::from_biguint()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, BigUint, Sign};
    ///
    /// assert_eq!(BigInt::from(1234).into_parts(), (Sign::Plus, BigUint::from(1234u32)));
    /// assert_eq!(BigInt::from(-4321).into_parts(), (Sign::Minus, BigUint::from(4321u32)));
    /// assert_eq!(BigInt::ZERO.into_parts(), (Sign::NoSign, BigUint::ZERO));
    /// ```
    fn into_parts(self) -> (Sign, BigUint);

    /// Determines the fewest bits necessary to express the [`BigInteger`],
    /// not including the sign.
    fn bits(&self) -> u64;

    /// Converts this [`BigInteger`] into a [`BigUint`], if it's not negative.
    fn to_biguint(&self) -> Option<BigUint>;

    fn checked_add(&self, v: &Self) -> Option<Self> {
        Some(self + v)
    }

    fn checked_sub(&self, v: &Self) -> Option<Self> {
        Some(self - v)
    }

    fn checked_mul(&self, v: &Self) -> Option<Self> {
        Some(self * v)
    }

    fn checked_div(&self, v: &Self) -> Option<Self> {
        if v.is_zero() {
            return None;
        }
        Some(self / v)
    }

    /// Returns `self ^ exponent`.
    fn pow(&self, exponent: u32) -> Self;

    /// Returns `(self ^ exponent) mod modulus`
    ///
    /// Note that this rounds like `mod_floor`, not like the `%` operator,
    /// which makes a difference when given a negative `self` or `modulus`.
    /// The result will be in the interval `[0, modulus)` for `modulus > 0`,
    /// or in the interval `(modulus, 0]` for `modulus < 0`
    ///
    /// Panics if the exponent is negative or the modulus is zero.
    fn modpow(&self, exponent: &Self, modulus: &Self) -> Self;

    /// Returns the modular multiplicative inverse if it exists, otherwise `None`.
    ///
    /// This solves for `x` such that `self * x ≡ 1 (mod modulus)`.
    /// Note that this rounds like `mod_floor`, not like the `%` operator,
    /// which makes a difference when given a negative `self` or `modulus`.
    /// The solution will be in the interval `[0, modulus)` for `modulus > 0`,
    /// or in the interval `(modulus, 0]` for `modulus < 0`,
    /// and it exists if and only if `gcd(self, modulus) == 1`.
    ///
    /// ```
    /// use num_bigint::BigInt;
    /// use num_integer::Integer;
    /// use num_traits::{One, Zero};
    ///
    /// let m = BigInt::from(383);
    ///
    /// // Trivial cases
    /// assert_eq!(BigInt::zero().modinv(&m), None);
    /// assert_eq!(BigInt::one().modinv(&m), Some(BigInt::one()));
    /// let neg1 = &m - 1u32;
    /// assert_eq!(neg1.modinv(&m), Some(neg1));
    ///
    /// // Positive self and modulus
    /// let a = BigInt::from(271);
    /// let x = a.modinv(&m).unwrap();
    /// assert_eq!(x, BigInt::from(106));
    /// assert_eq!(x.modinv(&m).unwrap(), a);
    /// assert_eq!((&a * x).mod_floor(&m), BigInt::one());
    ///
    /// // Negative self and positive modulus
    /// let b = -&a;
    /// let x = b.modinv(&m).unwrap();
    /// assert_eq!(x, BigInt::from(277));
    /// assert_eq!((&b * x).mod_floor(&m), BigInt::one());
    ///
    /// // Positive self and negative modulus
    /// let n = -&m;
    /// let x = a.modinv(&n).unwrap();
    /// assert_eq!(x, BigInt::from(-277));
    /// assert_eq!((&a * x).mod_floor(&n), &n + 1);
    ///
    /// // Negative self and modulus
    /// let x = b.modinv(&n).unwrap();
    /// assert_eq!(x, BigInt::from(-106));
    /// assert_eq!((&b * x).mod_floor(&n), &n + 1);
    /// ```
    fn modinv(&self, modulus: &Self) -> Option<Self>;

    /// Returns the truncated principal square root of `self` --
    /// see [`num_integer::Roots::sqrt()`].
    fn sqrt(&self) -> Self {
        Roots::sqrt(self)
    }

    /// Returns the truncated principal cube root of `self` --
    /// see [`num_integer::Roots::cbrt()`].
    fn cbrt(&self) -> Self {
        Roots::cbrt(self)
    }

    /// Returns the truncated principal `n`th root of `self` --
    /// See [`num_integer::Roots::nth_root()`].
    fn nth_root(&self, n: u32) -> Self {
        Roots::nth_root(self, n)
    }

    /// Returns the number of least-significant bits that are zero,
    /// or `None` if the entire number is zero.
    fn trailing_zeros(&self) -> Option<u64>;

    /// Returns whether the bit in position `bit` is set,
    /// using the two's complement for negative numbers
    fn bit(&self, bit: u64) -> bool;

    /// Sets or clears the bit in the given position,
    /// using the two's complement for negative numbers
    ///
    /// Note that setting/clearing a bit (for positive/negative numbers,
    /// respectively) greater than the current bit length, a reallocation
    /// may be needed to store the new digits
    fn set_bit(&mut self, bit: u64, value: bool);
}

macro_rules! impl_big_integer {
    ($t:ty, $(<$lifetime_decl:lifetime>)?, $(<$lifetime_impl:lifetime>)?) => {
        impl$(<$lifetime_decl>)? BigInteger for $t {
            fn new(sign: Sign, digits: Vec<u32>) -> Self {
                Self::new(sign, digits)
            }

            fn from_biguint(sign: Sign, data: BigUint) -> Self {
                Self::from_biguint(sign, data)
            }

            fn from_slice(sign: Sign, slice: &[u32]) -> Self {
                Self::from_biguint(sign, BigUint::from_slice(slice))
            }

            fn assign_from_slice(&mut self, sign: Sign, slice: &[u32]) {
                *self = Self::from_slice(sign, slice);
            }

            fn from_bytes_be(sign: Sign, bytes: &[u8]) -> Self {
                Self::from_bytes_be(sign, bytes)
            }

            fn from_bytes_le(sign: Sign, bytes: &[u8]) -> Self {
                Self::from_bytes_le(sign, bytes)
            }

            fn from_signed_bytes_be(digits: &[u8]) -> Self {
                Self::from_signed_bytes_be(digits)
            }

            fn from_signed_bytes_le(digits: &[u8]) -> Self {
                Self::from_signed_bytes_le(digits)
            }

            fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self> {
                Self::parse_bytes(buf, radix)
            }

            fn from_radix_be(sign: Sign, buf: &[u8], radix: u32) -> Option<Self> {
                Self::from_radix_be(sign, buf, radix)
            }

            fn from_radix_le(sign: Sign, buf: &[u8], radix: u32) -> Option<Self> {
                Self::from_radix_le(sign, buf, radix)
            }

            fn to_bytes_be(&self) -> (Sign, Vec<u8>) {
                self.to_bytes_be()
            }

            fn to_bytes_le(&self) -> (Sign, Vec<u8>) {
                self.to_bytes_le()
            }

            fn to_u32_digits(&self) -> (Sign, Vec<u32>) {
                self.to_u32_digits()
            }

            fn to_u64_digits(&self) -> (Sign, Vec<u64>) {
                self.to_u64_digits()
            }

            fn iter_u32_digits(
                &self,
            ) -> impl DoubleEndedIterator<Item = u32> + ExactSizeIterator<Item = u32> + '_
            {
                self.iter_u32_digits()
            }

            fn iter_u64_digits(
                &self,
            ) -> impl DoubleEndedIterator<Item = u64> + ExactSizeIterator<Item = u64> + '_
            {
                self.iter_u64_digits()
            }

            fn to_signed_bytes_be(&self) -> Vec<u8> {
                self.to_signed_bytes_be()
            }

            fn to_signed_bytes_le(&self) -> Vec<u8> {
                self.to_signed_bytes_le()
            }

            fn to_str_radix(&self, radix: u32) -> String {
                self.to_str_radix(radix)
            }

            fn to_radix_be(&self, radix: u32) -> (Sign, Vec<u8>) {
                self.to_radix_be(radix)
            }

            fn to_radix_le(&self, radix: u32) -> (Sign, Vec<u8>) {
                self.to_radix_le(radix)
            }

            fn sign(&self) -> Sign {
                self.sign()
            }

            fn into_parts(self) -> (Sign, BigUint) {
                self.into_parts()
            }

            fn bits(&self) -> u64 {
                self.bits()
            }

            fn to_biguint(&self) -> Option<BigUint> {
                self.to_biguint()
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
        }
    };
}

impl_big_integer!(BigInt, , );
impl_big_integer!(CBigInt<'static>, ,);
