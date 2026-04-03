use std::fmt::{Binary, Debug, Display, LowerHex, Octal, UpperHex};
use std::hash::Hash;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::str::FromStr;

use num_bigint::{BigInt, BigUint, ParseBigIntError, RandomBits, ToBigInt, ToBigUint};
use num_integer::{Integer, Roots};
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedEuclid, CheckedMul, CheckedSub, ConstZero, Euclid, FromBytes,
    FromPrimitive, Num, One, Pow, ToBytes, ToPrimitive, Zero,
};
use rand::distributions::uniform::SampleUniform;
use rand::prelude::Distribution;
use serde::{Deserialize, Serialize};

pub trait BigNumber
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
    // // TryFrom
    Self: TryInto<u128>,
    Self: TryInto<u16>,
    Self: TryInto<u32>,
    Self: TryInto<u64>,
    Self: TryInto<u8>,
    Self: TryInto<usize>,
    // for<'a> u128: TryFrom<Self> + TryFrom<&'a Self>,
    // for<'a> u16: TryFrom<Self> + TryFrom<&'a Self>,
    // for<'a> u32: TryFrom<Self> + TryFrom<&'a Self>,
    // for<'a> u64: TryFrom<Self> + TryFrom<&'a Self>,
    // for<'a> u8: TryFrom<Self> + TryFrom<&'a Self>,
    // for<'a> usize: TryFrom<Self> + TryFrom<&'a Self>,
    // Add
    for<'a> Self: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    for<'a> Self: Add<u128, Output = Self> + Add<&'a u128, Output = Self>,
    for<'a> Self: Add<u16, Output = Self> + Add<&'a u16, Output = Self>,
    for<'a> Self: Add<u32, Output = Self> + Add<&'a u32, Output = Self>,
    for<'a> Self: Add<u64, Output = Self> + Add<&'a u64, Output = Self>,
    for<'a> Self: Add<u8, Output = Self> + Add<&'a u8, Output = Self>,
    for<'a> Self: Add<usize, Output = Self> + Add<&'a usize, Output = Self>,
    // for<'a> &'a Self: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> &'a Self: Add<u128, Output = Self> + Add<&'a u128, Output = Self>,
    // for<'a> &'a Self: Add<u16, Output = Self> + Add<&'a u16, Output = Self>,
    // for<'a> &'a Self: Add<u32, Output = Self> + Add<&'a u32, Output = Self>,
    // for<'a> &'a Self: Add<u64, Output = Self> + Add<&'a u64, Output = Self>,
    // for<'a> &'a Self: Add<u8, Output = Self> + Add<&'a u8, Output = Self>,
    // for<'a> &'a Self: Add<usize, Output = Self> + Add<&'a usize, Output = Self>,
    // for<'a> u128: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> u16: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> u32: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> u64: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> u8: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> usize: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> &'a u128: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> &'a u16: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> &'a u32: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> &'a u64: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> &'a u8: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> &'a usize: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
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
    // for<'a> &'a Self: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> &'a Self: Add<u128, Output = Self> + Add<&'a u128, Output = Self>,
    // for<'a> &'a Self: Add<u16, Output = Self> + Add<&'a u16, Output = Self>,
    // for<'a> &'a Self: Add<u32, Output = Self> + Add<&'a u32, Output = Self>,
    // for<'a> &'a Self: Add<u64, Output = Self> + Add<&'a u64, Output = Self>,
    // for<'a> &'a Self: Add<u8, Output = Self> + Add<&'a u8, Output = Self>,
    // for<'a> &'a Self: Add<usize, Output = Self> + Add<&'a usize, Output = Self>,
    // for<'a> u128: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> u16: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> u32: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> u64: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> u8: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> usize: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> &'a u128: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> &'a u16: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> &'a u32: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> &'a u64: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> &'a u8: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
    // for<'a> &'a usize: Add<Self, Output = Self> + Add<&'a Self, Output = Self>,
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
    // for<'a> &'a Self: BitAnd<Self, Output = Self> + BitAnd<&'a Self, Output = Self>,
    for<'a> Self: BitAndAssign<Self> + BitAndAssign<&'a Self>,
    // BitOr
    for<'a> Self: BitOr<Self, Output = Self> + BitOr<&'a Self, Output = Self>,
    // for<'a> &'a Self: BitOr<Self, Output = Self> + BitOr<&'a Self, Output = Self>,
    for<'a> Self: BitOrAssign<Self> + BitOrAssign<&'a Self>,
    // BitXor
    for<'a> Self: BitXor<Self, Output = Self> + BitXor<&'a Self, Output = Self>,
    // for<'a> &'a Self: BitXor<Self, Output = Self> + BitXor<&'a Self, Output = Self>,
    for<'a> Self: BitXorAssign<Self> + BitXorAssign<&'a Self>,
    // Div
    for<'a> Self: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    for<'a> Self: Div<u128, Output = Self> + Div<&'a u128, Output = Self>,
    for<'a> Self: Div<u16, Output = Self> + Div<&'a u16, Output = Self>,
    for<'a> Self: Div<u32, Output = Self> + Div<&'a u32, Output = Self>,
    for<'a> Self: Div<u64, Output = Self> + Div<&'a u64, Output = Self>,
    for<'a> Self: Div<u8, Output = Self> + Div<&'a u8, Output = Self>,
    for<'a> Self: Div<usize, Output = Self> + Div<&'a usize, Output = Self>,
    // for<'a> &'a Self: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    // for<'a> &'a Self: Div<u128, Output = Self> + Div<&'a u128, Output = Self>,
    // for<'a> &'a Self: Div<u16, Output = Self> + Div<&'a u16, Output = Self>,
    // for<'a> &'a Self: Div<u32, Output = Self> + Div<&'a u32, Output = Self>,
    // for<'a> &'a Self: Div<u64, Output = Self> + Div<&'a u64, Output = Self>,
    // for<'a> &'a Self: Div<u8, Output = Self> + Div<&'a u8, Output = Self>,
    // for<'a> &'a Self: Div<usize, Output = Self> + Div<&'a usize, Output = Self>,
    // for<'a> u128: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    // for<'a> u16: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    // for<'a> u32: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    // for<'a> u64: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    // for<'a> u8: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    // for<'a> usize: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    // for<'a> &'a u128: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    // for<'a> &'a u16: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    // for<'a> &'a u32: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    // for<'a> &'a u64: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    // for<'a> &'a u8: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
    // for<'a> &'a usize: Div<Self, Output = Self> + Div<&'a Self, Output = Self>,
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
    // for<'a> &'a Self: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    // for<'a> &'a Self: Mul<u128, Output = Self> + Mul<&'a u128, Output = Self>,
    // for<'a> &'a Self: Mul<u16, Output = Self> + Mul<&'a u16, Output = Self>,
    // for<'a> &'a Self: Mul<u32, Output = Self> + Mul<&'a u32, Output = Self>,
    // for<'a> &'a Self: Mul<u64, Output = Self> + Mul<&'a u64, Output = Self>,
    // for<'a> &'a Self: Mul<u8, Output = Self> + Mul<&'a u8, Output = Self>,
    // for<'a> u128: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    // for<'a> u16: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    // for<'a> u32: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    // for<'a> u64: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    // for<'a> u8: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    // for<'a> usize: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    // for<'a> &'a Self: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    // for<'a> &'a u128: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    // for<'a> &'a u16: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    // for<'a> &'a u32: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    // for<'a> &'a u64: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    // for<'a> &'a u8: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
    // for<'a> &'a usize: Mul<Self, Output = Self> + Mul<&'a Self, Output = Self>,
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
    // for<'a> &'a Self: Pow<u128, Output = Self> + Pow<&'a u128, Output = Self>,
    // for<'a> &'a Self: Pow<u16, Output = Self> + Pow<&'a u16, Output = Self>,
    // for<'a> &'a Self: Pow<u32, Output = Self> + Pow<&'a u32, Output = Self>,
    // for<'a> &'a Self: Pow<u64, Output = Self> + Pow<&'a u64, Output = Self>,
    // for<'a> &'a Self: Pow<u8, Output = Self> + Pow<&'a u8, Output = Self>,
    // for<'a> &'a Self: Pow<usize, Output = Self> + Pow<&'a usize, Output = Self>,
    // Rem
    for<'a> Self: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    for<'a> Self: Rem<u128, Output = Self> + Rem<&'a u128, Output = Self>,
    for<'a> Self: Rem<u16, Output = Self> + Rem<&'a u16, Output = Self>,
    for<'a> Self: Rem<u32, Output = Self> + Rem<&'a u32, Output = Self>,
    for<'a> Self: Rem<u64, Output = Self> + Rem<&'a u64, Output = Self>,
    for<'a> Self: Rem<u8, Output = Self> + Rem<&'a u8, Output = Self>,
    for<'a> Self: Rem<usize, Output = Self> + Rem<&'a usize, Output = Self>,
    // for<'a> &'a Self: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    // for<'a> &'a Self: Rem<u128, Output = Self> + Rem<&'a u128, Output = Self>,
    // for<'a> &'a Self: Rem<u16, Output = Self> + Rem<&'a u16, Output = Self>,
    // for<'a> &'a Self: Rem<u32, Output = Self> + Rem<&'a u32, Output = Self>,
    // for<'a> &'a Self: Rem<u64, Output = Self> + Rem<&'a u64, Output = Self>,
    // for<'a> &'a Self: Rem<u8, Output = Self> + Rem<&'a u8, Output = Self>,
    // for<'a> &'a Self: Rem<usize, Output = Self> + Rem<&'a usize, Output = Self>,
    // for<'a> u128: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    // for<'a> u16: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    // for<'a> u32: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    // for<'a> u64: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    // for<'a> u8: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    // for<'a> &'a Self: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    // for<'a> &'a u128: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    // for<'a> &'a u16: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    // for<'a> &'a u32: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    // for<'a> &'a u64: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    // for<'a> &'a u8: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
    // for<'a> &'a usize: Rem<Self, Output = Self> + Rem<&'a Self, Output = Self>,
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
    // for<'a> &'a Self: Shl<i128, Output = Self>
    //     + Shl<i16, Output = Self>
    //     + Shl<i32, Output = Self>
    //     + Shl<i64, Output = Self>
    //     + Shl<i8, Output = Self>
    //     + Shl<isize, Output = Self>
    //     + Shl<u128, Output = Self>
    //     + Shl<u16, Output = Self>
    //     + Shl<u32, Output = Self>
    //     + Shl<u64, Output = Self>
    //     + Shl<u8, Output = Self>
    //     + Shl<usize, Output = Self>
    //     + Shl<&'a i128, Output = Self>
    //     + Shl<&'a i16, Output = Self>
    //     + Shl<&'a i32, Output = Self>
    //     + Shl<&'a i64, Output = Self>
    //     + Shl<&'a i8, Output = Self>
    //     + Shl<&'a isize, Output = Self>
    //     + Shl<&'a u128, Output = Self>
    //     + Shl<&'a u16, Output = Self>
    //     + Shl<&'a u32, Output = Self>
    //     + Shl<&'a u64, Output = Self>
    //     + Shl<&'a u8, Output = Self>
    //     + Shl<&'a usize, Output = Self>,
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
    // for<'a> &'a Self: Shr<i128, Output = Self>
    //     + Shr<i16, Output = Self>
    //     + Shr<i32, Output = Self>
    //     + Shr<i64, Output = Self>
    //     + Shr<i8, Output = Self>
    //     + Shr<isize, Output = Self>
    //     + Shr<u128, Output = Self>
    //     + Shr<u16, Output = Self>
    //     + Shr<u32, Output = Self>
    //     + Shr<u64, Output = Self>
    //     + Shr<u8, Output = Self>
    //     + Shr<usize, Output = Self>
    //     + Shr<&'a i128, Output = Self>
    //     + Shr<&'a i16, Output = Self>
    //     + Shr<&'a i32, Output = Self>
    //     + Shr<&'a i64, Output = Self>
    //     + Shr<&'a i8, Output = Self>
    //     + Shr<&'a isize, Output = Self>
    //     + Shr<&'a u128, Output = Self>
    //     + Shr<&'a u16, Output = Self>
    //     + Shr<&'a u32, Output = Self>
    //     + Shr<&'a u64, Output = Self>
    //     + Shr<&'a u8, Output = Self>
    //     + Shr<&'a usize, Output = Self>,
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
    // for<'a> &'a Self: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    // for<'a> &'a Self: Sub<u128, Output = Self> + Sub<&'a u128, Output = Self>,
    // for<'a> &'a Self: Sub<u16, Output = Self> + Sub<&'a u16, Output = Self>,
    // for<'a> &'a Self: Sub<u32, Output = Self> + Sub<&'a u32, Output = Self>,
    // for<'a> &'a Self: Sub<u64, Output = Self> + Sub<&'a u64, Output = Self>,
    // for<'a> &'a Self: Sub<u8, Output = Self> + Sub<&'a u8, Output = Self>,
    // for<'a> &'a Self: Sub<usize, Output = Self> + Sub<&'a usize, Output = Self>,
    // for<'a> u128: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    // for<'a> u16: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    // for<'a> u32: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    // for<'a> u64: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    // for<'a> u8: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    // for<'a> usize: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    // for<'a> &'a u128: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    // for<'a> &'a u16: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    // for<'a> &'a u32: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    // for<'a> &'a u64: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    // for<'a> &'a u8: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    // for<'a> &'a usize: Sub<Self, Output = Self> + Sub<&'a Self, Output = Self>,
    for<'a> Self: SubAssign<Self>
        + SubAssign<&'a Self>
        + SubAssign<u128>
        + SubAssign<u16>
        + SubAssign<u32>
        + SubAssign<u64>
        + SubAssign<u8>
        + SubAssign<usize>,
{
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

    /// Determines the fewest bits necessary to express the [`BigInteger`],
    /// not including the sign.
    fn bits(&self) -> u64;

    fn checked_add(&self, v: &Self) -> Option<Self>;

    fn checked_sub(&self, v: &Self) -> Option<Self>;

    fn checked_mul(&self, v: &Self) -> Option<Self>;

    fn checked_div(&self, v: &Self) -> Option<Self>;

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

#[macro_export]
macro_rules! impl_big_number {
    ($t:ty) => {
        impl BigNumber for $t {
            fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self> {
                Self::parse_bytes(buf, radix)
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

            fn bits(&self) -> u64 {
                self.bits()
            }

            fn pow(&self, exponent: u32) -> Self {
                self.pow(exponent)
            }

            fn checked_add(&self, v: &Self) -> Option<Self> {
                CheckedAdd::checked_add(self, v)
            }

            fn checked_sub(&self, v: &Self) -> Option<Self> {
                CheckedSub::checked_sub(self, v)
            }

            fn checked_mul(&self, v: &Self) -> Option<Self> {
                CheckedMul::checked_mul(self, v)
            }

            fn checked_div(&self, v: &Self) -> Option<Self> {
                CheckedDiv::checked_div(self, v)
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

impl_big_number!(BigInt);
impl_big_number!(BigUint);
