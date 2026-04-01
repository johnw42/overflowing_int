use std::borrow::Cow;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};

use num_bigint::{BigInt, BigUint, Sign};
use num_integer::Roots;
use num_traits::ConstZero;

use crate::accum::*;
use crate::big_integer::BigInteger;
use crate::encoding::{Encoded, Encoding};
use crate::{SMALL_BITS, SmallInt, SmallUint};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CBigInt(pub(crate) Encoded<SmallInt, BigInt>);

impl CBigInt {
    pub(crate) fn to_small(&self) -> Option<SmallInt> {
        match self.0.borrow_encoding() {
            Encoding::Small(n) => Some(*n),
            Encoding::Big(_) => None,
        }
    }

    pub(crate) fn to_small_with(&self, other: &CBigInt) -> Option<(SmallInt, SmallInt)> {
        match (self.to_small(), other.to_small()) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }

    fn try_apply_sign(sign: Sign, magnitude: SmallUint) -> Option<Self> {
        SmallInt::try_from(magnitude)
            .ok()
            .map(|signed_magnitude| match sign {
                Sign::Plus => signed_magnitude,
                Sign::Minus => -signed_magnitude,
                Sign::NoSign => 0,
            })
            .map(Self::from)
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
        match self.0.borrow_encoding() {
            Encoding::Small(n) => Cow::Owned(BigInt::from(*n).into_parts().1),
            Encoding::Big(n) => Cow::Borrowed(n.magnitude()),
        }
    }

    /// Returns the magnitude of the `CBigInt` as a `BigUint` if the necessary
    /// `BigUint` already exists.
    pub fn try_magnitude(&self) -> Option<&BigUint> {
        match self.0.borrow_encoding() {
            Encoding::Small(_) => None,
            Encoding::Big(n) => Some(n.magnitude()),
        }
    }

    /// Converts this `CBigInt` into a `BigInt`.
    pub(crate) fn to_bigint_cow(&self) -> Cow<'_, BigInt> {
        self.into()
    }
}

