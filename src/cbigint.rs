use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::mem::size_of;

use num_bigint::{BigInt, BigUint, Sign};
use num_integer::Roots;

use crate::Sign::*;
use crate::accum::*;
use crate::big_integer::BigInteger;
use crate::encoding::Encoded;
use crate::{DIGIT_BITS, Digit, Udigit};

#[derive(Clone, Eq)]
pub struct CBigInt(pub(crate) Encoded<BigInt>);

impl CBigInt {
    pub(crate) fn to_digit(&self) -> Option<Digit> {
        match self.0 {
            Encoded::Digit(n) => Some(n),
            _ => None,
        }
    }

    pub(crate) fn to_digit_with(&self, other: &CBigInt) -> Option<(Digit, Digit)> {
        match (self.to_digit(), other.to_digit()) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }

    fn try_apply_sign(sign: Sign, magnitude: Udigit) -> Option<Self> {
        try_apply_sign(sign, magnitude).map(|digit| CBigInt(Encoded::Digit(digit)))
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
    pub fn magnitude(&self) -> Cow<'_, BigUint> {
        match &self.0 {
            Encoded::Digit(n) => Cow::Owned(BigInt::from(*n).into_parts().1),
            Encoded::Big(n) => Cow::Borrowed(n.magnitude()),
        }
    }

    /// Returns the magnitude of the `CBigInt` as a `BigUint` if the necessary
    /// `BigUint` already exists.
    pub fn try_magnitude(&self) -> Option<&BigUint> {
        match &self.0 {
            Encoded::Digit(_) => None,
            Encoded::Big(n) => Some(n.magnitude()),
        }
    }

    /// Converts this `CBigInt` into a `BigInt`.
    pub(crate) fn to_bigint(&self) -> Cow<'_, BigInt> {
        self.into()
    }

    pub(crate) fn normalized(&self) -> Cow<'_, CBigInt> {
        let result = match self.0 {
            Encoded::Big(_) => match Digit::try_from(self) {
                Ok(digit) => Cow::Owned(Self::from(digit)),
                _ => Cow::Borrowed(self),
            },
            Encoded::Digit(_) => Cow::Borrowed(self),
        };
        debug_assert!(result.is_normal());
        result
    }

    fn is_normal(&self) -> bool {
        matches!(self.0, Encoded::Digit(_)) || Digit::try_from(self).is_ok()
    }
}

impl BigInteger for CBigInt {
    /// Creates and initializes a BigInt.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    fn new(sign: Sign, digits: Vec<u32>) -> CBigInt {
        if sign == NoSign {
            return CBigInt(Encoded::zero());
        }

        // If the u32 digits fit into one Digit, we can avoid the overhead of creating a BigInt.
        if digits.len() <= size_of::<Udigit>() / size_of::<u32>() {
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

        CBigInt(Encoded::Big(BigInt::new(sign, digits)))
    }

