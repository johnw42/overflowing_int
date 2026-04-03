use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Not, Rem, RemAssign, Sub, SubAssign,
};

use crate::BigNumber;
use num_bigint::{BigInt, BigUint, ParseBigIntError, RandomBits, Sign, ToBigInt, ToBigUint};
use num_traits::{Pow, Signed};
use rand::prelude::Distribution;

pub trait BigInteger: BigNumber
where
    Self: Neg + Not + Signed,
    RandomBits: Distribution<Self>,
    // From
    Self: From<BigInt>
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
    for<'a> u128: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> u16: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> u32: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> u64: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> u8: TryFrom<Self> + TryFrom<&'a Self>,
    for<'a> usize: TryFrom<Self> + TryFrom<&'a Self>,
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

    /// Converts this [`BigInteger`] into a [`BigUint`], if it's not negative.
    fn to_biguint(&self) -> Option<BigUint>;
}

macro_rules! impl_big_integer {
    ($t:ty) => {
        impl BigInteger for $t {
            fn new(sign: Sign, digits: Vec<u32>) -> Self {
                Self::new(sign, digits)
            }

            fn from_biguint(sign: Sign, data: BigUint) -> Self {
                Self::from_biguint(sign, data)
            }

            fn from_slice(sign: Sign, slice: &[u32]) -> Self {
                Self::from_slice(sign, slice)
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

            fn to_signed_bytes_be(&self) -> Vec<u8> {
                self.to_signed_bytes_be()
            }

            fn to_signed_bytes_le(&self) -> Vec<u8> {
                self.to_signed_bytes_le()
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

            fn to_biguint(&self) -> Option<BigUint> {
                self.to_biguint()
            }
        }
    };
}

impl_big_integer!(BigInt);