impl BigInteger for CBigInt {
    /// Creates and initializes a BigInt.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    fn new(sign: Sign, digits: Vec<u32>) -> CBigInt {
        if sign == Sign::NoSign {
            return CBigInt::ZERO;
        }
        CBigInt(Encoded::from_big(BigInt::new(sign, digits)))
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
        BigInt::from_slice(sign, slice).into()
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
    /// use compact_bigint::{BigInteger, CBigInt, Sign};
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
        BigInt::from_signed_bytes_be(digits).into()
    }

    /// Creates and initializes a `CBigInt` from an array of bytes in two's complement.
    ///
    /// The digits are in little-endian base 2<sup>8</sup>.
    fn from_signed_bytes_le(digits: &[u8]) -> CBigInt {
        BigInt::from_signed_bytes_le(digits).into()
    }

    /// Creates and initializes a `CBigInt`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CBigInt, BigInteger};
    ///
    /// assert_eq!(CBigInt::parse_bytes(b"1234", 10), Some(CBigInt::from(1234)));
    /// assert_eq!(CBigInt::parse_bytes(b"ABCD", 16), Some(CBigInt::from(0xABCD)));
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
    /// use compact_bigint::{BigInteger, CBigInt, Sign};
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
    /// use compact_bigint::{BigInteger, CBigInt, Sign};
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
    /// use compact_bigint::{BigInteger, CBigInt, Sign};
    ///
    /// let i = CBigInt::from(-1125);
    /// assert_eq!(i.to_bytes_be(), (Sign::Minus, vec![4, 101]));
    /// ```
    fn to_bytes_be(&self) -> (Sign, Vec<u8>) {
        BigInt::from(self).to_bytes_be()
    }

    /// Returns the sign and the byte representation of the `CBigInt` in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{BigInteger, CBigInt, Sign};
    ///
    /// let i = CBigInt::from(-1125);
    /// assert_eq!(i.to_bytes_le(), (Sign::Minus, vec![101, 4]));
    /// ```
    fn to_bytes_le(&self) -> (Sign, Vec<u8>) {
        BigInt::from(self).to_bytes_le()
    }

    /// Returns the sign and the `u32` digits representation of the `CBigInt` ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::compact_bigint::{CBigInt, BigInteger, Sign};
    ///
    /// assert_eq!(CBigInt::from(-1125).to_u32_digits(), (Sign::Minus, vec![1125]));
    /// assert_eq!(CBigInt::from(4294967295u32).to_u32_digits(), (Sign::Plus, vec![4294967295]));
    /// assert_eq!(CBigInt::from(4294967296u64).to_u32_digits(), (Sign::Plus, vec![0, 1]));
    /// assert_eq!(CBigInt::from(-112500000000i64).to_u32_digits(), (Sign::Minus, vec![830850304, 26]));
    /// assert_eq!(CBigInt::from(112500000000i64).to_u32_digits(), (Sign::Plus, vec![830850304, 26]));
    /// ```
    fn to_u32_digits(&self) -> (Sign, Vec<u32>) {
        BigInt::from(self).to_u32_digits()
    }

    /// Returns the two's-complement byte representation of the `CBigInt` in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CBigInt, BigInteger};
    ///
    /// let i = CBigInt::from(-1125);
    /// assert_eq!(i.to_signed_bytes_be(), vec![251, 155]);
    /// ```
    fn to_signed_bytes_be(&self) -> Vec<u8> {
        BigInt::from(self).to_signed_bytes_be()
    }

    /// Returns the two's-complement byte representation of the `CBigInt` in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CBigInt, BigInteger};
    ///
    /// let i = CBigInt::from(-1125);
    /// assert_eq!(i.to_signed_bytes_le(), vec![155, 251]);
    /// ```
    fn to_signed_bytes_le(&self) -> Vec<u8> {
        BigInt::from(self).to_signed_bytes_le()
    }

    /// Returns the integer formatted as a string in the given radix.
    /// `radix` must be in the range `2...36`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CBigInt, BigInteger};
    ///
    /// let i = CBigInt::parse_bytes(b"ff", 16).unwrap();
    /// assert_eq!(i.to_str_radix(16), "ff");
    /// ```
    fn to_str_radix(&self, radix: u32) -> String {
        self.to_bigint_cow().to_str_radix(radix)
    }

    /// Returns the integer in the requested base in big-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based u8 number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{BigInteger, CBigInt, Sign};
    ///
    /// assert_eq!(CBigInt::from(-0xFFFFi64).to_radix_be(159),
    ///            (Sign::Minus, vec![2, 94, 27]));
    /// // 0xFFFF = 65535 = 2*(159^2) + 94*159 + 27
    /// ```
    fn to_radix_be(&self, radix: u32) -> (Sign, Vec<u8>) {
        self.to_bigint_cow().to_radix_be(radix)
    }

    /// Returns the integer in the requested base in little-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based u8 number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{BigInteger, CBigInt, Sign};
    ///
    /// assert_eq!(CBigInt::from(-0xFFFFi64).to_radix_le(159),
    ///            (Sign::Minus, vec![27, 94, 2]));
    /// // 0xFFFF = 65535 = 27 + 94*159 + 2*(159^2)
    /// ```
    fn to_radix_le(&self, radix: u32) -> (Sign, Vec<u8>) {
        self.to_bigint_cow().to_radix_le(radix)
    }

    /// Returns the sign of the `CBigInt` as a `Sign`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{BigInteger, CBigInt, Sign};
    /// use num_traits::Zero;
    ///
    /// assert_eq!(CBigInt::from(1234).sign(), Sign::Plus);
    /// assert_eq!(CBigInt::from(-4321).sign(), Sign::Minus);
    /// assert_eq!(CBigInt::zero().sign(), Sign::NoSign);
    /// ```
    fn sign(&self) -> Sign {
        match self.0.borrow_encoding() {
            Encoding::Small(n) => match n.cmp(&0) {
                Ordering::Equal => Sign::NoSign,
                Ordering::Greater => Sign::Plus,
                Ordering::Less => Sign::Minus,
            },
            Encoding::Big(n) => n.sign(),
        }
    }

    /// Convert this `CBigInt` into its `Sign` and `BigUint` magnitude,
    /// the reverse of `CBigInt::from_biguint`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{BigInteger, CBigInt, Sign};
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
        match self.0.borrow_encoding() {
            Encoding::Small(small) => {
                if bit < SMALL_BITS as u64 {
                    (*small >> (bit as u32)) & 1 == 1
                } else {
                    *small < 0
                }
            }
            Encoding::Big(big) => big.bit(bit),
        }
    }

    /// Determines the fewest bits necessary to express the `BigInt`,
    /// not including the sign.
    fn bits(&self) -> u64 {
        match self.0.borrow_encoding() {
            Encoding::Small(n) => {
                if *n >= 0 {
                    SMALL_BITS as u32 - n.leading_zeros()
                } else {
                    SMALL_BITS as u32 - n.unsigned_abs().leading_zeros()
                }
            }
            .into(),
            Encoding::Big(n) => n.bits(),
        }
    }

    /// Converts this `CBigInt` into a `BigUint`, if it's not negative.
    fn to_biguint(&self) -> Option<BigUint> {
        self.to_bigint_cow().to_biguint()
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
        if let Some(a) = self.to_small()
            && let (a, false) = a.overflowing_pow(exponent)
        {
            return a.into();
        }
        self.to_bigint_cow().pow(exponent).into()
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
        self.to_bigint_cow()
            .modpow(&exponent.to_bigint_cow(), &modulus.to_bigint_cow())
            .into()
    }

    /// Returns the truncated principal square root of self.
    fn sqrt(&self) -> Self {
        match self.0.borrow_encoding() {
            Encoding::Small(n) => Self::from(n.sqrt()),
            Encoding::Big(n) => Self::from(n.sqrt()),
        }
    }

    /// Returns the truncated principal cube root of self.
    fn cbrt(&self) -> Self {
        match self.0.borrow_encoding() {
            Encoding::Small(n) => Self::from(n.cbrt()),
            Encoding::Big(n) => Self::from(n.cbrt()),
        }
    }

    /// Returns the truncated principal nth root of self.
    fn nth_root(&self, n: u32) -> Self {
        match self.0.borrow_encoding() {
            Encoding::Small(x) => Self::from(x.nth_root(n)),
            Encoding::Big(x) => Self::from(x.nth_root(n)),
        }
    }

    /// Returns the number of least-significant bits that are zero,
    /// or `None` if the entire number is zero.
    fn trailing_zeros(&self) -> Option<u64> {
        BigInt::from(self).trailing_zeros()
    }

    fn to_u64_digits(&self) -> (Sign, Vec<u64>) {
        self.to_bigint_cow().to_u64_digits()
    }

    fn iter_u32_digits(
        &self,
    ) -> impl DoubleEndedIterator<Item = u32> + ExactSizeIterator<Item = u32> + '_ {
        self.to_bigint_cow()
            .iter_u32_digits()
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn iter_u64_digits(
        &self,
    ) -> impl DoubleEndedIterator<Item = u64> + ExactSizeIterator<Item = u64> + '_ {
        self.to_bigint_cow()
            .iter_u64_digits()
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn modinv(&self, modulus: &Self) -> Option<Self> {
        BigInt::from(self)
            .modinv(&modulus.to_bigint_cow())
            .map(Self::from)
    }

    fn set_bit(&mut self, bit: u64, value: bool) {
        self.0.update_encoding(|encoding| match encoding {
            Encoding::Small(n) if (bit as usize) < SMALL_BITS - 1 => {
                let mask = 1 << bit;
                if value {
                    *n |= mask;
                } else {
                    *n &= !mask;
                }
            }
            Encoding::Small(n) => {
                let mut big = BigInt::from(*n);
                big.set_bit(bit, value);
                *encoding = Encoding::Big(big);
            }
            Encoding::Big(n) => n.set_bit(bit, value),
        });
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
        CBigInt::ZERO
    }
}

