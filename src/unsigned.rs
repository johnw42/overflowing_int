use crate::big_number::BigNumberDigits;
use crate::encoding::{BorrowingEncoding, Decode, Decoded, Encode, Encoding, EncodingKind};
use crate::small_num::SmallNumber;
use num_bigint::BigUint;
use std::borrow::Cow;
use std::marker::PhantomData;

/// An unsigned big integer type that can be used with any encoding that implements `Encoding` with `Big = BigUint`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Uint<'a, E: Encoding<'a>>(pub(crate) E, PhantomData<&'a ()>);

impl<'a, E: Encoding<'a, Big = BigUint>> Uint<'a, E> {
    fn from_encoding(encoding: E) -> Self {
        Self(encoding, PhantomData)
    }

    /// Converts this big integer to a version with a static lifetime.  This may require cloning a `BigInt`.
    pub fn into_static(self) -> Uint<'static, E::Static> {
        Uint::from_encoding(self.0.into_static())
    }

    /// Converts this big integer to into one that borrows from this one's data.
    pub fn borrow<'b>(&'b self) -> Uint<'b, E::WithLifetime<'b>>
    where
        E: BorrowingEncoding<'a>,
        'a: 'b,
    {
        Uint::from_encoding(self.0.borrow())
    }

    /// Creates and initializes a [`Uint`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn new(digits: Vec<u32>) -> Uint<'a, E> {
        Self::from_encoding(E::from_big(E::Big::new(digits)))
    }

    /// Creates and initializes a [`Uint`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn from_slice(slice: &[u32]) -> Uint<'a, E> {
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
    /// use compact_bigint::RcBigUint;
    ///
    /// assert_eq!(RcBigUint::from_bytes_be(b"A"),
    ///            RcBigUint::parse_bytes(b"65", 10).unwrap());
    /// assert_eq!(RcBigUint::from_bytes_be(b"AA"),
    ///            RcBigUint::parse_bytes(b"16705", 10).unwrap());
    /// assert_eq!(RcBigUint::from_bytes_be(b"AB"),
    ///            RcBigUint::parse_bytes(b"16706", 10).unwrap());
    /// assert_eq!(RcBigUint::from_bytes_be(b"Hello world!"),
    ///            RcBigUint::parse_bytes(b"22405534230753963835153736737", 10).unwrap());
    /// ```
    #[inline]
    pub fn from_bytes_be(bytes: &[u8]) -> Uint<'a, E> {
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
    pub fn from_bytes_le(bytes: &[u8]) -> Uint<'a, E> {
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
    /// use compact_bigint::RcBigUint;
    ///
    /// assert_eq!(RcBigUint::parse_bytes(b"1234", 10), Some(RcBigUint::from(1234u32)));
    /// assert_eq!(RcBigUint::parse_bytes(b"ABCD", 16), Some(RcBigUint::from(0xABCDu32)));
    /// assert_eq!(RcBigUint::parse_bytes(b"G", 16), None);
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
    /// use compact_bigint::RcBigUint;
    ///
    /// let inbase190 = &[15, 33, 125, 12, 14];
    /// let a = RcBigUint::from_radix_be(inbase190, 190).unwrap();
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
    /// use compact_bigint::RcBigUint;
    ///
    /// let inbase190 = &[14, 12, 125, 33, 15];
    /// let a = RcBigUint::from_radix_le(inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_le(190), inbase190);
    /// ```
    pub fn from_radix_le(buf: &[u8], radix: u32) -> Option<Uint<'a, E>> {
        E::Big::from_radix_le(buf, radix)
            .map(E::from_big)
            .map(Self::from_encoding)
    }

    /// Returns the byte representation of the [`Uint`] in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::RcBigUint;
    ///
    /// let i = RcBigUint::parse_bytes(b"1125", 10).unwrap();
    /// assert_eq!(i.to_bytes_be(), vec![4, 101]);
    /// ```
    #[inline]
    pub fn to_bytes_be(&self) -> Vec<u8> {
        self.0.with_big_cow(|big| big.as_ref().to_bytes_be())
    }

    /// Returns the byte representation of the [`Uint`] in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::RcBigUint;
    ///
    /// let i = RcBigUint::parse_bytes(b"1125", 10).unwrap();
    /// assert_eq!(i.to_bytes_le(), vec![101, 4]);
    /// ```
    #[inline]
    pub fn to_bytes_le(&self) -> Vec<u8> {
        self.0.with_big_cow(|big| big.as_ref().to_bytes_le())
    }

    /// Returns the `u32` digits representation of the [`Uint`] ordered least significant digit
    /// first.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::RcBigUint;
    ///
    /// assert_eq!(RcBigUint::from(1125u32).to_u32_digits(), vec![1125]);
    /// assert_eq!(RcBigUint::from(4294967295u32).to_u32_digits(), vec![4294967295]);
    /// assert_eq!(RcBigUint::from(4294967296u64).to_u32_digits(), vec![0, 1]);
    /// assert_eq!(RcBigUint::from(112500000000u64).to_u32_digits(), vec![830850304, 26]);
    /// ```
    #[inline]
    pub fn to_u32_digits(&self) -> Vec<u32> {
        self.0.with_big_cow(|big| big.as_ref().to_u32_digits())
    }

    /// Returns the `u64` digits representation of the [`Uint`] ordered least significant digit
    /// first.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::RcBigUint;
    ///
    /// assert_eq!(RcBigUint::from(1125u32).to_u64_digits(), vec![1125]);
    /// assert_eq!(RcBigUint::from(4294967295u32).to_u64_digits(), vec![4294967295]);
    /// assert_eq!(RcBigUint::from(4294967296u64).to_u64_digits(), vec![4294967296]);
    /// assert_eq!(RcBigUint::from(112500000000u64).to_u64_digits(), vec![112500000000]);
    /// assert_eq!(RcBigUint::from(1u128 << 64).to_u64_digits(), vec![0, 1]);
    /// ```
    #[inline]
    pub fn to_u64_digits(&self) -> Vec<u64> {
        self.0.with_big_cow(|big| big.as_ref().to_u64_digits())
    }

    /// Returns an iterator of `u32` digits representation of the [`Uint`] ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::RcBigUint;
    ///
    /// assert_eq!(RcBigUint::from(1125u32).iter_u32_digits().collect::<Vec<u32>>(), vec![1125]);
    /// assert_eq!(RcBigUint::from(4294967295u32).iter_u32_digits().collect::<Vec<u32>>(), vec![4294967295]);
    /// assert_eq!(RcBigUint::from(4294967296u64).iter_u32_digits().collect::<Vec<u32>>(), vec![0, 1]);
    /// assert_eq!(RcBigUint::from(112500000000u64).iter_u32_digits().collect::<Vec<u32>>(), vec![830850304, 26]);
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
    /// use compact_bigint::RcBigUint;
    ///
    /// assert_eq!(RcBigUint::from(1125u32).iter_u64_digits().collect::<Vec<u64>>(), vec![1125]);
    /// assert_eq!(RcBigUint::from(4294967295u32).iter_u64_digits().collect::<Vec<u64>>(), vec![4294967295]);
    /// assert_eq!(RcBigUint::from(4294967296u64).iter_u64_digits().collect::<Vec<u64>>(), vec![4294967296]);
    /// assert_eq!(RcBigUint::from(112500000000u64).iter_u64_digits().collect::<Vec<u64>>(), vec![112500000000]);
    /// assert_eq!(RcBigUint::from(1u128 << 64).iter_u64_digits().collect::<Vec<u64>>(), vec![0, 1]);
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
    /// use compact_bigint::RcBigUint;
    ///
    /// let i = RcBigUint::parse_bytes(b"ff", 16).unwrap();
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
    /// use compact_bigint::RcBigUint;
    ///
    /// assert_eq!(RcBigUint::from(0xFFFFu64).to_radix_be(159),
    ///            vec![2, 94, 27]);
    /// // 0xFFFF = 65535 = 2*(159^2) + 94*159 + 27
    /// ```
    #[inline]
    pub fn to_radix_be(&self, radix: u32) -> Vec<u8> {
        self.0.with_big_cow(|big| big.as_ref().to_radix_be(radix))
    }

    /// Returns the integer in the requested base in little-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based u8 number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::RcBigUint;
    ///
    /// assert_eq!(RcBigUint::from(0xFFFFu64).to_radix_le(159),
    ///            vec![27, 94, 2]);
    /// // 0xFFFF = 65535 = 27 + 94*159 + 2*(159^2)
    /// ```
    #[inline]
    pub fn to_radix_le(&self, radix: u32) -> Vec<u8> {
        self.0.with_big_cow(|big| big.as_ref().to_radix_le(radix))
    }

    /// Determines the fewest bits necessary to express the [`Uint`].
    #[inline]
    pub fn bits(&self) -> u64 {
        self.0.bits()
    }

    /// Returns `self ^ exponent`.
    pub fn pow(&self, exponent: u32) -> Self {
        #[allow(clippy::needless_borrow)]
        Self::from_encoding((&self.0).pow(exponent))
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
    /// use compact_bigint::RcBigUint;
    /// use num_traits::{One, Zero};
    ///
    /// let m = RcBigUint::from(383_u32);
    ///
    /// // Trivial cases
    /// assert_eq!(RcBigUint::zero().modinv(&m), None);
    /// assert_eq!(RcBigUint::one().modinv(&m), Some(RcBigUint::one()));
    /// let neg1 = &m - 1u32;
    /// assert_eq!(neg1.modinv(&m), Some(neg1));
    ///
    /// let a = RcBigUint::from(271_u32);
    /// let x = a.modinv(&m).unwrap();
    /// assert_eq!(x, RcBigUint::from(106_u32));
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
        self.0.with_big_cow(|cow| cow.as_ref().trailing_ones())
    }

    /// Returns the number of one bits.
    pub fn count_ones(&self) -> u64 {
        self.0.with_big_cow(|cow| cow.as_ref().count_ones())
    }

    /// Returns whether the bit in the given position is set
    pub fn bit(&self, bit: u64) -> bool {
        self.0.bit(bit)
    }

    /// Sets or clears the bit in the given position
    ///
    /// Note that setting a bit greater than the current bit length, a reallocation may be needed
    /// to store the new digits
    pub fn set_bit(&mut self, bit: u64, value: bool) {
        self.0.set_bit(bit, value)
    }
}

