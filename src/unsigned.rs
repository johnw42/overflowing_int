use crate::big_number::BigNumberDigits;
use crate::encoding::{Decode, Decoded, Encode, Encoding, EncodingMut};
use crate::small_num::SmallNumber;
use num_bigint::BigUint;
use num_traits::{Pow, PrimInt as _};
use std::borrow::Cow;
use std::marker::PhantomData;

/// An unsigned big integer type that can be used with any encoding that implements `Encoding` with `Big = BigUint`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Uint<'enc, E>(pub(crate) E, PhantomData<&'enc ()>)
where
    E: Encoding<'enc>;

impl<'enc, E> Uint<'enc, E>
where
    E: Encoding<'enc, Big = BigUint>,
{
    const fn from_encoding(encoding: E) -> Self {
        Self(encoding, PhantomData)
    }

    /// Converts this big integer to a version with a static lifetime.  This may require cloning a `BigInt`.
    pub fn into_owned(self) -> <Self as Encoding<'enc>>::Owned {
        Uint::from_encoding(self.0.into_owned())
    }

    /// Converts this big integer to into one that borrows from this one's data,
    /// if possible.  If the encoding does not support borrowing, this will
    /// simply clone self.
    pub fn borrow<'a>(&'a self) -> Uint<'a, E::Borrowed<'a>> {
        Uint::from_encoding(self.0.borrow())
    }

    /// A constant bigint with value 0, useful for static initialization.
    pub const ZERO: Self = Self::from_encoding(E::ZERO);

    /// Creates and initializes a [`Uint`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn new(digits: Vec<u32>) -> Uint<'enc, E> {
        Self::from_encoding(E::from_big(E::Big::new(digits)))
    }

    /// Creates and initializes a [`Uint`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn from_slice(slice: &[u32]) -> Uint<'enc, E> {
        Self::from_encoding(E::from_big(E::Big::from_slice(slice)))
    }

    /// Assign a value to a [`Uint`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn assign_from_slice(&mut self, slice: &[u32]) {
        *self = Self::from_slice(slice);
    }

    /// Creates and initializes a [`Uint`].
    ///
    /// The bytes are in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::EnumBigUint;
    ///
    /// assert_eq!(EnumBigUint::from_bytes_be(b"A"),
    ///            EnumBigUint::parse_bytes(b"65", 10).unwrap());
    /// assert_eq!(EnumBigUint::from_bytes_be(b"AA"),
    ///            EnumBigUint::parse_bytes(b"16705", 10).unwrap());
    /// assert_eq!(EnumBigUint::from_bytes_be(b"AB"),
    ///            EnumBigUint::parse_bytes(b"16706", 10).unwrap());
    /// assert_eq!(EnumBigUint::from_bytes_be(b"Hello world!"),
    ///            EnumBigUint::parse_bytes(b"22405534230753963835153736737", 10).unwrap());
    /// ```
    #[inline]
    pub fn from_bytes_be(bytes: &[u8]) -> Uint<'enc, E> {
        Self::from_encoding(
            if let Some(from_bytes) = SmallNumber::from_bytes_be(bytes) {
                E::from_small(from_bytes)
            } else {
                E::from_big(E::Big::from_bytes_be(bytes))
            },
        )
    }

    /// Creates and initializes a [`Uint`].
    ///
    /// The bytes are in little-endian byte order.
    #[inline]
    pub fn from_bytes_le(bytes: &[u8]) -> Uint<'enc, E> {
        Self::from_encoding(E::from_big(E::Big::from_bytes_le(bytes)))
    }

    /// Creates and initializes a [`Uint`]. The input slice must contain
    /// ascii/utf8 characters in [0-9a-zA-Z].
    /// `radix` must be in the range `2...36`.
    ///
    /// The function `from_str_radix` from the `Num` trait provides the same logic
    /// for `&str` buffers.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::EnumBigUint;
    ///
    /// assert_eq!(EnumBigUint::parse_bytes(b"1234", 10), Some(EnumBigUint::from(1234u32)));
    /// assert_eq!(EnumBigUint::parse_bytes(b"ABCD", 16), Some(EnumBigUint::from(0xABCDu32)));
    /// assert_eq!(EnumBigUint::parse_bytes(b"G", 16), None);
    /// ```
    #[inline]
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self> {
        E::parse_bytes(buf, radix).map(Self::from_encoding)
    }

    /// Creates and initializes a [`Uint`]. Each `u8` of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in big-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::EnumBigUint;
    ///
    /// let inbase190 = &[15, 33, 125, 12, 14];
    /// let a = EnumBigUint::from_radix_be(inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_be(190), inbase190);
    /// ```
    pub fn from_radix_be(buf: &[u8], radix: u32) -> Option<Self> {
        E::Big::from_radix_be(buf, radix)
            .map(E::from_big)
            .map(Self::from_encoding)
    }

    /// Creates and initializes a [`Uint`]. Each `u8` of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in little-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::EnumBigUint;
    ///
    /// let inbase190 = &[14, 12, 125, 33, 15];
    /// let a = EnumBigUint::from_radix_le(inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_le(190), inbase190);
    /// ```
    pub fn from_radix_le(buf: &[u8], radix: u32) -> Option<Uint<'enc, E>> {
        E::Big::from_radix_le(buf, radix)
            .map(E::from_big)
            .map(Self::from_encoding)
    }

    /// Returns the byte representation of the [`Uint`] in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::EnumBigUint;
    ///
    /// let i = EnumBigUint::parse_bytes(b"1125", 10).unwrap();
    /// assert_eq!(i.to_bytes_be(), vec![4, 101]);
    /// ```
    #[inline]
    pub fn to_bytes_be(&self) -> Vec<u8> {
        self.big_cow().to_bytes_be()
    }

    /// Returns the byte representation of the [`Uint`] in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::EnumBigUint;
    ///
    /// let i = EnumBigUint::parse_bytes(b"1125", 10).unwrap();
    /// assert_eq!(i.to_bytes_le(), vec![101, 4]);
    /// ```
    #[inline]
    pub fn to_bytes_le(&self) -> Vec<u8> {
        self.big_cow().to_bytes_le()
    }

    /// Returns the `u32` digits representation of the [`Uint`] ordered least significant digit
    /// first.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::EnumBigUint;
    ///
    /// assert_eq!(EnumBigUint::from(1125u32).to_u32_digits(), vec![1125]);
    /// assert_eq!(EnumBigUint::from(4294967295u32).to_u32_digits(), vec![4294967295]);
    /// assert_eq!(EnumBigUint::from(4294967296u64).to_u32_digits(), vec![0, 1]);
    /// assert_eq!(EnumBigUint::from(112500000000u64).to_u32_digits(), vec![830850304, 26]);
    /// ```
    #[inline]
    pub fn to_u32_digits(&self) -> Vec<u32> {
        self.big_cow().to_u32_digits()
    }

    /// Returns the `u64` digits representation of the [`Uint`] ordered least significant digit
    /// first.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::EnumBigUint;
    ///
    /// assert_eq!(EnumBigUint::from(1125u32).to_u64_digits(), vec![1125]);
    /// assert_eq!(EnumBigUint::from(4294967295u32).to_u64_digits(), vec![4294967295]);
    /// assert_eq!(EnumBigUint::from(4294967296u64).to_u64_digits(), vec![4294967296]);
    /// assert_eq!(EnumBigUint::from(112500000000u64).to_u64_digits(), vec![112500000000]);
    /// assert_eq!(EnumBigUint::from(1u128 << 64).to_u64_digits(), vec![0, 1]);
    /// ```
    #[inline]
    pub fn to_u64_digits(&self) -> Vec<u64> {
        self.big_cow().to_u64_digits()
    }

    /// Returns an iterator of `u32` digits representation of the [`Uint`] ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::EnumBigUint;
    ///
    /// assert_eq!(EnumBigUint::from(1125u32).iter_u32_digits().collect::<Vec<u32>>(), vec![1125]);
    /// assert_eq!(EnumBigUint::from(4294967295u32).iter_u32_digits().collect::<Vec<u32>>(), vec![4294967295]);
    /// assert_eq!(EnumBigUint::from(4294967296u64).iter_u32_digits().collect::<Vec<u32>>(), vec![0, 1]);
    /// assert_eq!(EnumBigUint::from(112500000000u64).iter_u32_digits().collect::<Vec<u32>>(), vec![830850304, 26]);
    /// ```
    #[inline]
    pub fn iter_u32_digits(&self) -> impl BigNumberDigits<'_, u32> {
        self.0.iter_u32_digits()
    }

    /// Returns an iterator of `u64` digits representation of the [`Uint`] ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::EnumBigUint;
    ///
    /// assert_eq!(EnumBigUint::from(1125u32).iter_u64_digits().collect::<Vec<u64>>(), vec![1125]);
    /// assert_eq!(EnumBigUint::from(4294967295u32).iter_u64_digits().collect::<Vec<u64>>(), vec![4294967295]);
    /// assert_eq!(EnumBigUint::from(4294967296u64).iter_u64_digits().collect::<Vec<u64>>(), vec![4294967296]);
    /// assert_eq!(EnumBigUint::from(112500000000u64).iter_u64_digits().collect::<Vec<u64>>(), vec![112500000000]);
    /// assert_eq!(EnumBigUint::from(1u128 << 64).iter_u64_digits().collect::<Vec<u64>>(), vec![0, 1]);
    /// ```
    #[inline]
    pub fn iter_u64_digits(&self) -> impl BigNumberDigits<'_, u64> {
        self.0.iter_u64_digits()
    }

    /// Returns the integer formatted as a string in the given radix.
    /// `radix` must be in the range `2...36`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::EnumBigUint;
    ///
    /// let i = EnumBigUint::parse_bytes(b"ff", 16).unwrap();
    /// assert_eq!(i.to_str_radix(16), "ff");
    /// ```
    #[inline]
    pub fn to_str_radix(&self, radix: u32) -> String {
        self.0.to_str_radix(radix)
    }

    /// Returns the integer in the requested base in big-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based `u8` number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::EnumBigUint;
    ///
    /// assert_eq!(EnumBigUint::from(0xFFFFu64).to_radix_be(159),
    ///            vec![2, 94, 27]);
    /// // 0xFFFF = 65535 = 2*(159^2) + 94*159 + 27
    /// ```
    #[inline]
    pub fn to_radix_be(&self, radix: u32) -> Vec<u8> {
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
    /// use compact_bigint::EnumBigUint;
    ///
    /// assert_eq!(EnumBigUint::from(0xFFFFu64).to_radix_le(159),
    ///            vec![27, 94, 2]);
    /// // 0xFFFF = 65535 = 27 + 94*159 + 2*(159^2)
    /// ```
    #[inline]
    pub fn to_radix_le(&self, radix: u32) -> Vec<u8> {
        self.big_cow().to_radix_le(radix)
    }

    /// Determines the fewest bits necessary to express the [`Uint`].
    #[inline]
    pub fn bits(&self) -> u64 {
        self.0.bits()
    }

    /// Returns `self ^ exponent`.
    pub fn pow(&self, exponent: u32) -> Self {
        Pow::pow(self, exponent)
    }

    /// Returns `(self ^ exponent) % modulus`.
    ///
    /// Panics if the modulus is zero.
    pub fn modpow(&self, exponent: &Self, modulus: &Self) -> Self {
        Self::from_encoding(self.0.modpow(&exponent.0, &modulus.0))
    }

    /// Returns the modular multiplicative inverse if it exists, otherwise `None`.
    ///
    /// This solves for `x` in the interval `[0, modulus)` such that `self * x ≡ 1 (mod modulus)`.
    /// The solution exists if and only if `gcd(self, modulus) == 1`.
    ///
    /// ```
    /// use compact_bigint::EnumBigUint;
    /// use num_traits::{One, Zero};
    ///
    /// let m = EnumBigUint::from(383_u32);
    ///
    /// // Trivial cases
    /// assert_eq!(EnumBigUint::zero().modinv(&m), None);
    /// assert_eq!(EnumBigUint::one().modinv(&m), Some(EnumBigUint::one()));
    /// let neg1 = &m - 1u32;
    /// assert_eq!(neg1.modinv(&m), Some(neg1));
    ///
    /// let a = EnumBigUint::from(271_u32);
    /// let x = a.modinv(&m).unwrap();
    /// assert_eq!(x, EnumBigUint::from(106_u32));
    /// assert_eq!(x.modinv(&m).unwrap(), a);
    /// assert!((a * x % m).is_one());
    /// ```
    pub fn modinv(&self, modulus: &Self) -> Option<Self> {
        self.0.modinv(&modulus.0).map(Self::from_encoding)
    }

    /// Returns the truncated principal square root of `self` --
    /// see [Roots::sqrt](https://docs.rs/num-integer/0.1/num_integer/trait.Roots.html#method.sqrt)
    pub fn sqrt(&self) -> Self {
        Self::from_encoding(self.0.sqrt())
    }

    /// Returns the truncated principal cube root of `self` --
    /// see [Roots::cbrt](https://docs.rs/num-integer/0.1/num_integer/trait.Roots.html#method.cbrt).
    pub fn cbrt(&self) -> Self {
        Self::from_encoding(self.0.cbrt())
    }

    /// Returns the truncated principal `n`th root of `self` --
    /// see [Roots::nth_root](https://docs.rs/num-integer/0.1/num_integer/trait.Roots.html#tymethod.nth_root).
    pub fn nth_root(&self, n: u32) -> Self {
        Self::from_encoding(self.0.nth_root(n))
    }

    /// Returns the number of least-significant bits that are zero,
    /// or `None` if the entire number is zero.
    pub fn trailing_zeros(&self) -> Option<u64> {
        self.0.trailing_zeros()
    }

    /// Returns the number of least-significant bits that are ones.
    pub fn trailing_ones(&self) -> u64 {
        match self.decode() {
            Decoded::Small(s) => s.trailing_ones() as u64,
            Decoded::Big(b) => b.as_ref().trailing_ones(),
        }
    }

    /// Returns the number of one bits.
    pub fn count_ones(&self) -> u64 {
        match self.decode() {
            Decoded::Small(s) => s.count_ones() as u64,
            Decoded::Big(b) => b.as_ref().count_ones(),
        }
    }

    /// Returns whether the bit in the given position is set
    pub fn bit(&self, bit: u64) -> bool {
        self.0.bit(bit)
    }

    /// Sets or clears the bit in the given position
    ///
    /// Note that setting a bit greater than the current bit length, a reallocation may be needed
    /// to store the new digits
    pub fn set_bit(&mut self, bit: u64, value: bool)
    where
        E: EncodingMut<'enc>,
    {
        self.0.set_bit(bit, value)
    }
}