impl Display for CBigInt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.0.borrow_encoding() {
            Encoding::Small(n) => Display::fmt(n, f),
            Encoding::Big(n) => Display::fmt(n, f),
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

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    use super::*;

    /// An arbitrary `Sign` for testing purposes.  Usually not `NoSign` to avoid generating too many zeroes.
    #[derive(Clone, Debug)]
    struct ArbSign(Sign);

    impl Arbitrary for ArbSign {
        fn arbitrary(g: &mut Gen) -> Self {
            ArbSign(match u8::arbitrary(g).cmp(&0) {
                Ordering::Less => Sign::Minus,
                Ordering::Equal => Sign::NoSign,
                Ordering::Greater => Sign::Plus,
            })
        }
    }

    #[quickcheck]
    fn prop_new_matches_bigint(sign: ArbSign, digits: Vec<u32>) -> bool {
        BigInt::from(CBigInt::new(sign.0, digits.clone())) == BigInt::new(sign.0, digits)
    }

    #[quickcheck]
    fn prop_from_slice_matches_bigint(sign: ArbSign, digits: Vec<u32>) -> bool {
        BigInt::from(CBigInt::from_slice(sign.0, &digits)) == BigInt::from_slice(sign.0, &digits)
    }

    #[quickcheck]
    fn prop_from_bytes_be_matches_bigint(sign: ArbSign, bytes: Vec<u8>) -> bool {
        BigInt::from(CBigInt::from_bytes_be(sign.0, &bytes))
            == BigInt::from_bytes_be(sign.0, &bytes)
    }

