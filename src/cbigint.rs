use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::mem::size_of;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

use num_bigint::{BigInt, BigUint, Sign, TryFromBigIntError};
#[allow(unused_imports)]
use num_traits::{ToPrimitive, Zero};

use crate::accum::*;
use crate::decoded::Decoded;
use crate::encoding::Encoded;
use crate::overflowing::Overflowing;
use crate::to_cow::{ToCow, ToDecodedCow};
use crate::Sign::*;
use crate::{Digit, Udigit, DIGIT_BITS};

#[derive(Clone)]
pub struct CBigInt(pub(crate) Encoded);

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
    pub fn magnitude(&self) -> Cow<BigUint> {
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
    pub fn to_bigint(&self) -> Cow<BigInt> {
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
            .modpow(&*exponent.to_bigint(), &*modulus.to_bigint())
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

struct BigIntOp {
    digits: fn(Digit, Digit) -> Option<Digit>,
    owned: fn(BigInt, BigInt) -> BigInt,
    owned_borrowed: fn(BigInt, &BigInt) -> BigInt,
    borrowed_owned: for<'a> fn(&'a BigInt, BigInt) -> BigInt,
    borrowed: for<'a> fn(&'a BigInt, &'a BigInt) -> BigInt,
}

impl BigIntOp {
    fn call<'a, L, R>(&self, lhs: L, rhs: R) -> CBigInt
    where
        L: ToDecodedCow<'a>,
        R: ToDecodedCow<'a>,
    {
        use Cow::*;
        let lhs = lhs.to_decoded_cow();
        let rhs = rhs.to_decoded_cow();

        if let (&Decoded::Digit(lhs), &Decoded::Digit(rhs)) = (&lhs, &rhs) {
            if let Some(out) = (self.digits)(lhs, rhs) {
                return out.into();
            }
        }

        match (lhs.to_cow(), rhs.to_cow()) {
            (Owned(lhs), Owned(rhs)) => (self.owned)(lhs, rhs),
            (Owned(lhs), Borrowed(rhs)) => (self.owned_borrowed)(lhs, rhs),
            (Borrowed(lhs), Owned(rhs)) => (self.borrowed_owned)(lhs, rhs),
            (Borrowed(lhs), Borrowed(rhs)) => (self.borrowed)(lhs, rhs),
        }
        .into()
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
            Decoded::Digit(n) => write!(f, "{}", n),
            Decoded::Big(n) => write!(f, "{}", n),
        }
    }
}

macro_rules! each_prim {
    [[int $(, $int_attr:tt)*], [$prim:ident, $to_prim:ident]] => {
        impl From<$prim> for CBigInt {
            fn from(value: $prim) -> Self {
                if let Ok(digit) = Digit::try_from(value) {
                    CBigInt(Decoded::Digit(digit).encode())
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
        with_ops!(each_prim_and_op, [[int $(, $int_attr)*], [$prim, $to_prim]]);
    };
    [[float $(, $float_attr:tt)*], $prim_attrs:tt] => {
        with_ops!(each_prim_and_op, [[float $(, $float_attr)*], $prim_attrs]);
    };
}

macro_rules! bigint_op {
    [arith_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        pub(super) const $op: BigIntOp = BigIntOp {
            digits: |lhs, rhs| {
                if let (out, false) = Overflowing::$op(lhs, rhs) {
                    Some(out)
                } else {
                    None
                }
            },
            owned: |lhs: BigInt, rhs: BigInt| $trait::$op(lhs, rhs),
            owned_borrowed: |lhs: BigInt, rhs: &BigInt| $trait::$op(lhs, rhs),
            borrowed_owned: |lhs: &BigInt, rhs: BigInt| $trait::$op(lhs, rhs),
            borrowed: |lhs: &BigInt, rhs: &BigInt| $trait::$op(lhs, rhs),
        };
    };
    [bit_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        pub(super) const $op: BigIntOp = BigIntOp {
            digits: |lhs, rhs| Some($trait::$op(lhs, rhs)),
            owned: |lhs: BigInt, rhs: BigInt| $trait::$op(lhs, rhs),
            owned_borrowed: |lhs: BigInt, rhs: &BigInt| $trait::$op(lhs, rhs),
            borrowed_owned: |lhs: &BigInt, rhs: BigInt| $trait::$op(lhs, rhs),
            borrowed: |lhs: &BigInt, rhs: &BigInt| $trait::$op(lhs, rhs),
        };
    };
    [$($_1:tt),*] => {};
}

#[allow(non_upper_case_globals)]
mod bigint_ops {
    use super::*;

    with_ops!(bigint_op, []);
}

macro_rules! op_traits {
    [arith_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        impl $trait<CBigInt> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: CBigInt) -> Self::Output {
                bigint_ops::$op.call(self, rhs)
            }
        }
        impl $trait<CBigInt> for &CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: CBigInt) -> Self::Output {
                bigint_ops::$op.call(self, rhs)
            }
        }
        impl $trait<&CBigInt> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: &CBigInt) -> Self::Output {
                bigint_ops::$op.call(self, rhs)
            }
        }
        impl $trait<&CBigInt> for &CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: &CBigInt) -> Self::Output {
                bigint_ops::$op.call(self, rhs)
            }
        }
        assign_op!($trait, $op, $assign_trait, $assign_op);
    };
    [shift_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        //assign_op!($trait, $op, $assign_trait, $assign_op);
    };
    [bit_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        op_traits!(arith_op, [$trait, $op, $assign_trait, $assign_op]);
        // impl<T> $assign_trait<T> for CBigInt
        // where
        //     CBigInt: $trait<T, Output = CBigInt>,
        //     BigInt: $assign_trait<T>,
        // {
        //     fn $assign_op(&mut self, rhs: T) {
        //         match self.decode_mut() {
        //             Decoded::Digit(_) => {
        //                 let lhs = std::mem::take(self);
        //                 *self = lhs.$op(rhs);
        //             }
        //             Decoded::Big(big) => {
        //                 big.$assign_op(rhs);
        //             }
        //         }
        //     }
        // }
    };
}