impl<'a, E: Encoding<'a>> Decode<'a, E::Small> for Uint<'a, E> {
    fn kind() -> EncodingKind {
        E::kind()
    }

    fn decode(self) -> Decoded<E::Small, Cow<'a, E::Big>> {
        self.0.decode()
    }

    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<E::Small, Cow<E::Big>>) -> T) -> T {
        self.0.with_decoded(f)
    }

    fn owns_bignum(&self) -> bool {
        self.0.owns_bignum()
    }
}

impl<'a, E: Encoding<'a>> Decode<'a, E::Small> for &Uint<'a, E> {
    fn kind() -> EncodingKind {
        E::kind()
    }

    fn decode(self) -> Decoded<E::Small, Cow<'a, E::Big>> {
        // TODO Can we do this without cloning?
        self.0.clone().decode()
    }

    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<E::Small, Cow<E::Big>>) -> T) -> T {
        self.0.with_decoded(f)
    }

    fn owns_bignum(&self) -> bool {
        false
    }
}

impl<'a, E: Encoding<'a, Big = BigUint>> Encode<'a, E::Small> for Uint<'a, E> {
    fn from_small(s: E::Small) -> Self {
        Self::from_encoding(E::from_small(s))
    }

    fn from_big_cow(b: Cow<'a, E::Big>) -> Self {
        Self::from_encoding(E::from_big_cow(b))
    }
}

impl<'a, E: Encoding<'a, Big = BigUint>> Encoding<'a> for Uint<'a, E> {
    type Small = E::Small;
    type Big = E::Big;
    type Unsigned = E::Unsigned;
    type Static = Uint<'static, E::Static>;

    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<E::Small, Cow<E::Big>>)) {
        self.0.update_encoding(f);
    }

    fn into_static(self) -> Self::Static {
        Uint::from_encoding(self.0.into_static())
    }
}

#[test]
fn test_borrow_and_static() {
    use crate::CowBigUint;
    use num_traits::Zero;

    struct CowTest<'a>(CowBigUint<'a>);

    impl<'a> CowTest<'a> {
        fn test(self) {
            // Prove that we can change the lifetime to a shorter one.
            Self::wants_borrowed(self.0.borrow());

            // Prove we can change the lifetime to 'static.
            Self::wants_static(self.0.clone().into_static());
        }

        fn wants_static(_x: CowBigUint<'static>) {}

        fn wants_borrowed<'b>(_x: CowBigUint<'b>)
        where
            'a: 'b,
        {
        }
    }

    CowTest(CowBigUint::zero()).test();
}
