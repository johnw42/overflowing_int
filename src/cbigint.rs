use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::mem::size_of;

use num_bigint::{BigInt, BigUint, Sign};
#[allow(unused_imports)]
use num_traits::{ToPrimitive, Zero};

use crate::accum::*;
use crate::decoded::Decoded;
use crate::encoding::Encoded;
use crate::Sign::*;
use crate::{Digit, Udigit, DIGIT_BITS};

#[derive(Clone)]
pub struct CBigInt(pub(crate) Encoded);

impl PartialEq for CBigInt {
    fn eq(&self, other: &Self) -> bool {
        self.decode_ref().eq(&other.decode_ref())
    }
}

impl Eq for CBigInt {}

impl From<CBigInt> for Decoded<BigInt> {
    fn from(arg: CBigInt) -> Self {
        arg.0.decode()
    }
}

impl From<Decoded<BigInt>> for CBigInt {
    fn from(value: Decoded<BigInt>) -> Self {
        Self(value.encode())
    }
}

impl From<Encoded> for CBigInt {
    fn from(value: Encoded) -> Self {
        Self(value)
    }
}

impl CBigInt {
    pub(crate) fn decode(self) -> Decoded<BigInt> {
        self.0.decode()
    }

    pub(crate) fn decode_ref(&self) -> Decoded<&BigInt> {
        self.0.decode_ref()
    }

    pub(crate) fn decode_mut(&mut self) -> Decoded<&mut BigInt> {
        self.0.decode_mut()
    }

    pub(crate) fn to_digit(&self) -> Option<Digit> {
        match self.decode_ref() {
            Decoded::Digit(n) => Some(n),
            _ => None,
        }
    }

    pub(crate) fn to_digit_with(&self, other: &CBigInt) -> Option<(Digit, Digit)> {
        match (self.to_digit(), other.to_digit()) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }

    fn from_accum(sign: Sign, accum: Udigit) -> Option<Self> {
        accum_to_digit(sign, accum).map(|digit| Decoded::Digit(digit).into())
    }

    /// Creates and initializes a BigInt.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[cfg(not(feature = "tiny_digit"))]
    pub fn new(sign: Sign, digits: Vec<u32>) -> CBigInt {
        if sign == NoSign {
            return CBigInt(Encoded::zero());
        }
        if digits.len() <= 4 {
            let mut value: Digit = 0;
            for &digit in &digits {
                value = (value << 32) | digit as Digit;
            }
            if value >= 0 {
                if sign == Minus {
                    value = -value;
                }
                return value.into();
            }
        }
        CBigInt(Decoded::Big(BigInt::new(sign, digits)).encode())
    }

