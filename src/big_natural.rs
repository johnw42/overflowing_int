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
    FromPrimitive, Num, One, Pow, ToBytes, ToPrimitive, Unsigned, Zero,
};
use rand::distributions::uniform::SampleUniform;
use rand::prelude::Distribution;
use serde::{Deserialize, Serialize};

use crate::BigNumber;

// A trait covering all the methods and trait bounds of BigUint.
pub trait BigNatural: BigNumber
where
    RandomBits: Distribution<Self>,
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
    Self: LowerHex,
    Self: Num<FromStrRadixErr = ParseBigIntError>,
    Self: Octal,
    Self: One,
    Self: Ord,
    Self: PartialEq,
    Self: PartialOrd,
    Self: RefUnwindSafe,
    Self: Roots,
    Self: SampleUniform,
    Self: Send,
    Self: Serialize,
    Self: Sync,
    Self: ToBigInt,
    Self: ToBigUint,
    Self: ToBytes,
    Self: ToPrimitive,
    Self: Unpin,
    Self: Unsigned,
    Self: UnwindSafe,
    Self: UpperHex,
    Self: Zero,
    Self: quickcheck::Arbitrary,
    for<'a> Self: arbitrary::Arbitrary<'a>,
    for<'de> Self: Deserialize<'de>,
    // From
    Self: From<BigUint>,
    Self: From<bool>,
    Self: From<u128>,
    Self: From<u16>,
    Self: From<u32>,
    Self: From<u64>,
    Self: From<u8>,
    Self: From<usize>,
    // TryInto
    Self: TryInto<u128>,
    Self: TryInto<u16>,
    Self: TryInto<u32>,
    Self: TryInto<u64>,
    Self: TryInto<u8>,
    Self: TryInto<usize>,
    for<'a> &'a Self: TryInto<u128>,
    for<'a> &'a Self: TryInto<u16>,
    for<'a> &'a Self: TryInto<u32>,
    for<'a> &'a Self: TryInto<u64>,
    for<'a> &'a Self: TryInto<u8>,
    for<'a> &'a Self: TryInto<usize>,
    // Add
    Self: Add<Self, Output = Self>,
    Self: Add<u128, Output = Self>,
    Self: Add<u16, Output = Self>,
    Self: Add<u32, Output = Self>,
    Self: Add<u64, Output = Self>,
    Self: Add<u8, Output = Self>,
    Self: Add<usize, Output = Self>,
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
    for<'a> &'a Self: Add<usize, Output = Self>,
    for<'a> &'a u128: Add<&'a Self, Output = Self>,
    for<'a> &'a u128: Add<Self, Output = Self>,
    for<'a> &'a u16: Add<&'a Self, Output = Self>,
    for<'a> &'a u16: Add<Self, Output = Self>,
    for<'a> &'a u32: Add<&'a Self, Output = Self>,
    for<'a> &'a u32: Add<Self, Output = Self>,
    for<'a> &'a u64: Add<&'a Self, Output = Self>,
    for<'a> &'a u64: Add<Self, Output = Self>,
    for<'a> &'a u8: Add<&'a Self, Output = Self>,
    for<'a> &'a u8: Add<Self, Output = Self>,
    for<'a> &'a usize: Add<&'a Self, Output = Self>,
    for<'a> &'a usize: Add<Self, Output = Self>,
    for<'a> Self: Add<&'a Self, Output = Self>,
    for<'a> Self: Add<&'a u128, Output = Self>,
    for<'a> Self: Add<&'a u16, Output = Self>,
    for<'a> Self: Add<&'a u32, Output = Self>,
    for<'a> Self: Add<&'a u64, Output = Self>,
    for<'a> Self: Add<&'a u8, Output = Self>,
    for<'a> Self: Add<&'a usize, Output = Self>,
    for<'a> u128: Add<&'a Self, Output = Self>,
    for<'a> u16: Add<&'a Self, Output = Self>,
    for<'a> u32: Add<&'a Self, Output = Self>,
    for<'a> u64: Add<&'a Self, Output = Self>,
    for<'a> u8: Add<&'a Self, Output = Self>,
    for<'a> usize: Add<&'a Self, Output = Self>,
    u128: Add<Self, Output = Self>,
    u16: Add<Self, Output = Self>,
    u32: Add<Self, Output = Self>,
    u64: Add<Self, Output = Self>,
    u8: Add<Self, Output = Self>,
    usize: Add<Self, Output = Self>,
    // AddAssign
    Self: AddAssign<Self>,
    Self: AddAssign<u128>,
    Self: AddAssign<u16>,
    Self: AddAssign<u32>,
    Self: AddAssign<u64>,
    Self: AddAssign<u8>,
    Self: AddAssign<usize>,
    for<'a> Self: AddAssign<&'a Self>,
    // BitAnd
    Self: BitAnd<Self, Output = Self>,
    for<'a> &'a Self: BitAnd<&'a Self, Output = Self>,
    for<'a> &'a Self: BitAnd<Self, Output = Self>,
    for<'a> Self: BitAnd<&'a Self, Output = Self>,
    // BitAndAssign
    Self: BitAndAssign<Self>,
    for<'a> Self: BitAndAssign<&'a Self>,
    // BitOr
    Self: BitOr<Self, Output = Self>,
    for<'a> &'a Self: BitOr<&'a Self, Output = Self>,
    for<'a> &'a Self: BitOr<Self, Output = Self>,
    for<'a> Self: BitOr<&'a Self, Output = Self>,
    // BitOrAssign
    Self: BitOrAssign<Self>,
    for<'a> Self: BitOrAssign<&'a Self>,
    // BitXor
    Self: BitXor<Self, Output = Self>,
    for<'a> &'a Self: BitXor<&'a Self, Output = Self>,
    for<'a> &'a Self: BitXor<Self, Output = Self>,
    for<'a> Self: BitXor<&'a Self, Output = Self>,
    // BitXorAssign
    Self: BitXorAssign<Self>,
    for<'a> Self: BitXorAssign<&'a Self>,
    // Div
    Self: Div<Self, Output = Self>,
    Self: Div<u128, Output = Self>,
    Self: Div<u16, Output = Self>,
    Self: Div<u32, Output = Self>,
    Self: Div<u64, Output = Self>,
    Self: Div<u8, Output = Self>,
    Self: Div<usize, Output = Self>,
    for<'a> &'a Self: Div<&'a Self, Output = Self>,
    for<'a> &'a Self: Div<&'a u128, Output = Self>,
    for<'a> &'a Self: Div<&'a u16, Output = Self>,
    for<'a> &'a Self: Div<&'a u32, Output = Self>,
    for<'a> &'a Self: Div<&'a u64, Output = Self>,
    for<'a> &'a Self: Div<&'a u8, Output = Self>,
    for<'a> &'a Self: Div<&'a usize, Output = Self>,
    for<'a> &'a Self: Div<Self, Output = Self>,
    for<'a> &'a Self: Div<u128, Output = Self>,
    for<'a> &'a Self: Div<u16, Output = Self>,
    for<'a> &'a Self: Div<u32, Output = Self>,
    for<'a> &'a Self: Div<u64, Output = Self>,
    for<'a> &'a Self: Div<u8, Output = Self>,
    for<'a> &'a Self: Div<usize, Output = Self>,
    for<'a> &'a u128: Div<&'a Self, Output = Self>,
    for<'a> &'a u128: Div<Self, Output = Self>,
    for<'a> &'a u16: Div<&'a Self, Output = Self>,
    for<'a> &'a u16: Div<Self, Output = Self>,
    for<'a> &'a u32: Div<&'a Self, Output = Self>,
    for<'a> &'a u32: Div<Self, Output = Self>,
    for<'a> &'a u64: Div<&'a Self, Output = Self>,
    for<'a> &'a u64: Div<Self, Output = Self>,
    for<'a> &'a u8: Div<&'a Self, Output = Self>,
    for<'a> &'a u8: Div<Self, Output = Self>,
    for<'a> &'a usize: Div<&'a Self, Output = Self>,
    for<'a> &'a usize: Div<Self, Output = Self>,
    for<'a> Self: Div<&'a Self, Output = Self>,
    for<'a> Self: Div<&'a u128, Output = Self>,
    for<'a> Self: Div<&'a u16, Output = Self>,
    for<'a> Self: Div<&'a u32, Output = Self>,
    for<'a> Self: Div<&'a u64, Output = Self>,
    for<'a> Self: Div<&'a u8, Output = Self>,
    for<'a> Self: Div<&'a usize, Output = Self>,
    for<'a> u128: Div<&'a Self, Output = Self>,
    for<'a> u16: Div<&'a Self, Output = Self>,
    for<'a> u32: Div<&'a Self, Output = Self>,
    for<'a> u64: Div<&'a Self, Output = Self>,
    for<'a> u8: Div<&'a Self, Output = Self>,
    for<'a> usize: Div<&'a Self, Output = Self>,
    u128: Div<Self, Output = Self>,
    u16: Div<Self, Output = Self>,
    u32: Div<Self, Output = Self>,
    u64: Div<Self, Output = Self>,
    u8: Div<Self, Output = Self>,
    usize: Div<Self, Output = Self>,
    // DivAssign
    Self: DivAssign<Self>,
    Self: DivAssign<u128>,
    Self: DivAssign<u16>,
    Self: DivAssign<u32>,
    Self: DivAssign<u64>,
    Self: DivAssign<u8>,
    Self: DivAssign<usize>,
    for<'a> Self: DivAssign<&'a Self>,
    // Mul
    Self: Mul<Self, Output = Self>,
    Self: Mul<u128, Output = Self>,
    Self: Mul<u16, Output = Self>,
    Self: Mul<u32, Output = Self>,
    Self: Mul<u64, Output = Self>,
    Self: Mul<u8, Output = Self>,
    Self: Mul<usize, Output = Self>,
    for<'a> &'a Self: Mul<&'a Self, Output = Self>,
    for<'a> &'a Self: Mul<&'a u128, Output = Self>,
    for<'a> &'a Self: Mul<&'a u16, Output = Self>,
    for<'a> &'a Self: Mul<&'a u32, Output = Self>,
    for<'a> &'a Self: Mul<&'a u64, Output = Self>,
    for<'a> &'a Self: Mul<&'a u8, Output = Self>,
    for<'a> &'a Self: Mul<Self, Output = Self>,
    for<'a> &'a Self: Mul<u128, Output = Self>,
    for<'a> &'a Self: Mul<u16, Output = Self>,
    for<'a> &'a Self: Mul<u32, Output = Self>,
    for<'a> &'a Self: Mul<u64, Output = Self>,
    for<'a> &'a Self: Mul<u8, Output = Self>,
    for<'a> &'a u128: Mul<&'a Self, Output = Self>,
    for<'a> &'a u128: Mul<Self, Output = Self>,
    for<'a> &'a u16: Mul<&'a Self, Output = Self>,
    for<'a> &'a u16: Mul<Self, Output = Self>,
    for<'a> &'a u32: Mul<&'a Self, Output = Self>,
    for<'a> &'a u32: Mul<Self, Output = Self>,
    for<'a> &'a u64: Mul<&'a Self, Output = Self>,
    for<'a> &'a u64: Mul<Self, Output = Self>,
    for<'a> &'a u8: Mul<&'a Self, Output = Self>,
    for<'a> &'a u8: Mul<Self, Output = Self>,
    for<'a> &'a usize: Mul<&'a Self, Output = Self>,
    for<'a> &'a usize: Mul<Self, Output = Self>,
    for<'a> Self: Mul<&'a Self, Output = Self>,
    for<'a> Self: Mul<&'a u128, Output = Self>,
    for<'a> Self: Mul<&'a u16, Output = Self>,
    for<'a> Self: Mul<&'a u32, Output = Self>,
    for<'a> Self: Mul<&'a u64, Output = Self>,
    for<'a> Self: Mul<&'a u8, Output = Self>,
    for<'a> Self: Mul<&'a usize, Output = Self>,
    for<'a> u128: Mul<&'a Self, Output = Self>,
    for<'a> u16: Mul<&'a Self, Output = Self>,
    for<'a> u32: Mul<&'a Self, Output = Self>,
    for<'a> u64: Mul<&'a Self, Output = Self>,
    for<'a> u8: Mul<&'a Self, Output = Self>,
    for<'a> usize: Mul<&'a Self, Output = Self>,
    u128: Mul<Self, Output = Self>,
    u16: Mul<Self, Output = Self>,
    u32: Mul<Self, Output = Self>,
    u64: Mul<Self, Output = Self>,
    u8: Mul<Self, Output = Self>,
    usize: Mul<Self, Output = Self>,
    // MulAssign
    Self: MulAssign<Self>,
    Self: MulAssign<u128>,
    Self: MulAssign<u16>,
    Self: MulAssign<u32>,
    Self: MulAssign<u64>,
    Self: MulAssign<u8>,
    Self: MulAssign<usize>,
    for<'a> Self: MulAssign<&'a Self>,
    // Pow
    Self: Pow<u128, Output = Self>,
    Self: Pow<u16, Output = Self>,
    Self: Pow<u32, Output = Self>,
    Self: Pow<u64, Output = Self>,
    Self: Pow<u8, Output = Self>,
    Self: Pow<usize, Output = Self>,
    for<'a> &'a Self: Pow<&'a u128, Output = Self>,
    for<'a> &'a Self: Pow<&'a u16, Output = Self>,
    for<'a> &'a Self: Pow<&'a u32, Output = Self>,
    for<'a> &'a Self: Pow<&'a u64, Output = Self>,
    for<'a> &'a Self: Pow<&'a u8, Output = Self>,
    for<'a> &'a Self: Pow<&'a usize, Output = Self>,
    for<'a> &'a Self: Pow<u128, Output = Self>,
    for<'a> &'a Self: Pow<u16, Output = Self>,
    for<'a> &'a Self: Pow<u32, Output = Self>,
    for<'a> &'a Self: Pow<u64, Output = Self>,
    for<'a> &'a Self: Pow<u8, Output = Self>,
    for<'a> &'a Self: Pow<usize, Output = Self>,
    for<'a> Self: Pow<&'a u128, Output = Self>,
    for<'a> Self: Pow<&'a u16, Output = Self>,
    for<'a> Self: Pow<&'a u32, Output = Self>,
    for<'a> Self: Pow<&'a u64, Output = Self>,
    for<'a> Self: Pow<&'a u8, Output = Self>,
    for<'a> Self: Pow<&'a usize, Output = Self>,
    // Rem
    Self: Rem<Self, Output = Self>,
    Self: Rem<u128, Output = Self>,
    Self: Rem<u16, Output = Self>,
    Self: Rem<u32, Output = Self>,
    Self: Rem<u64, Output = Self>,
    Self: Rem<u8, Output = Self>,
    Self: Rem<usize, Output = Self>,
    for<'a> &'a Self: Rem<&'a Self, Output = Self>,
    for<'a> &'a Self: Rem<&'a u128, Output = Self>,
    for<'a> &'a Self: Rem<&'a u16, Output = Self>,
    for<'a> &'a Self: Rem<&'a u32, Output = Self>,
    for<'a> &'a Self: Rem<&'a u64, Output = Self>,
    for<'a> &'a Self: Rem<&'a u8, Output = Self>,
    for<'a> &'a Self: Rem<&'a usize, Output = Self>,
    for<'a> &'a Self: Rem<Self, Output = Self>,
    for<'a> &'a Self: Rem<u128, Output = Self>,
    for<'a> &'a Self: Rem<u16, Output = Self>,
    for<'a> &'a Self: Rem<u32, Output = Self>,
    for<'a> &'a Self: Rem<u64, Output = Self>,
    for<'a> &'a Self: Rem<u8, Output = Self>,
    for<'a> &'a Self: Rem<usize, Output = Self>,
    for<'a> &'a u128: Rem<&'a Self, Output = Self>,
    for<'a> &'a u128: Rem<Self, Output = Self>,
    for<'a> &'a u16: Rem<&'a Self, Output = Self>,
    for<'a> &'a u16: Rem<Self, Output = Self>,
    for<'a> &'a u32: Rem<&'a Self, Output = Self>,
    for<'a> &'a u32: Rem<Self, Output = Self>,
    for<'a> &'a u64: Rem<&'a Self, Output = Self>,
    for<'a> &'a u64: Rem<Self, Output = Self>,
    for<'a> &'a u8: Rem<&'a Self, Output = Self>,
    for<'a> &'a u8: Rem<Self, Output = Self>,
    for<'a> &'a usize: Rem<&'a Self, Output = Self>,
    for<'a> &'a usize: Rem<Self, Output = Self>,
    for<'a> Self: Rem<&'a Self, Output = Self>,
    for<'a> Self: Rem<&'a u128, Output = Self>,
    for<'a> Self: Rem<&'a u16, Output = Self>,
    for<'a> Self: Rem<&'a u32, Output = Self>,
    for<'a> Self: Rem<&'a u64, Output = Self>,
    for<'a> Self: Rem<&'a u8, Output = Self>,
    for<'a> Self: Rem<&'a usize, Output = Self>,
    for<'a> u128: Rem<&'a Self, Output = Self>,
    for<'a> u16: Rem<&'a Self, Output = Self>,
    for<'a> u32: Rem<&'a Self, Output = Self>,
    for<'a> u64: Rem<&'a Self, Output = Self>,
    for<'a> u8: Rem<&'a Self, Output = Self>,
    u128: Rem<Self, Output = Self>,
    u16: Rem<Self, Output = Self>,
    u32: Rem<Self, Output = Self>,
    u64: Rem<Self, Output = Self>,
    u8: Rem<Self, Output = Self>,
    // RemAssign
    Self: RemAssign<Self>,
    Self: RemAssign<u128>,
    Self: RemAssign<u16>,
    Self: RemAssign<u32>,
    Self: RemAssign<u64>,
    Self: RemAssign<u8>,
    Self: RemAssign<usize>,
    for<'a> Self: RemAssign<&'a Self>,
    // Shl
    Self: Shl<i128, Output = Self>,
    Self: Shl<i16, Output = Self>,
    Self: Shl<i32, Output = Self>,
    Self: Shl<i64, Output = Self>,
    Self: Shl<i8, Output = Self>,
    Self: Shl<isize, Output = Self>,
    for<'a> &'a Self: Shl<&'a i128, Output = Self>,
    for<'a> &'a Self: Shl<&'a i16, Output = Self>,
    for<'a> &'a Self: Shl<&'a i32, Output = Self>,
    for<'a> &'a Self: Shl<&'a i64, Output = Self>,
    for<'a> &'a Self: Shl<&'a i8, Output = Self>,
    for<'a> &'a Self: Shl<&'a isize, Output = Self>,
    for<'a> &'a Self: Shl<&'a u128, Output = Self>,
    for<'a> &'a Self: Shl<&'a u16, Output = Self>,
    for<'a> &'a Self: Shl<&'a u32, Output = Self>,
    for<'a> &'a Self: Shl<&'a u64, Output = Self>,
    for<'a> &'a Self: Shl<&'a u8, Output = Self>,
    for<'a> &'a Self: Shl<&'a usize, Output = Self>,
    for<'a> &'a Self: Shl<i128, Output = Self>,
    for<'a> &'a Self: Shl<i16, Output = Self>,
    for<'a> &'a Self: Shl<i32, Output = Self>,
    for<'a> &'a Self: Shl<i64, Output = Self>,
    for<'a> &'a Self: Shl<i8, Output = Self>,
    for<'a> &'a Self: Shl<isize, Output = Self>,
    for<'a> &'a Self: Shl<u128, Output = Self>,
    for<'a> &'a Self: Shl<u16, Output = Self>,
    for<'a> &'a Self: Shl<u32, Output = Self>,
    for<'a> &'a Self: Shl<u64, Output = Self>,
    for<'a> &'a Self: Shl<u8, Output = Self>,
    for<'a> &'a Self: Shl<usize, Output = Self>,
    for<'a> Self: Shl<&'a i128, Output = Self>,
    for<'a> Self: Shl<&'a i16, Output = Self>,
    for<'a> Self: Shl<&'a i32, Output = Self>,
    for<'a> Self: Shl<&'a i64, Output = Self>,
    for<'a> Self: Shl<&'a i8, Output = Self>,
    for<'a> Self: Shl<&'a isize, Output = Self>,
    // ShlAssign
    Self: ShlAssign<i128>,
    Self: ShlAssign<i16>,
    Self: ShlAssign<i32>,
    Self: ShlAssign<i64>,
    Self: ShlAssign<i8>,
    Self: ShlAssign<isize>,
    Self: ShlAssign<u128>,
    Self: ShlAssign<u16>,
    Self: ShlAssign<u32>,
    Self: ShlAssign<u64>,
    Self: ShlAssign<u8>,
    Self: ShlAssign<usize>,
    for<'a> Self: ShlAssign<&'a i128>,
    for<'a> Self: ShlAssign<&'a i16>,
    for<'a> Self: ShlAssign<&'a i32>,
    for<'a> Self: ShlAssign<&'a i64>,
    for<'a> Self: ShlAssign<&'a i8>,
    for<'a> Self: ShlAssign<&'a isize>,
    for<'a> Self: ShlAssign<&'a u128>,
    for<'a> Self: ShlAssign<&'a u16>,
    for<'a> Self: ShlAssign<&'a u32>,
    for<'a> Self: ShlAssign<&'a u64>,
    for<'a> Self: ShlAssign<&'a u8>,
    for<'a> Self: ShlAssign<&'a usize>,
    // Shr
    Self: Shr<i128, Output = Self>,
    Self: Shr<i16, Output = Self>,
    Self: Shr<i32, Output = Self>,
    Self: Shr<i64, Output = Self>,
    Self: Shr<i8, Output = Self>,
    Self: Shr<isize, Output = Self>,
    for<'a> &'a Self: Shr<&'a i128, Output = Self>,
    for<'a> &'a Self: Shr<&'a i16, Output = Self>,
    for<'a> &'a Self: Shr<&'a i32, Output = Self>,
    for<'a> &'a Self: Shr<&'a i64, Output = Self>,
    for<'a> &'a Self: Shr<&'a i8, Output = Self>,
    for<'a> &'a Self: Shr<&'a isize, Output = Self>,
    for<'a> &'a Self: Shr<&'a u128, Output = Self>,
    for<'a> &'a Self: Shr<&'a u16, Output = Self>,
    for<'a> &'a Self: Shr<&'a u32, Output = Self>,
    for<'a> &'a Self: Shr<&'a u64, Output = Self>,
    for<'a> &'a Self: Shr<&'a u8, Output = Self>,
    for<'a> &'a Self: Shr<&'a usize, Output = Self>,
    for<'a> &'a Self: Shr<i128, Output = Self>,
    for<'a> &'a Self: Shr<i16, Output = Self>,
    for<'a> &'a Self: Shr<i32, Output = Self>,
    for<'a> &'a Self: Shr<i64, Output = Self>,
    for<'a> &'a Self: Shr<i8, Output = Self>,
    for<'a> &'a Self: Shr<isize, Output = Self>,
    for<'a> &'a Self: Shr<u128, Output = Self>,
    for<'a> &'a Self: Shr<u16, Output = Self>,
    for<'a> &'a Self: Shr<u32, Output = Self>,
    for<'a> &'a Self: Shr<u64, Output = Self>,
    for<'a> &'a Self: Shr<u8, Output = Self>,
    for<'a> &'a Self: Shr<usize, Output = Self>,
    for<'a> Self: Shr<&'a i128, Output = Self>,
    for<'a> Self: Shr<&'a i16, Output = Self>,
    for<'a> Self: Shr<&'a i32, Output = Self>,
    for<'a> Self: Shr<&'a i64, Output = Self>,
    for<'a> Self: Shr<&'a i8, Output = Self>,
    for<'a> Self: Shr<&'a isize, Output = Self>,
    // ShrAssign
    Self: ShrAssign<i128>,
    Self: ShrAssign<i16>,
    Self: ShrAssign<i32>,
    Self: ShrAssign<i64>,
    Self: ShrAssign<i8>,
    Self: ShrAssign<isize>,
    Self: ShrAssign<u128>,
    Self: ShrAssign<u16>,
    Self: ShrAssign<u32>,
    Self: ShrAssign<u64>,
    Self: ShrAssign<u8>,
    Self: ShrAssign<usize>,
    for<'a> Self: ShrAssign<&'a i128>,
    for<'a> Self: ShrAssign<&'a i16>,
    for<'a> Self: ShrAssign<&'a i32>,
    for<'a> Self: ShrAssign<&'a i64>,
    for<'a> Self: ShrAssign<&'a i8>,
    for<'a> Self: ShrAssign<&'a isize>,
    for<'a> Self: ShrAssign<&'a u128>,
    for<'a> Self: ShrAssign<&'a u16>,
    for<'a> Self: ShrAssign<&'a u32>,
    for<'a> Self: ShrAssign<&'a u64>,
    for<'a> Self: ShrAssign<&'a u8>,
    for<'a> Self: ShrAssign<&'a usize>,
    // Sub
    Self: Sub<Self, Output = Self>,
    Self: Sub<u128, Output = Self>,
    Self: Sub<u16, Output = Self>,
    Self: Sub<u32, Output = Self>,
    Self: Sub<u64, Output = Self>,
    Self: Sub<u8, Output = Self>,
    Self: Sub<usize, Output = Self>,
    for<'a> &'a Self: Sub<&'a Self, Output = Self>,
    for<'a> &'a Self: Sub<&'a u128, Output = Self>,
    for<'a> &'a Self: Sub<&'a u16, Output = Self>,
    for<'a> &'a Self: Sub<&'a u32, Output = Self>,
    for<'a> &'a Self: Sub<&'a u64, Output = Self>,
    for<'a> &'a Self: Sub<&'a u8, Output = Self>,
    for<'a> &'a Self: Sub<&'a usize, Output = Self>,
    for<'a> &'a Self: Sub<Self, Output = Self>,
    for<'a> &'a Self: Sub<u128, Output = Self>,
    for<'a> &'a Self: Sub<u16, Output = Self>,
    for<'a> &'a Self: Sub<u32, Output = Self>,
    for<'a> &'a Self: Sub<u64, Output = Self>,
    for<'a> &'a Self: Sub<u8, Output = Self>,
    for<'a> &'a Self: Sub<usize, Output = Self>,
    for<'a> &'a u128: Sub<&'a Self, Output = Self>,
    for<'a> &'a u128: Sub<Self, Output = Self>,
    for<'a> &'a u16: Sub<&'a Self, Output = Self>,
    for<'a> &'a u16: Sub<Self, Output = Self>,
    for<'a> &'a u32: Sub<&'a Self, Output = Self>,
    for<'a> &'a u32: Sub<Self, Output = Self>,
    for<'a> &'a u64: Sub<&'a Self, Output = Self>,
    for<'a> &'a u64: Sub<Self, Output = Self>,
    for<'a> &'a u8: Sub<&'a Self, Output = Self>,
    for<'a> &'a u8: Sub<Self, Output = Self>,
    for<'a> &'a usize: Sub<&'a Self, Output = Self>,
    for<'a> &'a usize: Sub<Self, Output = Self>,
    for<'a> Self: Sub<&'a Self, Output = Self>,
    for<'a> Self: Sub<&'a u128, Output = Self>,
    for<'a> Self: Sub<&'a u16, Output = Self>,
    for<'a> Self: Sub<&'a u32, Output = Self>,
    for<'a> Self: Sub<&'a u64, Output = Self>,
    for<'a> Self: Sub<&'a u8, Output = Self>,
    for<'a> Self: Sub<&'a usize, Output = Self>,
    for<'a> u128: Sub<&'a Self, Output = Self>,
    for<'a> u16: Sub<&'a Self, Output = Self>,
    for<'a> u32: Sub<&'a Self, Output = Self>,
    for<'a> u64: Sub<&'a Self, Output = Self>,
    for<'a> u8: Sub<&'a Self, Output = Self>,
    for<'a> usize: Sub<&'a Self, Output = Self>,
    u128: Sub<Self, Output = Self>,
    u16: Sub<Self, Output = Self>,
    u32: Sub<Self, Output = Self>,
    u64: Sub<Self, Output = Self>,
    u8: Sub<Self, Output = Self>,
    usize: Sub<Self, Output = Self>,
    // SubAssign
    Self: SubAssign<Self>,
    Self: SubAssign<u128>,
    Self: SubAssign<u16>,
    Self: SubAssign<u32>,
    Self: SubAssign<u64>,
    Self: SubAssign<u8>,
    Self: SubAssign<usize>,
    for<'a> Self: SubAssign<&'a Self>,
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
    fn to_bytes_be(&self) -> Vec<u8>;

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
    fn to_u32_digits(&self) -> Vec<u32>;

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
    fn to_u64_digits(&self) -> Vec<u64>;

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

    /// Returns the number of least-significant bits that are ones.
    fn trailing_ones(&self) -> u64;

    /// Returns the number of one bits.
    fn count_ones(&self) -> u64;
}

#[macro_export]
macro_rules! impl_big_natural {
    ($t:ty) => {
        impl BigNatural for $t {
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

            fn to_radix_be(&self, radix: u32) -> Vec<u8> {
                self.to_radix_be(radix)
            }

            fn to_radix_le(&self, radix: u32) -> Vec<u8> {
                self.to_radix_le(radix)
            }

            fn trailing_ones(&self) -> u64 {
                self.trailing_ones()
            }

            fn count_ones(&self) -> u64 {
                self.count_ones()
            }
        }
    };
}

impl_big_natural!(BigUint);
