use crate::big_number::BigNumberDigits;
use crate::encoding::{Decode, Decoded, Encoding};
use crate::small_num::SmallNumber;
use crate::unsigned::Uint;
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Pow, Zero};
use std::borrow::Cow;
use std::cmp::Ordering;

use std::ops::Neg;

/// A signed overflowing integer type that can be used with any encoding that
/// implements `Encoding` with `Big = BigInt`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Int<E>(pub(crate) E);

impl<'enc, E> Int<E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    /// Converts self into an `Int` with different encoding.
    ///
    /// This cannot be implemented using the standard `From` trait because it would overlap
    /// with the blanket implementation of `T: From<T>`.
    pub fn reencode_into<'e2, E2>(self) -> Int<E2>
    where
        E2: Encoding<'e2, Big = BigInt>,
        E2::Small: SmallNumber<Wide = <E::Small as SmallNumber>::Wide>,
        'enc: 'e2,
    {
        Int::<E2>::reencode_from(self)
    }

    /// Converts an `Int` with one encoding into an `Int` with another encoding.
    ///
    /// This cannot be implemented using the standard `From` trait because it would overlap
    /// with the blanket implementation of `T: From<T>`.
    pub fn reencode_from<'e2, E2>(other: Int<E2>) -> Self
    where
        E2: Encoding<'e2, Big = BigInt>,
        E2::Small: SmallNumber<Wide = <E::Small as SmallNumber>::Wide>,
        'e2: 'enc,
    {
        Self(E::reencode_from(other.0))
    }

    /// Converts this big integer to a version with a static lifetime.  This may require cloning a `BigInt`.
    pub fn into_owned(self) -> Int<E::Owned> {
        Int(self.0.into_owned())
    }

    /// Creates a big integer that borrows from this one's data, if possible.
    /// If the encoding does not support borrowing, this will simply clone self.
    pub fn borrow<'a>(&'a self) -> Int<E::Borrowed<'a>> {
        Int(self.0.borrow())
    }

    /// A constant bigint with value 0, useful for static initialization.
    pub const ZERO: Self = Self(E::ZERO);

    /// Creates and initializes a bigint.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn new(sign: Sign, digits: Vec<u32>) -> Self {
        Self(if sign == Sign::NoSign {
            E::from_small(E::Small::zero())
        } else {
            E::from_big(E::Big::new(sign, digits))
        })
    }

    /// Creates and initializes a bigint.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn from_biguint(sign: Sign, data: Uint<E::Unsigned>) -> Self {
        BigInt::from_biguint(sign, BigUint::from(data)).into()
    }

    /// Creates and initializes a bigint.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn from_slice(sign: Sign, slice: &[u32]) -> Self {
        Self(E::from_big(E::Big::from_slice(sign, slice)))
    }

    /// Reinitializes a bigint.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn assign_from_slice(&mut self, sign: Sign, slice: &[u32])
    where
        E: Encoding<'enc>,
    {
        match self.0.decode_mut() {
            Decoded::Small(_) => self.0 = E::from_big(E::Big::from_slice(sign, slice)),
            Decoded::Big(b) => {
                b.assign_from_slice(sign, slice);
            }
        }
    }

    /// Creates and initializes a bigint.
    ///
    /// The bytes are in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::{EnumBigInt, Sign};
    ///
    /// assert_eq!(EnumBigInt::from_bytes_be(Sign::Plus, b"A"),
    ///            EnumBigInt::parse_bytes(b"65", 10).unwrap());
    /// assert_eq!(EnumBigInt::from_bytes_be(Sign::Plus, b"AA"),
    ///            EnumBigInt::parse_bytes(b"16705", 10).unwrap());
    /// assert_eq!(EnumBigInt::from_bytes_be(Sign::Plus, b"AB"),
    ///            EnumBigInt::parse_bytes(b"16706", 10).unwrap());
    /// assert_eq!(EnumBigInt::from_bytes_be(Sign::Plus, b"Hello world!"),
    ///            EnumBigInt::parse_bytes(b"22405534230753963835153736737", 10).unwrap());
    /// ```
    #[inline]
    pub fn from_bytes_be(sign: Sign, bytes: &[u8]) -> Self
    where
        E::Small: Neg<Output = E::Small>,
    {
        if let Some(from_bytes) = SmallNumber::from_bytes_be(bytes) {
            let unsigned = Self(E::from_small(from_bytes));
            match sign {
                Sign::Plus => unsigned,
                Sign::Minus => -unsigned,
                Sign::NoSign => Self::ZERO,
            }
        } else {
            Self(E::from_big(E::Big::from_bytes_be(sign, bytes)))
        }
    }

    /// Creates and initializes a bigint.
    ///
    /// The bytes are in little-endian byte order.
    #[inline]
    pub fn from_bytes_le(sign: Sign, bytes: &[u8]) -> Self
    where
        E::Small: Neg<Output = E::Small>,
    {
        if let Some(from_bytes) = SmallNumber::from_bytes_le(bytes) {
            let unsigned = Self(E::from_small(from_bytes));
            match sign {
                Sign::Plus => unsigned,
                Sign::Minus => -unsigned,
                Sign::NoSign => Self::ZERO,
            }
        } else {
            Self(E::from_big(E::Big::from_bytes_le(sign, bytes)))
        }
    }

    /// Creates and initializes a bigint from an array of bytes in
    /// two's complement binary representation.
    ///
    /// The digits are in big-endian base 2<sup>8</sup>.
    #[inline]
    pub fn from_signed_bytes_be(digits: &[u8]) -> Self {
        Self(E::from_big(E::Big::from_signed_bytes_be(digits)))
    }

    /// Creates and initializes a bigint from an array of bytes in two's complement.
    ///
    /// The digits are in little-endian base 2<sup>8</sup>.
    #[inline]
    pub fn from_signed_bytes_le(digits: &[u8]) -> Self {
        Self(E::from_big(E::Big::from_signed_bytes_le(digits)))
    }

    /// Creates and initializes a bigint.
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::{EnumBigInt};
    ///
    /// assert_eq!(EnumBigInt::parse_bytes(b"1234", 10), Some(EnumBigInt::from(1234)));
    /// assert_eq!(EnumBigInt::parse_bytes(b"ABCD", 16), Some(EnumBigInt::from(0xABCD)));
    /// assert_eq!(EnumBigInt::parse_bytes(b"G", 16), None);
    /// ```
    #[inline]
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self> {
        E::parse_bytes(buf, radix).map(Self)
    }

    /// Creates and initializes a bigint. Each u8 of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in big-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::{EnumBigInt, Sign};
    ///
    /// let inbase190 = vec![15, 33, 125, 12, 14];
    /// let a = EnumBigInt::from_radix_be(Sign::Minus, &inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_be(190), (Sign::Minus, inbase190));
    /// ```
    #[inline]
    pub fn from_radix_be(sign: Sign, buf: &[u8], radix: u32) -> Option<Self> {
        E::Big::from_radix_be(sign, buf, radix)
            .map(E::from_big)
            .map(Self)
    }

    /// Creates and initializes a bigint. Each u8 of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in little-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::{EnumBigInt, Sign};
    ///
    /// let inbase190 = vec![14, 12, 125, 33, 15];
    /// let a = EnumBigInt::from_radix_le(Sign::Minus, &inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_le(190), (Sign::Minus, inbase190));
    /// ```
    #[inline]
    pub fn from_radix_le(sign: Sign, buf: &[u8], radix: u32) -> Option<Self> {
        E::Big::from_radix_le(sign, buf, radix)
            .map(E::from_big)
            .map(Self)
    }

    /// Returns the sign and the byte representation of the bigint in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::{EnumBigInt, Sign};
    ///
    /// let i = EnumBigInt::from(-1125);
    /// assert_eq!(i.to_bytes_be(), (Sign::Minus, vec![4, 101]));
    /// ```
    #[inline]
    pub fn to_bytes_be(&self) -> (Sign, Vec<u8>) {
        self.big_cow().to_bytes_be()
    }

    /// Returns the sign and the byte representation of the bigint in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::{EnumBigInt, Sign};
    ///
    /// let i = EnumBigInt::from(-1125);
    /// assert_eq!(i.to_bytes_le(), (Sign::Minus, vec![101, 4]));
    /// ```
    #[inline]
    pub fn to_bytes_le(&self) -> (Sign, Vec<u8>) {
        self.big_cow().to_bytes_le()
    }

    /// Returns the sign and the `u32` digits representation of the bigint ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::overflowing_int::{EnumBigInt, Sign};
    ///
    /// assert_eq!(EnumBigInt::from(-1125).to_u32_digits(), (Sign::Minus, vec![1125]));
    /// assert_eq!(EnumBigInt::from(4294967295u32).to_u32_digits(), (Sign::Plus, vec![4294967295]));
    /// assert_eq!(EnumBigInt::from(4294967296u64).to_u32_digits(), (Sign::Plus, vec![0, 1]));
    /// assert_eq!(EnumBigInt::from(-112500000000i64).to_u32_digits(), (Sign::Minus, vec![830850304, 26]));
    /// assert_eq!(EnumBigInt::from(112500000000i64).to_u32_digits(), (Sign::Plus, vec![830850304, 26]));
    /// ```
    #[inline]
    pub fn to_u32_digits(&self) -> (Sign, Vec<u32>) {
        self.big_cow().to_u32_digits()
    }

    /// Returns the sign and the `u64` digits representation of the [`BigInt`] ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::overflowing_int::{EnumBigInt, Sign};
    ///
    /// assert_eq!(EnumBigInt::from(-1125).to_u64_digits(), (Sign::Minus, vec![1125]));
    /// assert_eq!(EnumBigInt::from(4294967295u32).to_u64_digits(), (Sign::Plus, vec![4294967295]));
    /// assert_eq!(EnumBigInt::from(4294967296u64).to_u64_digits(), (Sign::Plus, vec![4294967296]));
    /// assert_eq!(EnumBigInt::from(-112500000000i64).to_u64_digits(), (Sign::Minus, vec![112500000000]));
    /// assert_eq!(EnumBigInt::from(112500000000i64).to_u64_digits(), (Sign::Plus, vec![112500000000]));
    /// assert_eq!(EnumBigInt::from(1u128 << 64).to_u64_digits(), (Sign::Plus, vec![0, 1]));
    /// ```
    #[inline]
    pub fn to_u64_digits(&self) -> (Sign, Vec<u64>) {
        self.big_cow().to_u64_digits()
    }

    /// Returns an iterator of `u32` digits representation of the bigint ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::EnumBigInt;
    ///
    /// assert_eq!(EnumBigInt::from(-1125).iter_u32_digits().collect::<Vec<u32>>(), vec![1125]);
    /// assert_eq!(EnumBigInt::from(4294967295u32).iter_u32_digits().collect::<Vec<u32>>(), vec![4294967295]);
    /// assert_eq!(EnumBigInt::from(4294967296u64).iter_u32_digits().collect::<Vec<u32>>(), vec![0, 1]);
    /// assert_eq!(EnumBigInt::from(-112500000000i64).iter_u32_digits().collect::<Vec<u32>>(), vec![830850304, 26]);
    /// assert_eq!(EnumBigInt::from(112500000000i64).iter_u32_digits().collect::<Vec<u32>>(), vec![830850304, 26]);
    /// ```
    #[inline]
    pub fn iter_u32_digits(&self) -> impl BigNumberDigits<'_, u32> {
        self.0.iter_u32_digits()
    }

    /// Returns an iterator of `u64` digits representation of the bigint ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::EnumBigInt;
    ///
    /// assert_eq!(EnumBigInt::from(-1125).iter_u64_digits().collect::<Vec<u64>>(), vec![1125u64]);
    /// assert_eq!(EnumBigInt::from(4294967295u32).iter_u64_digits().collect::<Vec<u64>>(), vec![4294967295u64]);
    /// assert_eq!(EnumBigInt::from(4294967296u64).iter_u64_digits().collect::<Vec<u64>>(), vec![4294967296u64]);
    /// assert_eq!(EnumBigInt::from(-112500000000i64).iter_u64_digits().collect::<Vec<u64>>(), vec![112500000000u64]);
    /// assert_eq!(EnumBigInt::from(112500000000i64).iter_u64_digits().collect::<Vec<u64>>(), vec![112500000000u64]);
    /// assert_eq!(EnumBigInt::from(1u128 << 64).iter_u64_digits().collect::<Vec<u64>>(), vec![0, 1]);
    /// ```
    #[inline]
    pub fn iter_u64_digits(&self) -> impl BigNumberDigits<'_, u64> {
        self.0.iter_u64_digits()
    }

    /// Returns the two's-complement byte representation of the bigint in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::{EnumBigInt};
    ///
    /// let i = EnumBigInt::from(-1125);
    /// assert_eq!(i.to_signed_bytes_be(), vec![251, 155]);
    /// ```
    #[inline]
    pub fn to_signed_bytes_be(&self) -> Vec<u8> {
        self.big_cow().to_signed_bytes_be()
    }

    /// Returns the two's-complement byte representation of the bigint in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::{EnumBigInt};
    ///
    /// let i = EnumBigInt::from(-1125);
    /// assert_eq!(i.to_signed_bytes_le(), vec![155, 251]);
    /// ```
    #[inline]
    pub fn to_signed_bytes_le(&self) -> Vec<u8> {
        self.big_cow().to_signed_bytes_le()
    }

    /// Returns the integer formatted as a string in the given radix.
    /// `radix` must be in the range `2...36`.
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::{EnumBigInt};
    ///
    /// let i = EnumBigInt::parse_bytes(b"ff", 16).unwrap();
    /// assert_eq!(i.to_str_radix(16), "ff");
    /// ```
    #[inline]
    pub fn to_str_radix(&self, radix: u32) -> String {
        self.0.to_str_radix(radix)
    }

    /// Returns the integer in the requested base in big-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based u8 number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::{EnumBigInt, Sign};
    ///
    /// assert_eq!(EnumBigInt::from(-0xFFFFi64).to_radix_be(159),
    ///            (Sign::Minus, vec![2, 94, 27]));
    /// // 0xFFFF = 65535 = 2*(159^2) + 94*159 + 27
    /// ```
    #[inline]
    pub fn to_radix_be(&self, radix: u32) -> (Sign, Vec<u8>) {
        self.big_cow().to_radix_be(radix)
    }

    /// Returns the integer in the requested base in little-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based u8 number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::{EnumBigInt, Sign};
    ///
    /// assert_eq!(EnumBigInt::from(-0xFFFFi64).to_radix_le(159),
    ///            (Sign::Minus, vec![27, 94, 2]));
    /// // 0xFFFF = 65535 = 27 + 94*159 + 2*(159^2)
    /// ```
    #[inline]
    pub fn to_radix_le(&self, radix: u32) -> (Sign, Vec<u8>) {
        self.big_cow().to_radix_le(radix)
    }

    /// Returns the magnitude of the bigint as an unsigned bigint.
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::{EnumBigInt, EnumBigUint};
    /// use num_traits::Zero;
    ///
    /// assert_eq!(EnumBigInt::from(1234).magnitude(), EnumBigUint::from(1234u32));
    /// assert_eq!(EnumBigInt::from(-4321).magnitude(), EnumBigUint::from(4321u32));
    /// assert!(EnumBigInt::ZERO.clone().magnitude().is_zero());
    /// ```
    #[inline]
    pub fn magnitude(self) -> Uint<E::Unsigned> {
        Uint::from(self.into_parts().1)
    }

    // Returns the sign of the [`Int`] as a Sign.
    #[inline]
    pub fn sign(&self) -> Sign {
        match self.decode() {
            Decoded::Small(n) => match n.cmp(&E::Small::zero()) {
                Ordering::Equal => Sign::NoSign,
                Ordering::Greater => Sign::Plus,
                Ordering::Less => Sign::Minus,
            },
            Decoded::Big(n) => n.sign(),
        }
    }

    /// Convert this bigint into its [`Sign`] and unsigned bigint magnitude,
    /// the reverse of [`Self::from_biguint()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use overflowing_int::{EnumBigInt, Sign};
    /// use num_bigint::BigUint;
    ///
    /// assert_eq!(EnumBigInt::from(1234).into_parts(), (Sign::Plus, BigUint::from(1234u32)));
    /// assert_eq!(EnumBigInt::from(-4321).into_parts(), (Sign::Minus, BigUint::from(4321u32)));
    /// assert_eq!(EnumBigInt::ZERO.into_parts(), (Sign::NoSign, BigUint::ZERO));
    /// ```
    #[inline]
    pub fn into_parts(self) -> (Sign, BigUint) {
        self.into_big().into_parts()
    }

    /// Determines the fewest bits necessary to express the bigint,
    /// not including the sign.
    #[inline]
    pub fn bits(&self) -> u64 {
        self.0.bits()
    }

    /// Converts this bigint into a an unsigned bigint, if it's not negative.
    #[inline]
    pub fn to_biguint(&self) -> Option<Uint<E::Unsigned>> {
        match self.sign() {
            Sign::Minus => None,
            _ => Some(self.clone().magnitude()),
        }
    }

    #[inline]
    pub fn checked_add(&self, v: &Self) -> Option<Self> {
        CheckedAdd::checked_add(self, v)
    }

    #[inline]
    pub fn checked_sub(&self, v: &Self) -> Option<Self> {
        CheckedSub::checked_sub(self, v)
    }

    #[inline]
    pub fn checked_mul(&self, v: &Self) -> Option<Self> {
        CheckedMul::checked_mul(self, v)
    }

    #[inline]
    pub fn checked_div(&self, v: &Self) -> Option<Self> {
        CheckedDiv::checked_div(self, v)
    }

    /// Returns `self ^ exponent`.
    #[inline]
    pub fn pow(&self, exponent: u32) -> Self {
        Pow::pow(self, exponent)
    }

    /// Returns `(self ^ exponent) mod modulus`
    ///
    /// Note that this rounds like `mod_floor`, not like the `%` operator,
    /// which makes a difference when given a negative `self` or `modulus`.
    /// The result will be in the interval `[0, modulus)` for `modulus > 0`,
    /// or in the interval `(modulus, 0]` for `modulus < 0`
    ///
    /// Panics if the exponent is negative or the modulus is zero.
    #[inline]
    pub fn modpow(&self, exponent: &Self, modulus: &Self) -> Self {
        Self(self.0.modpow(&exponent.0, &modulus.0))
    }

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
    /// use overflowing_int::EnumBigInt;
    /// use num_integer::Integer;
    /// use num_traits::{One, Zero};
    ///
    /// let m = EnumBigInt::from(383);
    ///
    /// // Trivial cases
    /// assert_eq!(EnumBigInt::zero().modinv(&m), None);
    /// assert_eq!(EnumBigInt::one().modinv(&m), Some(EnumBigInt::one()));
    /// let neg1 = &m - 1u32;
    /// assert_eq!(neg1.modinv(&m), Some(neg1));
    ///
    /// // Positive self and modulus
    /// let a = EnumBigInt::from(271);
    /// let x = a.modinv(&m).unwrap();
    /// assert_eq!(x, EnumBigInt::from(106));
    /// assert_eq!(x.modinv(&m).unwrap(), a);
    /// assert_eq!((&a * x).mod_floor(&m), EnumBigInt::one());
    ///
    /// // Negative self and positive modulus
    /// let b = -&a;
    /// let x = b.modinv(&m).unwrap();
    /// assert_eq!(x, EnumBigInt::from(277));
    /// assert_eq!((&b * x).mod_floor(&m), EnumBigInt::one());
    ///
    /// // Positive self and negative modulus
    /// let n = -&m;
    /// let x = a.modinv(&n).unwrap();
    /// assert_eq!(x, EnumBigInt::from(-277));
    /// assert_eq!((&a * x).mod_floor(&n), &n + 1);
    ///
    /// // Negative self and modulus
    /// let x = b.modinv(&n).unwrap();
    /// assert_eq!(x, EnumBigInt::from(-106));
    /// assert_eq!((&b * x).mod_floor(&n), &n + 1);
    /// ```
    #[inline]
    pub fn modinv(&self, modulus: &Self) -> Option<Self> {
        self.0.modinv(&modulus.0).map(Self)
    }

    /// Returns the truncated principal square root of `self` --
    /// see [`num_integer::Roots::sqrt()`].
    #[inline]
    pub fn sqrt(&self) -> Self {
        Self(self.0.sqrt())
    }

    /// Returns the truncated principal cube root of `self` --
    /// see [`num_integer::Roots::cbrt()`].
    #[inline]
    pub fn cbrt(&self) -> Self {
        Self(self.0.cbrt())
    }

    /// Returns the truncated principal `n`th root of `self` --
    /// See [`num_integer::Roots::nth_root()`].
    #[inline]
    pub fn nth_root(&self, n: u32) -> Self {
        Self(self.0.nth_root(n))
    }

    /// Returns the number of least-significant bits that are zero,
    /// or `None` if the entire number is zero.
    #[inline]
    pub fn trailing_zeros(&self) -> Option<u64> {
        self.0.trailing_zeros()
    }

    /// Returns whether the bit in position `bit` is set,
    /// using the two's complement for negative numbers
    #[inline]
    pub fn bit(&self, bit: u64) -> bool {
        self.0.bit(bit)
    }

    /// Sets or clears the bit in the given position,
    /// using the two's complement for negative numbers
    ///
    /// Note that setting/clearing a bit (for positive/negative numbers,
    /// respectively) greater than the current bit length, a reallocation
    /// may be needed to store the new digits
    #[inline]
    pub fn set_bit(&mut self, bit: u64, value: bool)
    where
        E: Encoding<'enc>,
    {
        self.0.set_bit(bit, value);
    }
}

impl<'enc, E> Default for Int<E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    fn default() -> Self {
        Self::zero()
    }
}

impl<'enc, E> Decode<'enc, E::Small> for Int<E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    fn into_decoded(self) -> Decoded<E::Small, Cow<'enc, E::Big>> {
        self.0.into_decoded()
    }

    fn decode<'a>(&'a self) -> Decoded<E::Small, Cow<'a, <E::Small as SmallNumber>::Big>> {
        self.0.decode()
    }
}

impl<'enc, E> Decode<'enc, E::Small> for &Int<E>
where
    E: Encoding<'enc, Big = BigInt>,
{
    fn into_decoded(self) -> Decoded<E::Small, Cow<'enc, E::Big>> {
        self.0.clone().into_decoded()
    }

    fn decode<'a>(&'a self) -> Decoded<E::Small, Cow<'a, <E::Small as SmallNumber>::Big>> {
        self.0.decode()
    }
}