    /// Creates and initializes a `CBigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn from_biguint(sign: Sign, data: BigUint) -> CBigInt {
        BigInt::from_biguint(sign, data).into()
    }

    /// Creates and initializes a `CBigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[cfg(not(feature = "tiny_digit"))]
    pub fn from_slice(sign: Sign, slice: &[u32]) -> CBigInt {
        if slice.len() <= size_of::<Udigit>() / size_of::<u32>() {
            let mut accum = 0;
            for (i, &word) in slice.iter().enumerate() {
                accum |= (word as Udigit) << (i * 8);
            }
            if let Some(result) = Self::from_accum(sign, accum) {
                return result;
            }
        }
        Self::new(sign, Vec::from(slice))
    }

    /// Reinitializes a `CBigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    #[cfg(not(feature = "tiny_digit"))]
    pub fn assign_from_slice(&mut self, sign: Sign, slice: &[u32]) {
        *self = Self::from_slice(sign, slice);
    }

    /// Creates and initializes a `CBigInt`.
    ///
    /// The bytes are in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CBigInt, Sign};
    ///
    /// assert_eq!(CBigInt::from_bytes_be(Sign::Plus, b"A"),
    ///            CBigInt::parse_bytes(b"65", 10).unwrap());
    /// assert_eq!(CBigInt::from_bytes_be(Sign::Plus, b"AA"),
    ///            CBigInt::parse_bytes(b"16705", 10).unwrap());
    /// assert_eq!(CBigInt::from_bytes_be(Sign::Plus, b"AB"),
    ///            CBigInt::parse_bytes(b"16706", 10).unwrap());
    /// assert_eq!(CBigInt::from_bytes_be(Sign::Plus, b"Hello world!"),
    ///            CBigInt::parse_bytes(b"22405534230753963835153736737", 10).unwrap());
    /// ```
    pub fn from_bytes_be(sign: Sign, bytes: &[u8]) -> CBigInt {
        if let Some(accum) = accum_be(bytes) {
            if let Some(result) = Self::from_accum(sign, accum) {
                return result;
            }
        }
        BigInt::from_bytes_be(sign, bytes).into()
    }

    /// Creates and initializes a `CBigInt`.
    ///
    /// The bytes are in little-endian byte order.
    pub fn from_bytes_le(sign: Sign, bytes: &[u8]) -> CBigInt {
        if let Some(accum) = accum_le(bytes) {
            if let Some(result) = Self::from_accum(sign, accum) {
                return result;
            }
        }
        BigInt::from_bytes_le(sign, bytes).into()
    }

    /// Creates and initializes a `CBigInt` from an array of bytes in
    /// two's complement binary representation.
    ///
    /// The digits are in big-endian base 2<sup>8</sup>.
    pub fn from_signed_bytes_be(digits: &[u8]) -> CBigInt {
        if let Some(accum) = accum_be(digits) {
            (accum as Digit).into()
        } else {
            BigInt::from_signed_bytes_be(digits).into()
        }
    }

    /// Creates and initializes a `CBigInt` from an array of bytes in two's complement.
    ///
    /// The digits are in little-endian base 2<sup>8</sup>.
    pub fn from_signed_bytes_le(digits: &[u8]) -> CBigInt {
        if let Some(accum) = accum_le(digits) {
            (accum as Digit).into()
        } else {
            BigInt::from_signed_bytes_le(digits).into()
        }
    }

    /// Creates and initializes a `CBigInt`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CBigInt, ToCBigInt};
    ///
    /// assert_eq!(CBigInt::parse_bytes(b"1234", 10), ToCBigInt::to_cbigint(&1234));
    /// assert_eq!(CBigInt::parse_bytes(b"ABCD", 16), ToCBigInt::to_cbigint(&0xABCD));
    /// assert_eq!(CBigInt::parse_bytes(b"G", 16), None);
    /// ```
    #[inline]
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Option<CBigInt> {
        BigInt::parse_bytes(buf, radix).map(Self::from)
    }

    /// Creates and initializes a `CBigInt`. Each u8 of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in big-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CBigInt, Sign};
    ///
    /// let inbase190 = vec![15, 33, 125, 12, 14];
    /// let a = CBigInt::from_radix_be(Sign::Minus, &inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_be(190), (Sign::Minus, inbase190));
    /// ```
    pub fn from_radix_be(sign: Sign, buf: &[u8], radix: u32) -> Option<CBigInt> {
        BigInt::from_radix_be(sign, buf, radix).map(Self::from)
    }

    /// Creates and initializes a `CBigInt`. Each u8 of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in little-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CBigInt, Sign};
    ///
    /// let inbase190 = vec![14, 12, 125, 33, 15];
    /// let a = CBigInt::from_radix_be(Sign::Minus, &inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_be(190), (Sign::Minus, inbase190));
    /// ```
    pub fn from_radix_le(sign: Sign, buf: &[u8], radix: u32) -> Option<CBigInt> {
        BigInt::from_radix_le(sign, buf, radix).map(Self::from)
    }

    /// Returns the sign and the byte representation of the `CBigInt` in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{ToCBigInt, Sign};
    ///
    /// let i = -1125.to_cbigint().unwrap();
    /// assert_eq!(i.to_bytes_be(), (Sign::Minus, vec![4, 101]));
    /// ```
    pub fn to_bytes_be(&self) -> (Sign, Vec<u8>) {
        match self.decode_ref() {
            Decoded::Digit(n) => match make_accum(n) {
                (NoSign, _) => (NoSign, Vec::new()),
                (sign, accum) => {
                    let bytes = accum.to_be_bytes();
                    let mut i = 0;
                    while i < bytes.len() && bytes[i] == 0 {
                        i += 1
                    }
                    (sign, bytes[i..].to_vec())
                }
            },
            Decoded::Big(n) => n.to_bytes_be(),
        }
    }

    /// Returns the sign and the byte representation of the `CBigInt` in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{ToCBigInt, Sign};
    ///
    /// let i = -1125.to_cbigint().unwrap();
    /// assert_eq!(i.to_bytes_le(), (Sign::Minus, vec![101, 4]));
    /// ```
    pub fn to_bytes_le(&self) -> (Sign, Vec<u8>) {
        match self.decode_ref() {
            Decoded::Digit(n) => {
                let (sign, accum) = make_accum(n);
                if sign == NoSign {
                    (sign, Vec::new())
                } else {
                    let mut bytes = accum.to_le_bytes().to_vec();
                    while let Some(0) = bytes.last() {
                        bytes.pop();
                    }
                    (sign, bytes)
                }
            }
            Decoded::Big(n) => n.to_bytes_le(),
        }
    }

    /// Returns the sign and the `u32` digits representation of the `CBigInt` ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CBigInt, Sign};
    ///
    /// assert_eq!(CBigInt::from(-1125).to_u32_digits(), (Sign::Minus, vec![1125]));
    /// assert_eq!(CBigInt::from(4294967295u32).to_u32_digits(), (Sign::Plus, vec![4294967295]));
    /// assert_eq!(CBigInt::from(4294967296u64).to_u32_digits(), (Sign::Plus, vec![0, 1]));
    /// assert_eq!(CBigInt::from(-112500000000i64).to_u32_digits(), (Sign::Minus, vec![830850304, 26]));
    /// assert_eq!(CBigInt::from(112500000000i64).to_u32_digits(), (Sign::Plus, vec![830850304, 26]));
    /// ```
    #[cfg(not(feature = "tiny_digit"))]
    pub fn to_u32_digits(&self) -> (Sign, Vec<u32>) {
        match self.decode_ref() {
            Decoded::Digit(n) => match make_accum(n) {
                (NoSign, _) => (NoSign, Vec::new()),
                (sign, mut accum) => {
                    let mut digits = Vec::with_capacity(4);
                    while accum != 0 {
                        digits.push(accum as u32);
                        accum >>= 32;
                    }
                    (sign, digits)
                }
            },
            Decoded::Big(n) => n.to_u32_digits(),
        }
    }

    /// Returns the two's-complement byte representation of the `CBigInt` in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::ToCBigInt;
    ///
    /// let i = -1125.to_cbigint().unwrap();
    /// assert_eq!(i.to_signed_bytes_be(), vec![251, 155]);
    /// ```
    pub fn to_signed_bytes_be(&self) -> Vec<u8> {
        match self.decode_ref() {
            Decoded::Digit(0) => Vec::new(),
            Decoded::Digit(n) => {
                let bytes = n.to_be_bytes();
                let to_discard = if n >= 0 { 0 } else { 0xff };
                let mut i = 0;
                while i < bytes.len() && bytes[i] == to_discard {
                    i += 1
                }
                bytes[i..].to_vec()
            }
            Decoded::Big(n) => n.to_signed_bytes_be(),
        }
    }

    /// Returns the two's-complement byte representation of the `CBigInt` in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::ToCBigInt;
    ///
    /// let i = -1125.to_cbigint().unwrap();
    /// assert_eq!(i.to_signed_bytes_le(), vec![155, 251]);
    /// ```
    pub fn to_signed_bytes_le(&self) -> Vec<u8> {
        match self.decode_ref() {
            Decoded::Digit(0) => Vec::new(),
            Decoded::Digit(n) => {
                let bytes = n.to_le_bytes();
                let to_discard = if n >= 0 { 0 } else { 0xff };
                let mut i = size_of::<Digit>();
                while i > 0 && bytes[i - 1] == to_discard {
                    i -= 1
                }
                bytes[..i].to_vec()
            }
            Decoded::Big(n) => n.to_signed_bytes_le(),
        }
    }

    /// Returns the integer formatted as a string in the given radix.
    /// `radix` must be in the range `2...36`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::CBigInt;
    ///
    /// let i = CBigInt::parse_bytes(b"ff", 16).unwrap();
    /// assert_eq!(i.to_str_radix(16), "ff");
    /// ```
    #[inline]
    pub fn to_str_radix(&self, radix: u32) -> String {
        self.to_bigint().to_str_radix(radix)
    }

    /// Returns the integer in the requested base in big-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based u8 number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CBigInt, Sign};
    ///
    /// assert_eq!(CBigInt::from(-0xFFFFi64).to_radix_be(159),
    ///            (Sign::Minus, vec![2, 94, 27]));
    /// // 0xFFFF = 65535 = 2*(159^2) + 94*159 + 27
    /// ```
    #[inline]
    pub fn to_radix_be(&self, radix: u32) -> (Sign, Vec<u8>) {
        self.to_bigint().to_radix_be(radix)
    }

    /// Returns the integer in the requested base in little-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based u8 number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CBigInt, Sign};
    ///
    /// assert_eq!(CBigInt::from(-0xFFFFi64).to_radix_le(159),
    ///            (Sign::Minus, vec![27, 94, 2]));
    /// // 0xFFFF = 65535 = 27 + 94*159 + 2*(159^2)
    /// ```
    #[inline]
    pub fn to_radix_le(&self, radix: u32) -> (Sign, Vec<u8>) {
        self.to_bigint().to_radix_le(radix)
    }

    /// Returns the sign of the `CBigInt` as a `Sign`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CBigInt, Sign};
    /// use num_traits::Zero;
    ///
    /// assert_eq!(CBigInt::from(1234).sign(), Sign::Plus);
    /// assert_eq!(CBigInt::from(-4321).sign(), Sign::Minus);
    /// assert_eq!(CBigInt::zero().sign(), Sign::NoSign);
    /// ```
    #[inline]
    pub fn sign(&self) -> Sign {
        match self.decode_ref() {
            Decoded::Digit(n) => {
                if n > 0 {
                    Plus
                } else if n < 0 {
                    Minus
                } else {
                    NoSign
                }
            }
            Decoded::Big(n) => n.sign(),
        }
    }

    /// Returns the magnitude of the `CBigInt` as a `BigUint`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::CBigInt;
    /// use num_traits::Zero;
    /// use std::borrow::Borrow;
    /// use num_bigint::BigUint;
    ///
    /// assert_eq!(*CBigInt::from(1234).magnitude(), BigUint::from(1234u32));
    /// assert_eq!(*CBigInt::from(-4321).magnitude(), BigUint::from(4321u32));
    /// assert!(CBigInt::zero().magnitude().is_zero());
    /// ```
    #[inline]
    pub fn magnitude(&self) -> Cow<'_, BigUint> {
        match self.decode_ref() {
            Decoded::Digit(n) => Cow::Owned(BigInt::from(n).into_parts().1),
            Decoded::Big(n) => Cow::Borrowed(n.magnitude()),
        }
    }

    /// Returns the magnitude of the `CBigInt` as a `BigUint` if the necessary
    /// `BigUint` already exists.
    #[inline]
    pub fn try_magnitude(&self) -> Option<&BigUint> {
        match self.decode_ref() {
            Decoded::Digit(_) => None,
            Decoded::Big(n) => Some(n.magnitude()),
        }
    }

    /// Convert this `CBigInt` into its `Sign` and `BigUint` magnitude,
    /// the reverse of `CBigInt::from_biguint`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CBigInt, Sign};
    /// use num_traits::Zero;
    /// use num_bigint::BigUint;
    ///
    /// assert_eq!(CBigInt::from(1234).into_parts(), (Sign::Plus, BigUint::from(1234u32)));
    /// assert_eq!(CBigInt::from(-4321).into_parts(), (Sign::Minus, BigUint::from(4321u32)));
    /// assert_eq!(CBigInt::zero().into_parts(), (Sign::NoSign, BigUint::zero()));
    /// ```
    #[inline]
    pub fn into_parts(self) -> (Sign, BigUint) {
        BigInt::from(self).into_parts()
    }

    /// Determines the fewest bits necessary to express the `BigInt`,
    /// not including the sign.
    #[inline]
    pub fn bits(&self) -> u64 {
        match self.decode_ref() {
            Decoded::Digit(n) => {
                if n >= 0 {
                    DIGIT_BITS as u32 - n.leading_zeros()
                } else if n == Digit::MIN {
                    DIGIT_BITS as u32
                } else {
                    DIGIT_BITS as u32 - (-n).leading_zeros()
                }
            }
            .into(),
            Decoded::Big(n) => n.bits(),
        }
    }

    /// Converts this `CBigInt` into a `BigInt`.
    pub fn to_bigint(&self) -> Cow<'_, BigInt> {
        match self.decode_ref() {
            Decoded::Digit(n) => Cow::Owned(BigInt::from(n)),
            Decoded::Big(n) => Cow::Borrowed(n),
        }
    }

    /// Converts this `CBigInt` into a `BigUint`, if it's not negative.
    pub fn to_biguint(&self) -> Option<BigUint> {
        match self.decode_ref() {
            Decoded::Digit(n) if n >= 0 => BigInt::from(n).to_biguint(),
            Decoded::Digit(_) => None,
            Decoded::Big(n) => n.to_biguint(),
        }
    }

    #[inline]
    pub fn checked_add(&self, v: &CBigInt) -> Option<CBigInt> {
        Some(self + v)
    }

    #[inline]
    pub fn checked_sub(&self, v: &CBigInt) -> Option<CBigInt> {
        Some(self - v)
    }

    #[inline]
    pub fn checked_mul(&self, v: &CBigInt) -> Option<CBigInt> {
        Some(self * v)
    }

    #[inline]
    pub fn checked_div(&self, v: &CBigInt) -> Option<CBigInt> {
        Some(self / v)
    }

    /// Returns `self ^ exponent`.
    pub fn pow(&self, exponent: u32) -> Self {
        if let Some(a) = self.to_digit() {
            if let (a, false) = a.overflowing_pow(exponent) {
                return a.into();
            }
        }
        BigInt::from(self.clone()).pow(exponent).into()
    }

    /// Returns `(self ^ exponent) mod modulus`
    ///
    /// Note that this rounds like `mod_floor`, not like the `%` operator,
    /// which makes a difference when given a negative `self` or `modulus`.
    /// The result will be in the interval `[0, modulus)` for `modulus > 0`,
    /// or in the interval `(modulus, 0]` for `modulus < 0`
    ///
    /// Panics if the exponent is negative or the modulus is zero.
    pub fn modpow(&self, exponent: &Self, modulus: &Self) -> Self {
        self.to_bigint()
            .modpow(&exponent.to_bigint(), &modulus.to_bigint())
            .into()
    }

    /// Returns the number of least-significant bits that are zero,
    /// or `None` if the entire number is zero.
    pub fn trailing_zeros(&self) -> Option<u64> {
        match self.decode_ref() {
            Decoded::Digit(0) => None,
            Decoded::Digit(n) if n > 0 => Some(n.trailing_zeros() as u64),
            Decoded::Digit(Digit::MIN) => Some(DIGIT_BITS as u64),
            Decoded::Digit(n) => Some((-n).trailing_zeros() as u64),
            Decoded::Big(n) => n.trailing_zeros(),
        }
    }
}

