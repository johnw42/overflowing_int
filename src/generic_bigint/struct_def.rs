use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::Neg;

use num_bigint::{BigUint, Sign, ToBigInt};
use num_integer::Roots;
use num_traits::{One, PrimInt, Zero};

use crate::big_number::BigNumberDigits;
use crate::generic_bigint::encoding::{Decoded, EncodedBigNum, InspectEncoding};
use crate::small_num::SmallNumber;
use crate::{BigInteger, BigNumber};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GenericBigInt<'a, E: EncodedBigNum<'a>>(pub(crate) E, PhantomData<&'a ()>);

impl<'a, E: EncodedBigNum<'a>> GenericBigInt<'a, E> {
    pub const ZERO: Self = Self(E::ZERO, PhantomData);

    pub(crate) fn from_encoding(enc: E) -> Self {
        Self(enc, PhantomData)
    }

    pub(crate) fn small_with<'b, E2: EncodedBigNum<'b, Small = E::Small>>(
        &self,
        right: &GenericBigInt<'b, E2>,
    ) -> Option<(E::Small, E::Small)> {
        match (self.small(), right.small()) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }

    pub(crate) fn big_cow(&self) -> Cow<'a, E::Big> {
        self.0.big_cow()
    }

    fn big_ref(&self) -> Option<&'a E::Big> {
        self.0.big_ref()
    }

    /// Returns the magnitude of the `RcBigInt` as a `BigUint`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::RcBigInt;
    /// use num_traits::Zero;
    /// use std::borrow::Borrow;
    /// use num_bigint::BigUint;
    ///
    /// assert_eq!(*RcBigInt::from(1234).magnitude(), BigUint::from(1234u32));
    /// assert_eq!(*RcBigInt::from(-4321).magnitude(), BigUint::from(4321u32));
    /// assert!(RcBigInt::zero().magnitude().is_zero());
    /// ```
    pub fn magnitude(&self) -> BigUint
    where
        E::Big: BigInteger,
    {
        if let Some(small) = self.small() {
            E::Big::from(small).magnitude().clone()
        } else {
            self.big_cow().magnitude().clone()
        }
        // match self.decode_ref() {
        //     Decoded::Small(n) => E::Big::from(n).magnitude().clone(),
        //     Decoded::Big(n) => n.magnitude().clone(),
        // }
    }

    /// Returns the magnitude of the `RcBigInt` as a `BigUint` if the necessary
    /// `BigUint` already exists.
    pub fn try_magnitude(&'a self) -> Option<&'a BigUint>
    where
        E::Big: BigInteger,
    {
        match self.big_ref() {
            Some(n) => Some(n.magnitude()),
            None => None,
        }
    }

    /// Creates and initializes a E::Big.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    pub fn new(sign: Sign, digits: Vec<u32>) -> Self
    where
        E::Big: BigInteger,
    {
        if sign == Sign::NoSign {
            return GenericBigInt::ZERO;
        }
        Self::from_big(E::Big::new(sign, digits))
    }

    /// Creates and initializes a `RcBigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    pub(crate) fn from_biguint(sign: Sign, data: BigUint) -> Self
    where
        E::Big: BigInteger,
    {
        Self::from_big(E::Big::from_biguint(sign, data))
    }

    /// Creates and initializes a `RcBigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    pub fn from_slice(sign: Sign, slice: &[u32]) -> Self
    where
        E::Big: BigInteger,
    {
        Self::from_big(E::Big::from_slice(sign, slice))
    }

    /// Reinitializes a `RcBigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    pub fn assign_from_slice(&mut self, sign: Sign, slice: &[u32])
    where
        E::Big: BigInteger,
    {
        *self = Self::from_slice(sign, slice);
    }

    /// Creates and initializes a `RcBigInt`.
    ///
    /// The bytes are in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{BigInteger, RcBigInt, Sign};
    ///
    /// assert_eq!(RcBigInt::from_bytes_be(Sign::Plus, b"A"),
    ///            RcBigInt::parse_bytes(b"65", 10).unwrap());
    /// assert_eq!(RcBigInt::from_bytes_be(Sign::Plus, b"AA"),
    ///            RcBigInt::parse_bytes(b"16705", 10).unwrap());
    /// assert_eq!(RcBigInt::from_bytes_be(Sign::Plus, b"AB"),
    ///            RcBigInt::parse_bytes(b"16706", 10).unwrap());
    /// assert_eq!(RcBigInt::from_bytes_be(Sign::Plus, b"Hello world!"),
    ///            RcBigInt::parse_bytes(b"22405534230753963835153736737", 10).unwrap());
    /// ```
    pub fn from_bytes_be(sign: Sign, bytes: &[u8]) -> Self
    where
        E::Big: BigInteger,
        E::Small: From<E::SmallUint> + Neg<Output = E::Small>,
    {
        if let Some(from_bytes) = SmallNumber::from_bytes_be(bytes) {
            return Self::from_small(from_bytes);
        }
        Self::from_big(E::Big::from_bytes_be(sign, bytes))
    }

    /// Creates and initializes a `RcBigInt`.
    ///
    /// The bytes are in little-endian byte order.
    pub fn from_bytes_le(sign: Sign, bytes: &[u8]) -> Self
    where
        E::Big: BigInteger,
        E::Small: From<E::SmallUint> + Neg<Output = E::Small>,
    {
        if let Some(from_bytes) = SmallNumber::from_bytes_le(bytes) {
            return Self::from_small(from_bytes);
        }
        Self::from_big(E::Big::from_bytes_le(sign, bytes))
    }

    /// Creates and initializes a `RcBigInt` from an array of bytes in
    /// two's complement binary representation.
    ///
    /// The digits are in big-endian base 2<sup>8</sup>.
    pub fn from_signed_bytes_be(digits: &[u8]) -> Self
    where
        E::Big: BigInteger,
    {
        Self::from_big(E::Big::from_signed_bytes_be(digits))
    }

    /// Creates and initializes a `RcBigInt` from an array of bytes in two's complement.
    ///
    /// The digits are in little-endian base 2<sup>8</sup>.
    pub fn from_signed_bytes_le(digits: &[u8]) -> Self
    where
        E::Big: BigInteger,
    {
        Self::from_big(E::Big::from_signed_bytes_le(digits))
    }

    /// Creates and initializes a `RcBigInt`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{RcBigInt, BigInteger};
    ///
    /// assert_eq!(RcBigInt::parse_bytes(b"1234", 10), Some(RcBigInt::from(1234)));
    /// assert_eq!(RcBigInt::parse_bytes(b"ABCD", 16), Some(RcBigInt::from(0xABCD)));
    /// assert_eq!(RcBigInt::parse_bytes(b"G", 16), None);
    /// ```
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self> {
        E::Big::parse_bytes(buf, radix).map(Self::from_big)
    }

    /// Creates and initializes a `RcBigInt`. Each u8 of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in big-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{BigInteger, RcBigInt, Sign};
    ///
    /// let inbase190 = vec![15, 33, 125, 12, 14];
    /// let a = RcBigInt::from_radix_be(Sign::Minus, &inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_be(190), (Sign::Minus, inbase190));
    /// ```
    pub fn from_radix_be(sign: Sign, buf: &[u8], radix: u32) -> Option<Self>
    where
        E::Big: BigInteger,
    {
        E::Big::from_radix_be(sign, buf, radix).map(Self::from_big)
    }

    /// Creates and initializes a `RcBigInt`. Each u8 of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in little-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{BigInteger, RcBigInt, Sign};
    ///
    /// let inbase190 = vec![14, 12, 125, 33, 15];
    /// let a = RcBigInt::from_radix_be(Sign::Minus, &inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_be(190), (Sign::Minus, inbase190));
    /// ```
    pub fn from_radix_le(sign: Sign, buf: &[u8], radix: u32) -> Option<Self>
    where
        E::Big: BigInteger,
    {
        E::Big::from_radix_le(sign, buf, radix).map(Self::from_big)
    }

    /// Returns the sign and the byte representation of the `RcBigInt` in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{BigInteger, RcBigInt, Sign};
    ///
    /// let i = RcBigInt::from(-1125);
    /// assert_eq!(i.to_bytes_be(), (Sign::Minus, vec![4, 101]));
    /// ```
    pub fn to_bytes_be(&self) -> (Sign, Vec<u8>)
    where
        E::Big: BigInteger,
    {
        self.big_cow().to_bytes_be()
    }

    /// Returns the sign and the byte representation of the `RcBigInt` in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{BigInteger, RcBigInt, Sign};
    ///
    /// let i = RcBigInt::from(-1125);
    /// assert_eq!(i.to_bytes_le(), (Sign::Minus, vec![101, 4]));
    /// ```
    pub fn to_bytes_le(&self) -> (Sign, Vec<u8>)
    where
        E::Big: BigInteger,
    {
        self.big_cow().to_bytes_le()
    }

    /// Returns the sign and the `u32` digits representation of the `RcBigInt` ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::compact_bigint::{RcBigInt, BigInteger, Sign};
    ///
    /// assert_eq!(RcBigInt::from(-1125).to_u32_digits(), (Sign::Minus, vec![1125]));
    /// assert_eq!(RcBigInt::from(4294967295u32).to_u32_digits(), (Sign::Plus, vec![4294967295]));
    /// assert_eq!(RcBigInt::from(4294967296u64).to_u32_digits(), (Sign::Plus, vec![0, 1]));
    /// assert_eq!(RcBigInt::from(-112500000000i64).to_u32_digits(), (Sign::Minus, vec![830850304, 26]));
    /// assert_eq!(RcBigInt::from(112500000000i64).to_u32_digits(), (Sign::Plus, vec![830850304, 26]));
    /// ```
    pub fn to_u32_digits(&self) -> (Sign, Vec<u32>)
    where
        E::Big: BigInteger,
    {
        self.big_cow().to_u32_digits()
    }

    /// Returns the two's-complement byte representation of the `RcBigInt` in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{RcBigInt, BigInteger};
    ///
    /// let i = RcBigInt::from(-1125);
    /// assert_eq!(i.to_signed_bytes_be(), vec![251, 155]);
    /// ```
    pub fn to_signed_bytes_be(&self) -> Vec<u8>
    where
        E::Big: BigInteger,
    {
        self.big_cow().to_signed_bytes_be()
    }

    /// Returns the two's-complement byte representation of the `RcBigInt` in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{RcBigInt, BigInteger};
    ///
    /// let i = RcBigInt::from(-1125);
    /// assert_eq!(i.to_signed_bytes_le(), vec![155, 251]);
    /// ```
    pub fn to_signed_bytes_le(&self) -> Vec<u8>
    where
        E::Big: BigInteger,
    {
        self.big_cow().to_signed_bytes_le()
    }

    /// Returns the integer formatted as a string in the given radix.
    /// `radix` must be in the range `2...36`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{RcBigInt, BigInteger};
    ///
    /// let i = RcBigInt::parse_bytes(b"ff", 16).unwrap();
    /// assert_eq!(i.to_str_radix(16), "ff");
    /// ```
    pub fn to_str_radix(&self, radix: u32) -> String {
        self.big_cow().to_str_radix(radix)
    }

    /// Returns the integer in the requested base in big-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based u8 number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{BigInteger, RcBigInt, Sign};
    ///
    /// assert_eq!(RcBigInt::from(-0xFFFFi64).to_radix_be(159),
    ///            (Sign::Minus, vec![2, 94, 27]));
    /// // 0xFFFF = 65535 = 2*(159^2) + 94*159 + 27
    /// ```
    pub fn to_radix_be(&self, radix: u32) -> (Sign, Vec<u8>)
    where
        E::Big: BigInteger,
    {
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
    /// use compact_bigint::{BigInteger, RcBigInt, Sign};
    ///
    /// assert_eq!(RcBigInt::from(-0xFFFFi64).to_radix_le(159),
    ///            (Sign::Minus, vec![27, 94, 2]));
    /// // 0xFFFF = 65535 = 27 + 94*159 + 2*(159^2)
    /// ```
    pub fn to_radix_le(&self, radix: u32) -> (Sign, Vec<u8>)
    where
        E::Big: BigInteger,
    {
        self.big_cow().to_radix_le(radix)
    }

    /// Returns the sign of the `RcBigInt` as a `Sign`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{BigInteger, RcBigInt, Sign};
    /// use num_traits::Zero;
    ///
    /// assert_eq!(RcBigInt::from(1234).sign(), Sign::Plus);
    /// assert_eq!(RcBigInt::from(-4321).sign(), Sign::Minus);
    /// assert_eq!(RcBigInt::zero().sign(), Sign::NoSign);
    /// ```
    pub fn sign(&self) -> Sign
    where
        E::Big: BigInteger,
    {
        match self.decode_ref() {
            Decoded::Small(n) => match n.cmp(&E::Small::zero()) {
                Ordering::Equal => Sign::NoSign,
                Ordering::Greater => Sign::Plus,
                Ordering::Less => Sign::Minus,
            },
            Decoded::Big(n) => n.sign(),
        }
    }

    /// Convert this `RcBigInt` into its `Sign` and `BigUint` magnitude,
    /// the reverse of `RcBigInt::from_biguint`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{BigInteger, RcBigInt, Sign};
    /// use num_traits::Zero;
    /// use num_bigint::BigUint;
    ///
    /// assert_eq!(RcBigInt::from(1234).into_parts(), (Sign::Plus, BigUint::from(1234u32)));
    /// assert_eq!(RcBigInt::from(-4321).into_parts(), (Sign::Minus, BigUint::from(4321u32)));
    /// assert_eq!(RcBigInt::zero().into_parts(), (Sign::NoSign, BigUint::zero()));
    /// ```
    pub fn into_parts(self) -> (Sign, BigUint)
    where
        E::Big: BigInteger,
    {
        self.to_bigint().unwrap().into_parts()
    }

    /// Returns whether the bit in position `bit` is set, using the two’s complement for negative numbers
    pub fn bit(&self, bit: u64) -> bool {
        match self.decode_ref() {
            Decoded::Small(small) => {
                if bit < E::Small::BITS as u64 {
                    (small >> (bit as u32)) & E::Small::one() == E::Small::one()
                } else {
                    small < E::Small::zero()
                }
            }
            Decoded::Big(big) => big.bit(bit),
        }
    }

    /// Determines the fewest bits necessary to express the `E::Big`,
    /// not including the sign.
    pub fn bits(&self) -> u64 {
        match self.decode_ref() {
            Decoded::Small(n) => {
                if n >= E::Small::zero() {
                    E::Small::BITS as u32 - n.leading_zeros()
                } else {
                    E::Small::BITS as u32 - n.unsigned_abs().leading_zeros()
                }
            }
            .into(),
            Decoded::Big(n) => n.bits(),
        }
    }

    /// Converts this `RcBigInt` into a `BigUint`, if it's not negative.
    pub fn to_biguint(&self) -> Option<BigUint>
    where
        E::Big: BigInteger,
    {
        self.big_cow().to_biguint()
    }

    pub fn checked_add(&self, v: &Self) -> Option<Self> {
        self.big_cow().checked_add(&v.big_cow()).map(Self::from_big)
    }

    pub fn checked_sub(&self, v: &Self) -> Option<Self> {
        self.big_cow().checked_sub(&v.big_cow()).map(Self::from_big)
    }

    pub fn checked_mul(&self, v: &Self) -> Option<Self> {
        self.big_cow().checked_mul(&v.big_cow()).map(Self::from_big)
    }

    pub fn checked_div(&self, v: &Self) -> Option<Self> {
        self.big_cow().checked_div(&v.big_cow()).map(Self::from_big)
    }

    /// Returns `self ^ exponent`.
    pub fn pow(&self, exponent: u32) -> Self {
        if let Some(a) = self.small()
            && let (a, false) = a.overflowing_pow(exponent)
        {
            return Self::from_small(a);
        }
        Self::from_big(self.big_cow().pow(exponent))
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
        Self::from_big(
            self.big_cow()
                .modpow(&exponent.big_cow(), &modulus.big_cow()),
        )
    }

    /// Returns the truncated principal square root of self.
    pub fn sqrt(&self) -> Self {
        match self.decode_ref() {
            Decoded::Small(n) => Self::from_small(n.sqrt()),
            Decoded::Big(n) => Self::from_big(Roots::sqrt(&n)),
        }
    }

    /// Returns the truncated principal cube root of self.
    pub fn cbrt(&self) -> Self {
        match self.decode_ref() {
            Decoded::Small(n) => Self::from_small(n.cbrt()),
            Decoded::Big(n) => Self::from_big(Roots::cbrt(&n)),
        }
    }

    /// Returns the truncated principal nth root of self.
    pub fn nth_root(&self, n: u32) -> Self {
        match self.decode_ref() {
            Decoded::Small(x) => Self::from_small(x.nth_root(n)),
            Decoded::Big(x) => Self::from_big(Roots::nth_root(&x, n)),
        }
    }

    /// Returns the number of least-significant bits that are zero,
    /// or `None` if the entire number is zero.
    pub fn trailing_zeros(&'a self) -> Option<u64> {
        self.big_cow().trailing_zeros()
    }

    pub fn to_u64_digits(&'a self) -> (Sign, Vec<u64>)
    where
        E::Big: BigInteger,
    {
        self.big_cow().to_u64_digits()
    }

    // pub fn iter_u32_digits(&'a self) -> impl BigNumberDigits<'a, u32> {
    //     let cow = self.big_cow();
    //     let digits = (*cow).iter_u32_digits().collect::<Vec<_>>();
    //     digits.into_iter()
    // }

    // pub fn iter_u64_digits(&'a self) -> impl BigNumberDigits<'a, u64> {
    //     let cow = self.big_cow();
    //     let digits = (*cow).iter_u64_digits().collect::<Vec<_>>();
    //     digits.into_iter()
    // }

    pub fn modinv(&self, modulus: &Self) -> Option<Self> {
        self.big_cow()
            .modinv(&modulus.big_cow())
            .map(Self::from_big)
    }

    pub fn set_bit(&mut self, bit: u64, value: bool) {
        self.update_encoding(|encoding| match encoding {
            Decoded::Small(n) if (bit as u32) < E::Small::BITS - 1 => {
                let to_set = E::Small::one() << bit as u32;
                if value {
                    *n = *n | to_set;
                } else {
                    *n = *n & !to_set;
                }
            }
            Decoded::Small(n) => {
                let mut big: <E as EncodedBigNum<'a>>::Big = E::Big::from(*n);
                big.set_bit(bit, value);
                *encoding = Decoded::Big(big);
            }
            Decoded::Big(n) => n.set_bit(bit, value),
        })
    }
}

// impl<'a, E: EncodedBigNum<'a>> InspectEncoding<'a, E::Small, E::Big> for GenericBigInt<'a, E> {
//     fn decode(self) -> Decoded<E::Small, Cow<'a, E::Big>> {
//         self.0.decode()
//     }

//     fn decode_ref(&'a self) -> Decoded<E::Small, Cow<'a, E::Big>> {
//         self.0.decode_ref()
//     }

//     fn small(&self) -> Option<E::Small> {
//         self.0.small()
//     }

//     fn big_ref(&'a self) -> Option<&'a E::Big> {
//         self.0.big_ref()
//     }

//     fn big_cow(&'a self) -> Cow<'a, E::Big> {
//         self.0.big_cow()
//     }

//     fn into_big_cow(self) -> Cow<'a, E::Big> {
//         self.0.into_big_cow()
//     }
// }

impl<'a, E: EncodedBigNum<'a>> EncodedBigNum<'a> for GenericBigInt<'a, E> {
    type Small = E::Small;
    type SmallUint = E::SmallUint;
    type Big = E::Big;
    type Repr = E::Repr;

    const ZERO: Self = Self::ZERO;
    const ONE: Self = Self(E::ONE, PhantomData);

    fn from_decoded(enc: Decoded<E::Small, Cow<'a, E::Big>>) -> Self {
        Self::from_encoding(E::from_decoded(enc))
    }

    fn from_small(s: E::Small) -> Self {
        Self::from_encoding(E::from_small(s))
    }

    fn from_big(b: E::Big) -> Self {
        Self::from_encoding(E::from_big(b))
    }

    fn from_big_cow(b: Cow<'a, E::Big>) -> Self {
        Self::from_encoding(E::from_big_cow(b))
    }

    fn decode(self) -> Decoded<E::Small, Cow<'a, E::Big>> {
        self.0.decode()
    }

    fn decode_ref(&self) -> Decoded<E::Small, Cow<'a, E::Big>> {
        self.0.decode_ref()
    }

    fn small(&self) -> Option<E::Small> {
        self.0.small()
    }

    fn big_ref(&self) -> Option<&'a E::Big> {
        self.0.big_ref()
    }

    fn big_cow(&self) -> Cow<'a, E::Big> {
        self.0.big_cow()
    }

    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<E::Small, E::Big>)) {
        self.0.update_encoding(f);
    }
}

impl<'a, E: EncodedBigNum<'a>> Default for GenericBigInt<'a, E> {
    fn default() -> Self {
        GenericBigInt::ZERO
    }
}

impl<'a, E: EncodedBigNum<'a>> Display for GenericBigInt<'a, E> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.decode_ref() {
            Decoded::Small(n) => Display::fmt(&n, f),
            Decoded::Big(n) => Display::fmt(&n, f),
        }
    }
}

impl<'a, E: EncodedBigNum<'a>> Debug for GenericBigInt<'a, E> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.decode_ref() {
            Decoded::Small(n) => Debug::fmt(&n, f),
            Decoded::Big(n) => Debug::fmt(&n, f),
        }
    }
}
