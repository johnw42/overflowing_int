use std::borrow::Cow;
use std::cmp::Ordering;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Debug, Display, Formatter};
use std::mem::size_of;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};
use num_integer::{Integer, Roots};
use num_traits::{Num, One, Signed, ToPrimitive, Zero};

use crate::encoding::{Decoded, Encoded};
use crate::overflowing::Overflowing;
use crate::Sign::*;
use crate::{Digit, Udigit};

#[derive(Clone)]
pub struct CBigInt(Encoded);

impl Debug for CBigInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.decode_ref().fmt(f)
    }
}

impl PartialEq for CBigInt {
    fn eq(&self, other: &Self) -> bool {
        self.decode_ref().eq(&other.decode_ref())
    }
}

impl Eq for CBigInt {}

type GenInt = Decoded<BigInt>;
type GenIntRef<'a> = Decoded<&'a BigInt>;
type GenIntCow<'a> = Decoded<Cow<'a, BigInt>>;

impl From<CBigInt> for GenInt {
    fn from(arg: CBigInt) -> Self {
        arg.0.decode()
    }
}

impl<'a> From<GenInt> for GenIntCow<'a> {
    fn from(value: GenInt) -> Self {
        match value {
            GenInt::Small(x) => GenIntCow::Small(x),
            GenInt::Big(x) => GenIntCow::Big(Cow::Owned(x)),
        }
    }
}

impl<'a> From<GenIntRef<'a>> for GenIntCow<'a> {
    fn from(value: GenIntRef<'a>) -> Self {
        match value {
            GenIntRef::Small(x) => GenIntCow::Small(x),
            GenIntRef::Big(x) => GenIntCow::Big(Cow::Borrowed(x)),
        }
    }
}

const DIGIT_BITS: usize = size_of::<Digit>() * 8;

impl CBigInt {
    #[inline(always)]
    pub(crate) fn from_small_int(n: Digit) -> CBigInt {
        CBigInt(Decoded::Small(n).encode())
    }

    #[inline]
    fn decode(self) -> Decoded<BigInt> {
        self.0.decode()
    }

    #[inline]
    fn decode_ref(&self) -> Decoded<&BigInt> {
        self.0.decode_ref()
    }

    #[inline]
    fn to_digit(&self) -> Option<Digit> {
        match self.decode_ref() {
            Decoded::Small(n) => Some(n),
            _ => None,
        }
    }

    #[inline]
    fn to_digit_with(&self, other: &CBigInt) -> Option<(Digit, Digit)> {
        match (self.to_digit(), other.to_digit()) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }

    /// Creates and initializes a BigInt.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
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

    #[inline]
    pub fn from_bigint(data: BigInt) -> CBigInt {
        let decoded = match Digit::try_from(data) {
            Ok(digit) => Decoded::Small(digit),
            Err(err) => Decoded::Big(err.into_original()),
        };
        CBigInt(decoded.encode())
    }