    #[quickcheck]
    fn prop_from_bytes_le_matches_bigint(sign: ArbSign, bytes: Vec<u8>) -> bool {
        BigInt::from(CBigInt::from_bytes_le(sign.0, &bytes))
            == BigInt::from_bytes_le(sign.0, &bytes)
    }

    #[quickcheck]
    fn prop_from_signed_bytes_be_matches_bigint(bytes: Vec<u8>) {
        assert_eq!(
            BigInt::from(CBigInt::from_signed_bytes_be(&bytes)),
            BigInt::from_signed_bytes_be(&bytes)
        );
    }

    #[quickcheck]
    fn prop_from_signed_bytes_le_matches_bigint(bytes: Vec<u8>) -> bool {
        BigInt::from(CBigInt::from_signed_bytes_le(&bytes)) == BigInt::from_signed_bytes_le(&bytes)
    }

    #[quickcheck]
    fn prop_to_bytes_be_matches_bigint(n: CBigInt) -> bool {
        BigInt::from(&n).to_bytes_be() == n.to_bytes_be()
    }

    #[quickcheck]
    fn prop_to_bytes_le_matches_bigint(n: CBigInt) -> bool {
        BigInt::from(&n).to_bytes_le() == n.to_bytes_le()
    }

    #[quickcheck]
    fn prop_to_u32_digits_matches_bigint(n: CBigInt) -> bool {
        BigInt::from(&n).to_u32_digits() == n.to_u32_digits()
    }

    #[quickcheck]
    fn prop_to_signed_bytes_be_matches_bigint(n: CBigInt) -> bool {
        BigInt::from(&n).to_signed_bytes_be() == n.to_signed_bytes_be()
    }

    #[quickcheck]
    fn prop_to_signed_bytes_le_matches_bigint(n: CBigInt) -> bool {
        BigInt::from(&n).to_signed_bytes_le() == n.to_signed_bytes_le()
    }

    #[quickcheck]
    fn prop_sign_matches_bigint(n: CBigInt) -> bool {
        BigInt::from(&n).sign() == n.sign()
    }

    #[quickcheck]
    fn prop_bit_matches_bigint(n: CBigInt, bit: u64) -> () {
        let bit = bit % 1024;
        assert_eq!(BigInt::from(n.clone()).bit(bit), n.bit(bit));
    }

    #[quickcheck]
    fn prop_bits_matches_bigint(n: CBigInt) -> bool {
        BigInt::from(&n).bits() == n.bits()
    }

    #[quickcheck]
    fn prop_pow_matches_bigint(n: CBigInt, k: u32) -> bool {
        let k = k % 16;
        BigInt::from(&n).pow(k) == n.pow(k).into()
    }

    #[quickcheck]
    fn prop_trailing_zeros_matches_bigint(n: CBigInt) -> bool {
        BigInt::from(&n).trailing_zeros() == n.trailing_zeros()
    }

    #[quickcheck]
    fn prop_set_bit_matches_bigint(mut n1: CBigInt, bit: u64, value: bool) -> bool {
        let bit = bit % (n1.bits() + 16);
        let mut n2 = BigInt::from(&n1);
        n1.set_bit(bit, value);
        n2.set_bit(bit, value);
        BigInt::from(n1) == n2
    }
}
