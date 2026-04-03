use num_bigint::RandomBits;
use rand::prelude::Distribution;

use crate::BigNumber;

pub trait BigNatural: BigNumber
where
    RandomBits: Distribution<Self>,
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

    /// Returns the integer formatted as a string in the given radix.
    /// `radix` must be in the range `2...36`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigUint;
    ///
    /// let i = BigUint::parse_bytes(b"ff", 16).unwrap();
    /// assert_eq!(i.to_str_radix(16), "ff");
    /// ```
    fn to_str_radix(&self, radix: u32) -> String;

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

// impl BigNatural for BigUint {
//     const ZERO: Self = Self::ZERO;

//     fn new(digits: Vec<u32>) -> Self {
//         BigUint::new(digits)
//     }

//     fn from_slice(slice: &[u32]) -> Self {
//         Self::from_slice(slice)
//     }

//     fn assign_from_slice(&mut self, slice: &[u32]) {
//         self.assign_from_slice(slice)
//     }

//     fn from_bytes_be(bytes: &[u8]) -> Self {
//         Self::from_bytes_be(bytes)
//     }

//     fn from_bytes_le(bytes: &[u8]) -> Self {
//         Self::from_bytes_le(bytes)
//     }

//     fn from_radix_be(buf: &[u8], radix: u32) -> Option<Self> {
//         Self::from_radix_be(buf, radix)
//     }

//     fn from_radix_le(buf: &[u8], radix: u32) -> Option<Self> {
//         Self::from_radix_le(buf, radix)
//     }

//     fn to_bytes_be(&self) -> Vec<u8> {
//         self.to_bytes_be()
//     }

//     fn to_bytes_le(&self) -> Vec<u8> {
//         self.to_bytes_le()
//     }

//     fn to_u32_digits(&self) -> Vec<u32> {
//         self.to_u32_digits()
//     }

//     fn to_u64_digits(&self) -> Vec<u64> {
//         self.to_u64_digits()
//     }

//     fn to_str_radix(&self, radix: u32) -> String {
//         self.to_str_radix(radix)
//     }

//     fn to_radix_be(&self, radix: u32) -> Vec<u8> {
//         self.to_radix_be(radix)
//     }

//     fn to_radix_le(&self, radix: u32) -> Vec<u8> {
//         self.to_radix_le(radix)
//     }

//     fn trailing_ones(&self) -> u64 {
//         self.trailing_ones()
//     }

//     fn count_ones(&self) -> u64 {
//         self.count_ones()
//     }
// }