pub trait ToCBigInt {
    fn to_cbigint(&self) -> Option<CBigInt>;
}

impl<T> ToCBigInt for T
where
    T: Clone,
    CBigInt: TryFrom<T>,
{
    fn to_cbigint(&self) -> Option<CBigInt> {
        CBigInt::try_from(self.clone()).ok()
    }
}

impl Default for CBigInt {
    fn default() -> Self {
        CBigInt(Encoded::zero())
    }
}

impl Display for CBigInt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.decode_ref() {
            Decoded::Digit(n) => Display::fmt(&n, f),
            Decoded::Big(n) => Display::fmt(n, f),
        }
    }
}

impl Debug for CBigInt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if cfg!(feature = "debug_structure") {
            Debug::fmt(&self.decode_ref(), f)
        } else {
            Display::fmt(self, f)
        }
    }
}

#[test]
fn bits_test() {
    use num_traits::One;

    let mut nums: Vec<BigInt> = vec![0.into(), Digit::MAX.into(), Digit::MIN.into()];
    nums.extend((0..200).map(|x| BigInt::one() << x));

    for big in nums {
        assert_eq!(big.bits(), CBigInt::from(big.clone()).bits());
        assert_eq!((-big.clone()).bits(), CBigInt::from((-big).clone()).bits());
    }
}