    /// Creates and initializes a `CBigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    fn from_biguint(sign: Sign, data: BigUint) -> CBigInt {
        BigInt::from_biguint(sign, data).into()
    }

    /// Creates and initializes a `CBigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    fn from_slice(sign: Sign, slice: &[u32]) -> CBigInt {
        if slice.len() <= size_of::<Udigit>() / size_of::<u32>() {
            let mut accum = 0;
            for (i, &word) in slice.iter().enumerate() {
                accum |= (word as Udigit) << (i * 8);
            }
            if let Some(result) = Self::try_apply_sign(sign, accum) {
                return result;
            }
        }
        Self::new(sign, Vec::from(slice))
    }

    /// Reinitializes a `CBigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    fn assign_from_slice(&mut self, sign: Sign, slice: &[u32]) {
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
    fn from_bytes_be(sign: Sign, bytes: &[u8]) -> CBigInt {
        if let Some(accum) = bytes_to_digit_be(bytes)
            && let Some(result) = Self::try_apply_sign(sign, accum)
        {
            return result;
        }
        BigInt::from_bytes_be(sign, bytes).into()
    }

    /// Creates and initializes a `CBigInt`.
    ///
    /// The bytes are in little-endian byte order.
    fn from_bytes_le(sign: Sign, bytes: &[u8]) -> CBigInt {
        if let Some(accum) = bytes_to_digit_le(bytes)
            && let Some(result) = Self::try_apply_sign(sign, accum)
        {
            return result;
        }
        BigInt::from_bytes_le(sign, bytes).into()
    }

    /// Creates and initializes a `CBigInt` from an array of bytes in
    /// two's complement binary representation.
    ///
    /// The digits are in big-endian base 2<sup>8</sup>.
    fn from_signed_bytes_be(digits: &[u8]) -> CBigInt {
        if let Some(accum) = bytes_to_digit_be(digits) {
            (accum as Digit).into()
        } else {
            BigInt::from_signed_bytes_be(digits).into()
        }
    }

    /// Creates and initializes a `CBigInt` from an array of bytes in two's complement.
    ///
    /// The digits are in little-endian base 2<sup>8</sup>.
    fn from_signed_bytes_le(digits: &[u8]) -> CBigInt {
        if let Some(accum) = bytes_to_digit_le(digits) {
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
    fn parse_bytes(buf: &[u8], radix: u32) -> Option<CBigInt> {
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
    fn from_radix_be(sign: Sign, buf: &[u8], radix: u32) -> Option<CBigInt> {
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
    fn from_radix_le(sign: Sign, buf: &[u8], radix: u32) -> Option<CBigInt> {
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
    fn to_bytes_be(&self) -> (Sign, Vec<u8>) {
        match &self.0 {
            Encoded::Digit(n) => match sign_and_magnitude(*n) {
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
            Encoded::Big(n) => n.to_bytes_be(),
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
    fn to_bytes_le(&self) -> (Sign, Vec<u8>) {
        match &self.0 {
            Encoded::Digit(n) => {
                let (sign, accum) = sign_and_magnitude(*n);
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
            Encoded::Big(n) => n.to_bytes_le(),
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
    fn to_u32_digits(&self) -> (Sign, Vec<u32>) {
        match &self.0 {
            Encoded::Digit(n) => match sign_and_magnitude(*n) {
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
            Encoded::Big(n) => n.to_u32_digits(),
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
    fn to_signed_bytes_be(&self) -> Vec<u8> {
        match &self.0 {
            Encoded::Digit(0) => Vec::new(),
            Encoded::Digit(n) => {
                let bytes = n.to_be_bytes();
                let to_discard = if *n >= 0 { 0 } else { 0xff };
                let mut i = 0;
                while i < bytes.len() && bytes[i] == to_discard {
                    i += 1
                }
                bytes[i..].to_vec()
            }
            Encoded::Big(n) => n.to_signed_bytes_be(),
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
    fn to_signed_bytes_le(&self) -> Vec<u8> {
        match &self.0 {
            Encoded::Digit(0) => Vec::new(),
            Encoded::Digit(n) => {
                let bytes = n.to_le_bytes();
                let to_discard = if *n >= 0 { 0 } else { 0xff };
                let mut i = size_of::<Digit>();
                while i > 0 && bytes[i - 1] == to_discard {
                    i -= 1
                }
                bytes[..i].to_vec()
            }
            Encoded::Big(n) => n.to_signed_bytes_le(),
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
    fn to_str_radix(&self, radix: u32) -> String {
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
    fn to_radix_be(&self, radix: u32) -> (Sign, Vec<u8>) {
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
    fn to_radix_le(&self, radix: u32) -> (Sign, Vec<u8>) {
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
    fn sign(&self) -> Sign {
        match &self.0 {
            Encoded::Digit(n) => {
                if *n > 0 {
                    Plus
                } else if *n < 0 {
                    Minus
                } else {
                    NoSign
                }
            }
            Encoded::Big(n) => n.sign(),
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
    fn into_parts(self) -> (Sign, BigUint) {
        BigInt::from(self).into_parts()
    }

    /// Returns whether the bit in position `bit` is set, using the two’s complement for negative numbers
    fn bit(&self, bit: u64) -> bool {
        match &self.0 {
            &Encoded::Digit(digit) => (digit as Udigit >> bit) & 1 == 1,
            Encoded::Big(big) => big.bit(bit),
        }
    }

    /// Determines the fewest bits necessary to express the `BigInt`,
    /// not including the sign.
    fn bits(&self) -> u64 {
        match &self.0 {
            &Encoded::Digit(n) => {
                if n >= 0 {
                    DIGIT_BITS as u32 - n.leading_zeros()
                } else {
                    DIGIT_BITS as u32 - n.unsigned_abs().leading_zeros()
                }
            }
            .into(),
            Encoded::Big(n) => n.bits(),
        }
    }

    /// Converts this `CBigInt` into a `BigUint`, if it's not negative.
    fn to_biguint(&self) -> Option<BigUint> {
        self.to_bigint().to_biguint()
    }

    fn checked_add(&self, v: &CBigInt) -> Option<CBigInt> {
        Some(self + v)
    }

    fn checked_sub(&self, v: &CBigInt) -> Option<CBigInt> {
        Some(self - v)
    }

    fn checked_mul(&self, v: &CBigInt) -> Option<CBigInt> {
        Some(self * v)
    }

    fn checked_div(&self, v: &CBigInt) -> Option<CBigInt> {
        Some(self / v)
    }

    /// Returns `self ^ exponent`.
    fn pow(&self, exponent: u32) -> Self {
        if let Some(a) = self.to_digit()
            && let (a, false) = a.overflowing_pow(exponent)
        {
            return a.into();
        }
        self.to_bigint().pow(exponent).into()
    }

    /// Returns `(self ^ exponent) mod modulus`
    ///
    /// Note that this rounds like `mod_floor`, not like the `%` operator,
    /// which makes a difference when given a negative `self` or `modulus`.
    /// The result will be in the interval `[0, modulus)` for `modulus > 0`,
    /// or in the interval `(modulus, 0]` for `modulus < 0`
    ///
    /// Panics if the exponent is negative or the modulus is zero.
    fn modpow(&self, exponent: &Self, modulus: &Self) -> Self {
        self.to_bigint()
            .modpow(&exponent.to_bigint(), &modulus.to_bigint())
            .into()
    }

    /// Returns the truncated principal square root of self.
    fn sqrt(&self) -> Self {
        match &self.0 {
            Encoded::Digit(n) => Self::from(n.sqrt()),
            Encoded::Big(n) => Self::from(n.sqrt()),
        }
    }

    /// Returns the truncated principal cube root of self.
    fn cbrt(&self) -> Self {
        match &self.0 {
            Encoded::Digit(n) => Self::from(n.cbrt()),
            Encoded::Big(n) => Self::from(n.cbrt()),
        }
    }

    /// Returns the truncated principal nth root of self.
    fn nth_root(&self, n: u32) -> Self {
        match &self.0 {
            Encoded::Digit(x) => Self::from(x.nth_root(n)),
            Encoded::Big(x) => Self::from(x.nth_root(n)),
        }
    }

    /// Returns the number of least-significant bits that are zero,
    /// or `None` if the entire number is zero.
    fn trailing_zeros(&self) -> Option<u64> {
        match &self.0 {
            Encoded::Digit(0) => None,
            Encoded::Digit(n) if *n > 0 => Some(n.trailing_zeros() as u64),
            Encoded::Digit(Digit::MIN) => Some(DIGIT_BITS as u64),
            Encoded::Digit(n) => Some((-*n).trailing_zeros() as u64),
            Encoded::Big(n) => n.trailing_zeros(),
        }
    }

    const ZERO: Self = CBigInt(Encoded::zero());

    fn to_u64_digits(&self) -> (Sign, Vec<u64>) {
        self.to_bigint().to_u64_digits()
    }

    fn iter_u32_digits(
        &self,
    ) -> impl DoubleEndedIterator<Item = u32> + ExactSizeIterator<Item = u32> + '_ {
        self.to_bigint()
            .iter_u32_digits()
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn iter_u64_digits(
        &self,
    ) -> impl DoubleEndedIterator<Item = u64> + ExactSizeIterator<Item = u64> + '_ {
        self.to_bigint()
            .iter_u64_digits()
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn modinv(&self, modulus: &Self) -> Option<Self> {
        BigInt::from(self)
            .modinv(&modulus.to_bigint())
            .map(Self::from)
    }

    fn set_bit(&mut self, bit: u64, value: bool) {
        match &mut self.0 {
            Encoded::Digit(n) if (bit as usize) < DIGIT_BITS - 1 => {
                let mask = 1 << bit;
                if value {
                    *n |= mask;
                } else {
                    *n &= !mask;
                }
            }
            Encoded::Digit(n) => {
                let mut big = BigInt::from(*n);
                big.set_bit(bit, value);
                self.0 = Encoded::Big(big);
            }
            Encoded::Big(n) => n.set_bit(bit, value),
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
        match &self.0 {
            Encoded::Digit(n) => Display::fmt(n, f),
            Encoded::Big(n) => Display::fmt(n, f),
        }
    }
}

impl Debug for CBigInt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if cfg!(feature = "debug_structure") {
            Debug::fmt(&self.0, f)
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