    /// Creates and initializes a `CBigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn from_biguint(sign: Sign, data: BigUint) -> CBigInt {
        Self::from_bigint(BigInt::from_biguint(sign, data))
    }

    #[inline(always)]
    fn from_accum(sign: Sign, accum: Udigit) -> Option<CBigInt> {
        let accum = accum as Digit;
        if accum >= 0 {
            Some(CBigInt(
                Decoded::Small(match sign {
                    Plus => accum,
                    Minus => -accum,
                    NoSign => 0,
                })
                .encode(),
            ))
        } else {
            None
        }
    }

    #[inline(always)]
    fn accum_be(bytes: &[u8]) -> Option<Udigit> {
        if bytes.len() <= size_of::<Digit>() {
            let mut accum = 0;
            for &byte in bytes {
                accum = accum << 8 | byte as Udigit;
            }
            Some(accum)
        } else {
            None
        }
    }

    #[inline(always)]
    fn accum_le(bytes: &[u8]) -> Option<Udigit> {
        if bytes.len() <= size_of::<Digit>() {
            let mut accum = 0;
            for (i, &byte) in bytes.iter().enumerate() {
                accum |= (byte as Udigit) << 8 * i;
            }
            Some(accum)
        } else {
            None
        }
    }

    /// Creates and initializes a `CBigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    pub fn from_slice(sign: Sign, slice: &[u32]) -> CBigInt {
        if slice.len() <= size_of::<Udigit>() / size_of::<u32>() {
            let mut accum = 0;
            for (i, &word) in slice.iter().enumerate() {
                accum |= (word as Udigit) << i * 8;
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
        if let Some(accum) = Self::accum_be(bytes) {
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
        if let Some(accum) = Self::accum_le(bytes) {
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
        if let Some(accum) = Self::accum_be(digits) {
            Self::from_small_int(accum as Digit)
        } else {
            Self::from_bigint(BigInt::from_signed_bytes_be(digits))
        }
    }

    /// Creates and initializes a `CBigInt` from an array of bytes in two's complement.
    ///
    /// The digits are in little-endian base 2<sup>8</sup>.
    pub fn from_signed_bytes_le(digits: &[u8]) -> CBigInt {
        if let Some(accum) = Self::accum_le(digits) {
            Self::from_small_int(accum as Digit)
        } else {
            Self::from_bigint(BigInt::from_signed_bytes_le(digits))
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
        BigInt::parse_bytes(buf, radix).map(Self::from_bigint)
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
        BigInt::from_radix_be(sign, buf, radix).map(Self::from_bigint)
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
        BigInt::from_radix_le(sign, buf, radix).map(Self::from_bigint)
    }

    fn make_accum(value: Digit) -> (Sign, Udigit) {
        if value == 0 {
            (NoSign, 0)
        } else if value >= 0 {
            (Plus, value as Udigit)
        } else if value == Digit::MIN {
            (Minus, value as Udigit)
        } else {
            (Minus, (-value) as Udigit)
        }
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
            Decoded::Small(n) => match Self::make_accum(n) {
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
            Decoded::Small(n) => {
                let (sign, accum) = Self::make_accum(n);
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
    pub fn to_u32_digits(&self) -> (Sign, Vec<u32>) {
        match self.decode_ref() {
            Decoded::Small(n) => match Self::make_accum(n) {
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
            Decoded::Small(0) => Vec::new(),
            Decoded::Small(n) => {
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
            Decoded::Small(0) => Vec::new(),
            Decoded::Small(n) => {
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
            Decoded::Small(n) => {
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
    pub fn magnitude(&self) -> Cow<BigUint> {
        match self.decode_ref() {
            Decoded::Small(n) => Cow::Owned(BigInt::from(n).into_parts().1),
            Decoded::Big(n) => Cow::Borrowed(n.magnitude()),
        }
    }

    /// Returns the magnitude of the `CBigInt` as a `BigUint` if the necessary
    /// `BigUint` already exists.
    #[inline]
    pub fn try_magnitude(&self) -> Option<&BigUint> {
        match self.decode_ref() {
            Decoded::Small(_) => None,
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
            Decoded::Small(n) => {
                if n >= 0 {
                    DIGIT_BITS as u32 - 1 - n.leading_zeros()
                } else if n == Digit::MIN {
                    DIGIT_BITS as u32
                } else {
                    (-n).leading_zeros()
                }
            }
            .into(),
            Decoded::Big(n) => n.bits(),
        }
    }

    /// Converts this `CBigInt` into a `BigInt`.
    #[inline]
    fn into_bigint(self) -> BigInt {
        match self.decode() {
            Decoded::Small(n) => BigInt::from(n),
            Decoded::Big(n) => n,
        }
    }

    /// Converts this `CBigInt` into a `BigInt`.
    #[inline]
    pub fn to_bigint(&self) -> Cow<BigInt> {
        match self.decode_ref() {
            Decoded::Small(n) => Cow::Owned(BigInt::from(n)),
            Decoded::Big(n) => Cow::Borrowed(n),
        }
    }

    /// Converts this `CBigInt` into a `BigUint`, if it's not negative.
    pub fn to_biguint(&self) -> Option<BigUint> {
        match self.decode_ref() {
            Decoded::Small(n) if n >= 0 => BigInt::from(n).to_biguint(),
            Decoded::Small(_) => None,
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
            .modpow(&*exponent.to_bigint(), &*modulus.to_bigint())
            .into()
    }

    /// Returns the number of least-significant bits that are zero,
    /// or `None` if the entire number is zero.
    pub fn trailing_zeros(&self) -> Option<u64> {
        match self.decode_ref() {
            Decoded::Small(0) => None,
            Decoded::Small(n) if n > 0 => Some(n.trailing_zeros() as u64),
            Decoded::Small(Digit::MIN) => Some(DIGIT_BITS as u64),
            Decoded::Small(n) => Some((-n).trailing_zeros() as u64),
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.decode_ref() {
            Decoded::Small(n) => write!(f, "{}", n),
            Decoded::Big(n) => write!(f, "{}", n),
        }
    }
}

impl ToBigInt for CBigInt {
    fn to_bigint(&self) -> Option<BigInt> {
        Some(Cow::into_owned(self.to_bigint()))
    }
}

impl ToBigUint for CBigInt {
    fn to_biguint(&self) -> Option<BigUint> {
        self.clone().try_into().ok()
    }
}

impl From<BigInt> for CBigInt {
    fn from(value: BigInt) -> Self {
        Self::from_bigint(value)
    }
}

impl From<BigUint> for CBigInt {
    fn from(value: BigUint) -> Self {
        Self::from_biguint(Plus, value)
    }
}

impl From<CBigInt> for BigInt {
    fn from(value: CBigInt) -> Self {
        value.into_bigint()
    }
}

impl TryFrom<CBigInt> for BigUint {
    type Error = TryFromBigIntError<()>;
    fn try_from(value: CBigInt) -> Result<Self, Self::Error> {
        match value.0.decode() {
            Decoded::Small(n) => n.to_biguint(),
            Decoded::Big(n) => n.to_biguint(),
        }
        .ok_or_else(try_into_bigint_error)
    }
}

impl TryFrom<&CBigInt> for BigUint {
    type Error = TryFromBigIntError<()>;
    fn try_from(value: &CBigInt) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}

impl Zero for CBigInt {
    fn zero() -> Self {
        CBigInt::default()
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl One for CBigInt {
    fn one() -> Self {
        CBigInt::from_small_int(1)
    }

    fn is_one(&self) -> bool {
        self.0.is_one()
    }
}

impl Num for CBigInt {
    type FromStrRadixErr = ParseBigIntError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        BigInt::from_str_radix(str, radix).map(CBigInt::from_bigint)
    }
}

impl Signed for CBigInt {
    fn abs(&self) -> Self {
        match self.decode_ref() {
            Decoded::Small(a) => {
                if let (b, false) = a.overflowing_abs() {
                    b.into()
                } else {
                    BigInt::from(a).abs().into()
                }
            }
            Decoded::Big(a) => a.abs().into(),
        }
    }

    fn abs_sub(&self, other: &Self) -> Self {
        self.sub(other).abs()
    }

    fn signum(&self) -> Self {
        match self.sign() {
            NoSign => 0,
            Plus => 1,
            Minus => -1,
        }
        .into()
    }

    fn is_positive(&self) -> bool {
        self.sign() == Plus
    }

    fn is_negative(&self) -> bool {
        self.sign() == Minus
    }
}

impl PartialOrd for CBigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CBigInt {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.decode_ref(), other.decode_ref()) {
            (Decoded::Small(a), Decoded::Small(b)) => a.cmp(&b),
            (Decoded::Big(a), Decoded::Big(b)) => a.cmp(b),
            _ => self
                .sign()
                .cmp(&other.sign())
                .then_with(|| self.to_bigint().cmp(&other.to_bigint())),
        }
    }
}

impl Integer for CBigInt {
    fn div_floor(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_digit_with(other) {
            if (lhs, rhs) != (Digit::MIN, -1) {
                return lhs.div_floor(&rhs).into();
            }
        }
        self.to_bigint().div_floor(&*other.to_bigint()).into()
    }

    fn mod_floor(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_digit_with(other) {
            if (lhs, rhs) != (Digit::MIN, -1) {
                return lhs.mod_floor(&rhs).into();
            }
        }
        self.to_bigint().mod_floor(&*other.to_bigint()).into()
    }

    fn gcd(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_digit_with(other) {
            return lhs.gcd(&rhs).into();
        }
        self.to_bigint().gcd(&*other.to_bigint()).into()
    }

    fn lcm(&self, other: &Self) -> Self {
        if let Some((lhs, rhs)) = self.to_digit_with(other) {
            return lhs.lcm(&rhs).into();
        }
        self.to_bigint().lcm(&*other.to_bigint()).into()
    }

    fn divides(&self, other: &Self) -> bool {
        if let Some((lhs, rhs)) = self.to_digit_with(other) {
            return lhs.divides(&rhs).into();
        }
        self.to_bigint().divides(&*other.to_bigint()).into()
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        if let Some((lhs, rhs)) = self.to_digit_with(other) {
            return lhs.is_multiple_of(&rhs).into();
        }
        self.to_bigint().is_multiple_of(&*other.to_bigint()).into()
    }

    fn is_even(&self) -> bool {
        match self.decode_ref() {
            Decoded::Small(n) => n.is_even(),
            Decoded::Big(n) => n.is_even(),
        }
    }

    fn is_odd(&self) -> bool {
        match self.decode_ref() {
            Decoded::Small(n) => n.is_odd(),
            Decoded::Big(n) => n.is_odd(),
        }
    }

    fn div_rem(&self, other: &Self) -> (Self, Self) {
        if let Some((lhs, rhs)) = self.to_digit_with(other) {
            if (lhs, rhs) != (Digit::MIN, -1) {
                let (q, r) = lhs.div_rem(&rhs);
                return (q.into(), r.into());
            }
        }
        let (q, r) = self.to_bigint().div_rem(&*other.to_bigint());
        return (q.into(), r.into());
    }
}

#[test]
fn gcd_test() {
    let small = CBigInt::from(5);
    let huge = CBigInt::from(i128::MAX).pow(2);
    assert_eq!(huge.gcd(&small), CBigInt::from(1));
    assert_eq!(small.gcd(&huge), CBigInt::from(1));
}

impl Roots for CBigInt {
    fn nth_root(&self, n: u32) -> Self {
        match self.decode_ref() {
            Decoded::Small(a) => a.nth_root(n).into(),
            Decoded::Big(a) => a.nth_root(n).into(),
        }
    }
}

impl Neg for CBigInt {
    type Output = CBigInt;

    fn neg(self) -> Self::Output {
        if let Some(a) = self.to_digit() {
            if let (b, false) = a.overflowing_neg() {
                return b.into();
            }
        }
        BigInt::from(self).neg().into()
    }
}

impl Neg for &CBigInt {
    type Output = CBigInt;

    fn neg(self) -> Self::Output {
        if let Some(a) = self.to_digit() {
            if let (b, false) = a.overflowing_neg() {
                return b.into();
            }
        }
        (&*self.to_bigint()).neg().into()
    }
}

impl Not for CBigInt {
    type Output = CBigInt;

    fn not(self) -> Self::Output {
        match self.decode() {
            Decoded::Small(n) => n.not().into(),
            Decoded::Big(n) => n.not().into(),
        }
    }
}

impl Not for &CBigInt {
    type Output = CBigInt;

    fn not(self) -> Self::Output {
        match self.decode_ref() {
            Decoded::Small(n) => n.not().into(),
            Decoded::Big(n) => n.not().into(),
        }
    }
}

// We can't constructor a TryFromBigIntError directly, so we get sneaky.
fn try_into_bigint_error() -> TryFromBigIntError<()> {
    BigUint::try_from(-1).expect_err("converting -1 to BigUint fails")
}

macro_rules! each_prim {
    [[int $(, $_1:tt)*], [$prim:ident, $to_prim:ident]] => {
        impl From<$prim> for CBigInt {
            fn from(value: $prim) -> Self {
                if let Ok(converted) = Digit::try_from(value) {
                    CBigInt::from_small_int(converted)
                } else {
                    BigInt::from(value).into()
                }
            }
        }
        impl TryFrom<CBigInt> for $prim {
            type Error = TryFromBigIntError<BigInt>;
            fn try_from(value: CBigInt) -> Result<Self, Self::Error> {
                if let Some(n) = value.to_digit() {
                    match n.$to_prim() {
                        Some(prim) => Ok(prim),
                        None => {
                            // This is guaranteed to fail; it's done because there's no more
                            // straightforward way to construct an appropriate TryFromBigIntError.
                            $prim::try_from(BigInt::from(value))
                        }
                    }
                } else {
                    $prim::try_from(BigInt::from(value))
                }
            }
        }
    };
    [[float $(, $_1:tt)*], $prim_attrs:tt] => {
    };
}

macro_rules! to_prim_method {
    [$_1:tt, [$prim:ident, $to_prim:ident]] => {
        fn $to_prim(&self) -> Option<$prim> {
            match self.decode_ref() {
                Decoded::Small(value) => value.$to_prim(),
                Decoded::Big(value) => value.$to_prim(),
            }
        }
    };
}

macro_rules! each_op {
    [arith_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        impl $trait for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: Self) -> Self::Output {
                if let Some((a, b)) = self.to_digit_with(&rhs) {
                    if let (c, false) = Overflowing::$op(a, b) {
                        return c.into();
                    }
                }
                BigInt::from(self).$op(BigInt::from(rhs)).into()
                //dbg!(dbg!(BigInt::from(self)).$op(dbg!(BigInt::from(rhs)))).into()
            }
        }
        assign_op!($trait, $op, $assign_trait, $assign_op);
        ref_op!($trait<CBigInt> for CBigInt, $op);
    };
    [shift_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        assign_op!($trait, $op, $assign_trait, $assign_op);
    };
    [bit_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        impl $trait for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: Self) -> Self::Output {
                if let Some((a, b)) = self.to_digit_with(&rhs) {
                    return a.$op(b).into();
                }
                BigInt::from(self).$op(BigInt::from(rhs)).into()
            }
        }
        assign_op![$trait, $op, $assign_trait, $assign_op];
        ref_op![$trait<CBigInt> for CBigInt, $op];
    };
}

macro_rules! ref_op {
    [$trait:ident<$rhs_type:ty> for $lhs_type:ty, $op:ident] => {
        impl $trait<&$rhs_type> for $lhs_type {
            type Output = CBigInt;
            fn $op(self, rhs: &$rhs_type) -> CBigInt {
                self.$op(rhs.clone())
            }
        }
        impl $trait<$rhs_type> for &$lhs_type {
            type Output = CBigInt;
            fn $op(self, rhs: $rhs_type) -> CBigInt {
                self.clone().$op(rhs)
            }
        }
        impl $trait<&$rhs_type> for &$lhs_type {
            type Output = CBigInt;
            fn $op(self, rhs: &$rhs_type) -> CBigInt {
                self.clone().$op(rhs.clone())
            }
        }
    };
}

macro_rules! assign_op {
    [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident] => {
        impl<T> $assign_trait<T> for CBigInt
        where
            CBigInt: $trait<T, Output = CBigInt>,
        {
            fn $assign_op(&mut self, rhs: T) {
                let lhs = std::mem::take(self);
                *self = lhs.$op(rhs);
            }
        }
    };
}

macro_rules! each_prim_and_op {
    [
        [int $(, $_1:tt)*], [$prim:ident, $to_prim:ident],
        arith_op, [
            $trait:ident,
            $op:ident,
            $assign_trait:ident,
            $assign_op:ident,
        ]
    ] => {
        impl $trait<$prim> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: $prim) -> Self::Output {
                if let Small(prim) = &self {
                    if let Ok(promoted) = Digit::try_from(rhs) {
                        if let (result, false) = Overflowing::$op(prim, promoted) {
                            return result.into();
                        }
                    }
                }
                BigInt::from(self).$op(rhs).into()
            }
        }
        impl $trait<CBigInt> for $prim {
            type Output = CBigInt;
            fn $op(self, rhs: CBigInt) -> Self::Output {
                if let Small(prim) = &rhs {
                    if let Ok(promoted) = Digit::try_from(self) {
                        if let (result, false) = Overflowing::$op(promoted, *prim) {
                            return result.into();
                        }
                    }
                }
                self.$op(BigInt::from(rhs)).into()
            }
        }
        ref_op!($trait<$prim> for CBigInt, $op);
        ref_op!($trait<CBigInt> for $prim, $op);
    };
    [
        [int $(, $_1:tt)*], [$prim:ident, $to_prim:ident],
        shift_op, [
            $trait:ident,
            $op:ident,
            $assign_trait:ident,
            $assign_op:ident,
            $overflowing_op:ident
        ]
    ] => {
        impl $trait<$prim> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: $prim) -> Self::Output {
                if let Small(lhs) = &self {
                    if let Ok(rhs) = u32::try_from(rhs) {
                        if let (result, false) = lhs.$overflowing_op(rhs) {
                            return result.into();
                        }
                    }
                }
                BigInt::from(self).$op(rhs).into()
            }
        }
        ref_op!($trait<$prim> for CBigInt, $op);
    };
    [$($_1:tt),*] => {};
}

impl ToPrimitive for CBigInt {
    with_prims!(to_prim_method, []);
}

with_prims!(each_prim, []);
with_prims_and_ops!(each_prim_and_op, []);
with_ops!(each_op, []);

#[test]
fn test() {
    let bin_ops: &[(
        &str,
        fn(CBigInt, CBigInt) -> CBigInt,
        fn(BigInt, BigInt) -> BigInt,
    )] = &[
        ("+", CBigInt::add, BigInt::add),
        ("-", CBigInt::sub, BigInt::sub),
        ("*", CBigInt::mul, BigInt::mul),
        ("/", CBigInt::div, BigInt::div),
        ("%", CBigInt::rem, BigInt::rem),
    ];
    let mut small_range = vec![Digit::MIN, Digit::MAX, -Digit::MAX];
    small_range.extend((-10..=10).into_iter());
    let mut range: Vec<_> = small_range.into_iter().map(BigInt::from).collect();
    range.push(BigInt::from(i128::MAX) * 2);
    range.push(BigInt::from(i128::MIN) * 2);

    for (op_name, cop, op) in bin_ops {
        for a in &range {
            for b in &range {
                if !b.is_zero() {
                    let expected = op(a.clone(), b.clone());
                    let actual =
                        BigInt::from(cop(CBigInt::from(a.clone()), CBigInt::from(b.clone())));
                    assert_eq!(
                        expected, actual,
                        "failed: {} {} {} == {} (got {})",
                        a, op_name, b, expected, actual
                    );
                }
            }
        }
    }
}