macro_rules! assign_op {
    [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident] => {
        impl<T> $assign_trait<T> for CBigInt
        where
            CBigInt: $trait<T, Output = CBigInt>,
            BigInt: From<T>,
            BigInt: $assign_trait,
        {
            fn $assign_op(&mut self, rhs: T) {
                match self.decode_mut() {
                    Decoded::Digit(_) => {
                        let lhs = std::mem::take(self);
                        *self = lhs.$op(rhs);
                    }
                    Decoded::Big(big) => {
                        big.$assign_op(BigInt::from(rhs));
                    }
                }
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
            $assign_op:ident
        ]
    ] => {
        // impl $trait<$prim> for CBigInt {
        //     type Output = CBigInt;
        //     fn $op(self, rhs: $prim) -> Self::Output {
        //         if let Small(prim) = &self {
        //             if let Ok(promoted) = Digit::try_from(rhs) {
        //                 if let (result, false) = Overflowing::$op(prim, promoted) {
        //                     return result.into();
        //                 }
        //             }
        //         }
        //         BigInt::from(self).$op(rhs).into()
        //     }
        // }
        // impl $trait<CBigInt> for $prim {
        //     type Output = CBigInt;
        //     fn $op(self, rhs: CBigInt) -> Self::Output {
        //         if let Small(prim) = &rhs {
        //             if let Ok(promoted) = Digit::try_from(self) {
        //                 if let (result, false) = Overflowing::$op(promoted, *prim) {
        //                     return result.into();
        //                 }
        //             }
        //         }
        //         self.$op(BigInt::from(rhs)).into()
        //     }
        // }
        // ref_op!($trait<$prim> for CBigInt, $op);
        // ref_op!($trait<CBigInt> for $prim, $op);
    };
    [
        [int $(, $_1:tt)*], [$prim:ident, $to_prim:ident],
        shift_op, [
            $trait:ident,
            $op:ident,
            $assign_trait:ident,
            $assign_op:ident
        ]
    ] => {
        impl $trait<$prim> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: $prim) -> Self::Output {
                BigInt::from(self).$op(rhs).into()
                // if let Small(lhs) = &self {
                //     if let Ok(rhs) = u32::try_from(rhs) {
                //         if let (result, false) = lhs.$overflowing_op(rhs) {
                //             return result.into();
                //         }
                //     }
                // }
                // BigInt::from(self).$op(rhs).into()
            }
        }
        impl $assign_trait<$prim> for CBigInt {
            fn $assign_op(&mut self, rhs: $prim) {
                let mut lhs = BigInt::from(std::mem::take(self));
                lhs.$assign_op(rhs);
                *self = lhs.into();
            }
        }
        // ref_op!($trait<$prim> for CBigInt, $op);
    };
    [
        [int $(, $_1:tt)*], [$prim:ident, $to_prim:ident],
        bit_op, [
            $trait:ident,
            $op:ident,
            $assign_trait:ident,
            $assign_op:ident
        ]
    ] => {
    };
    [
        [float $(, $_1:tt)*], [$prim:ident, $to_prim:ident],
        $op:tt, $op_attrs:tt
    ] => {};
}

each_prim_and_op!(
    [int, signed],
    [i8, to_i8],
    arith_op,
    [Add, add, AddAssign, add_assign]
);

with_prims!(each_prim, []);
with_ops!(op_traits, []);

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
