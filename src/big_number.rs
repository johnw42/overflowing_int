#![allow(unused)]

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

// A trait covering the common functionality of BigInt and BigUint.
pub trait BigNumber: Sized {
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
    fn sqrt(&self) -> Self;

    /// Returns the truncated principal cube root of `self` --
    /// see [`num_integer::Roots::cbrt()`].
    fn cbrt(&self) -> Self;

    /// Returns the truncated principal `n`th root of `self` --
    /// See [`num_integer::Roots::nth_root()`].
    fn nth_root(&self, n: u32) -> Self;

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