impl<'enc, E> Default for Uint<'enc, E>
where
    E: Encoding<'enc, Big = BigUint>,
{
    fn default() -> Self {
        Self::ZERO
    }
}

impl<'enc, E> Decode<'enc, E::Small> for Uint<'enc, E>
where
    E: Encoding<'enc>,
{
    fn into_decoded(self) -> Decoded<E::Small, Cow<'enc, E::Big>> {
        self.0.into_decoded()
    }

    fn decode<'a>(&'a self) -> Decoded<E::Small, Cow<'a, <E::Small as SmallNumber>::Big>> {
        self.0.decode()
    }
}

impl<'enc, E> Decode<'enc, E::Small> for &Uint<'enc, E>
where
    E: Encoding<'enc>,
{
    fn into_decoded(self) -> Decoded<E::Small, Cow<'enc, E::Big>> {
        self.0.clone().into_decoded()
    }

    fn decode<'a>(&'a self) -> Decoded<E::Small, Cow<'a, <E::Small as SmallNumber>::Big>> {
        self.0.decode()
    }
}

impl<'enc, E> Encode<'enc, E::Small> for Uint<'enc, E>
where
    E: Encoding<'enc, Big = BigUint>,
{
    fn from_small(s: E::Small) -> Self {
        Self::from_encoding(E::from_small(s))
    }

    fn from_big(b: E::Big) -> Self {
        Self::from_encoding(E::from_big(b))
    }

    fn from_big_ref(b: &'enc E::Big) -> Self {
        Self::from_encoding(E::from_big_ref(b))
    }
}

impl<'enc, E> Encoding<'enc> for Uint<'enc, E>
where
    E: Encoding<'enc, Big = BigUint>,
{
    type Small = E::Small;
    type Big = E::Big;
    type Unsigned = Uint<'enc, E::Unsigned>;
    type Owned = Uint<'static, E::Owned>;
    type Borrowed<'a>
        = Uint<'a, E::Borrowed<'a>>
    where
        Self: 'a;

    const ZERO: Self = Self::ZERO;

    fn borrow<'a>(&'a self) -> Self::Borrowed<'a>
    where
        Self: 'a,
    {
        Uint::from_encoding(self.0.borrow())
    }

    fn into_owned(self) -> Self::Owned {
        Uint::from_encoding(self.0.into_owned())
    }
}

impl<'enc, E> EncodingMut<'enc> for Uint<'enc, E>
where
    E: EncodingMut<'enc, Big = BigUint>,
{
    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<E::Small, Cow<E::Big>>)) {
        self.0.update_encoding(f);
    }
}
