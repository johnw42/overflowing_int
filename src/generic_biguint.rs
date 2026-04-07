use crate::big_number::{BigNumber, BigNumberDigits};
use crate::generic_bignum::GenericBigNum;
use crate::generic_bignum::encoding::{Decode, Decoded, Encoding};
use crate::small_num::SmallNumber;
use crate::{
    duplicate_arith_ops, duplicate_bit_ops, duplicate_prims, duplicate_shift_ops, duplicate_uprims,
};
use num_bigint::BigUint;
use num_traits::{Pow, Zero};
use paste::paste;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

/// An unsigned big integer type that can be used with any encoding that implements `Encoding` with `Big = BigUint`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GenericBigUint<'a, E: Encoding<'a>>(pub(crate) GenericBigNum<'a, E>);

impl<'a, E: Encoding<'a, Big = BigUint>> GenericBigUint<'a, E> {
    /// Creates and initializes a [`GenericBigUint`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn new(digits: Vec<u32>) -> GenericBigUint<'a, E> {
        Self(GenericBigNum::from_big(E::Big::new(digits)))
    }

    /// Creates and initializes a [`GenericBigUint`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn from_slice(slice: &[u32]) -> GenericBigUint<'a, E> {
        Self(GenericBigNum::from_big(E::Big::from_slice(slice)))
    }

    /// Assign a value to a [`GenericBigUint`].
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn assign_from_slice(&mut self, slice: &[u32]) {
        *self = Self::from_slice(slice);
    }

    /// Creates and initializes a [`GenericBigUint`].
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
    pub fn from_bytes_be(bytes: &[u8]) -> GenericBigUint<'a, E> {
        Self(
            if let Some(from_bytes) = SmallNumber::from_bytes_be(bytes) {
                GenericBigNum::from_small(from_bytes)
            } else {
                GenericBigNum::from_big(E::Big::from_bytes_be(bytes))
            },
        )
    }

    /// Creates and initializes a [`GenericBigUint`].
    ///
    /// The bytes are in little-endian byte order.
    #[inline]
    pub fn from_bytes_le(bytes: &[u8]) -> GenericBigUint<'a, E> {
        Self(GenericBigNum::from_big(E::Big::from_bytes_le(bytes)))
    }

    /// Creates and initializes a [`GenericBigUint`]. The input slice must contain
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
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Option<GenericBigUint<'a, E>> {
        E::Big::parse_bytes(buf, radix)
            .map(GenericBigNum::from_big)
            .map(Self)
    }

    /// Creates and initializes a [`GenericBigUint`]. Each `u8` of the input slice is
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
    pub fn from_radix_be(buf: &[u8], radix: u32) -> Option<GenericBigUint<'a, E>> {
        E::Big::from_radix_be(buf, radix)
            .map(GenericBigNum::from_big)
            .map(Self)
    }

    /// Creates and initializes a [`GenericBigUint`]. Each `u8` of the input slice is
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
    pub fn from_radix_le(buf: &[u8], radix: u32) -> Option<GenericBigUint<'a, E>> {
        E::Big::from_radix_le(buf, radix)
            .map(GenericBigNum::from_big)
            .map(Self)
    }

    /// Returns the byte representation of the [`GenericBigUint`] in big-endian byte order.
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

    /// Returns the byte representation of the [`GenericBigUint`] in little-endian byte order.
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

    /// Returns the `u32` digits representation of the [`GenericBigUint`] ordered least significant digit
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

    /// Returns the `u64` digits representation of the [`GenericBigUint`] ordered least significant digit
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

    /// Returns an iterator of `u32` digits representation of the [`GenericBigUint`] ordered least
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

    /// Returns an iterator of `u64` digits representation of the [`GenericBigUint`] ordered least
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

    /// Determines the fewest bits necessary to express the [`GenericBigUint`].
    #[inline]
    pub fn bits(&self) -> u64 {
        self.0.bits()
    }

    /// Returns `self ^ exponent`.
    pub fn pow(&self, exponent: u32) -> Self {
        Self((&self.0).pow(exponent))
    }

    /// Returns `(self ^ exponent) % modulus`.
    ///
    /// Panics if the modulus is zero.
    pub fn modpow(&self, exponent: &Self, modulus: &Self) -> Self {
        Self(self.0.modpow(&exponent.0, &modulus.0))
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
        self.0.modinv(&modulus.0).map(Self)
    }

    /// Returns the truncated principal square root of `self` --
    /// see [Roots::sqrt](https://docs.rs/num-integer/0.1/num_integer/trait.Roots.html#method.sqrt)
    pub fn sqrt(&self) -> Self {
        Self(self.0.sqrt())
    }

    /// Returns the truncated principal cube root of `self` --
    /// see [Roots::cbrt](https://docs.rs/num-integer/0.1/num_integer/trait.Roots.html#method.cbrt).
    pub fn cbrt(&self) -> Self {
        Self(self.0.cbrt())
    }

    /// Returns the truncated principal `n`th root of `self` --
    /// see [Roots::nth_root](https://docs.rs/num-integer/0.1/num_integer/trait.Roots.html#tymethod.nth_root).
    pub fn nth_root(&self, n: u32) -> Self {
        Self(self.0.nth_root(n))
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

impl<'a, E: Encoding<'a, Big = BigUint>> From<GenericBigNum<'a, E>> for GenericBigUint<'a, E> {
    fn from(value: GenericBigNum<'a, E>) -> Self {
        Self(value)
    }
}

impl<'a, E: Encoding<'a, Big = BigUint>> From<GenericBigUint<'a, E>> for GenericBigNum<'a, E> {
    fn from(value: GenericBigUint<'a, E>) -> Self {
        value.0
    }
}

impl<'a, E: Encoding<'a>> Decode<'a, E::Small> for GenericBigUint<'a, E>
where
    E::Small: SmallNumber<Big = E::Big>,
    E::Big: BigNumber,
{
    fn decode(self) -> Decoded<E::Small, Cow<'a, E::Big>> {
        self.0.decode()
    }

    fn small(&self) -> Option<E::Small> {
        self.0.small()
    }

    fn into_big_cow(self) -> Cow<'a, E::Big> {
        self.0.into_big_cow()
    }

    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<E::Small, Cow<E::Big>>) -> T) -> T {
        self.0.with_decoded(f)
    }
}

impl<'a, E: Encoding<'a>> Encoding<'a> for GenericBigUint<'a, E>
where
    E::Small: SmallNumber<Big = E::Big>,
    E::Big: BigNumber,
{
    type Small = E::Small;
    type Big = E::Big;
    type Unsigned = E::Unsigned;

    fn from_small(s: E::Small) -> Self {
        Self(GenericBigNum::from_encoding(E::from_small(s)))
    }

    fn from_big_cow(b: Cow<'a, E::Big>) -> Self {
        Self(GenericBigNum::from_encoding(E::from_big_cow(b)))
    }

    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<E::Small, Cow<E::Big>>)) {
        self.0.update_encoding(f);
    }
}

macro_rules! impl_binary_op_traits {
    ($trait:ident,
        $op_fn:ident,
        $lhs_type:ty,
        $lhs_param:ident,
        $lhs_expr:expr,
        $lhs_ref_expr:expr,
        $rhs_type:ty,
        $rhs_param:ident,
        $rhs_expr:expr,
        $rhs_ref_expr:expr
    ) => {
        impl<'a, E: Encoding<'a, Big = BigUint>> $trait<$rhs_type> for $lhs_type {
            type Output = GenericBigUint<'a, E>;

            fn $op_fn($lhs_param: Self, $rhs_param: $rhs_type) -> GenericBigUint<'a, E> {
                GenericBigUint($lhs_expr.$op_fn($rhs_expr))
            }
        }

        impl<'a, E: Encoding<'a, Big = BigUint>> $trait<&$rhs_type> for $lhs_type {
            type Output = GenericBigUint<'a, E>;

            fn $op_fn($lhs_param: Self, $rhs_param: &$rhs_type) -> GenericBigUint<'a, E> {
                GenericBigUint($lhs_expr.$op_fn($rhs_ref_expr))
            }
        }

        impl<'a, E: Encoding<'a, Big = BigUint>> $trait<$rhs_type> for &$lhs_type {
            type Output = GenericBigUint<'a, E>;

            fn $op_fn($lhs_param: Self, $rhs_param: $rhs_type) -> GenericBigUint<'a, E> {
                GenericBigUint($lhs_ref_expr.$op_fn($rhs_expr))
            }
        }

        impl<'a, E: Encoding<'a, Big = BigUint>> $trait<&$rhs_type> for &$lhs_type {
            type Output = GenericBigUint<'a, E>;

            fn $op_fn($lhs_param: Self, $rhs_param: &$rhs_type) -> GenericBigUint<'a, E> {
                GenericBigUint($lhs_ref_expr.$op_fn($rhs_ref_expr))
            }
        }
    };
}

macro_rules! impl_binary_assign_op_trait {
    ($trait:ident, $op_fn:ident, $rhs_type:ty, $rhs_param:ident, $rhs_expr:expr, $rhs_ref_expr:expr) => {
        paste! {
            impl<'a, E: Encoding<'a, Big = BigUint>> [<$trait Assign>]<$rhs_type> for GenericBigUint<'a, E> {
                fn [<$op_fn _assign>](&mut self, $rhs_param: $rhs_type) {
                    self.0.[<$op_fn _assign>]($rhs_expr)
                }
            }
        }
    };
}

macro_rules! impl_binary_assign_ref_op_trait {
    ($trait:ident, $op_fn:ident) => {
        paste! {
            impl<'a, E: Encoding<'a, Big = BigUint>> [<$trait Assign>]<&GenericBigUint<'a, E>> for GenericBigUint<'a, E> {
                fn [<$op_fn _assign>](&mut self, rhs: &GenericBigUint<'a, E>) {
                    self.0.[<$op_fn _assign>](&rhs.0)
                }
            }
        }
    };
}

macro_rules! impl_pow_traits {
    ($rhs_type:ty, $rhs_param:ident, $rhs_expr:expr, $rhs_ref_expr:expr) => {
        paste! {
            impl<'a, E: Encoding<'a, Big = BigUint, Unsigned = E>> Pow<$rhs_type> for GenericBigUint<'a, E> {
                type Output = GenericBigUint<'a, E>;

                fn pow(self, $rhs_param: $rhs_type) -> GenericBigUint<'a, E> {
                    GenericBigUint(self.0.pow($rhs_expr))
                }
            }

            impl<'a, E: Encoding<'a, Big = BigUint>> Pow<&$rhs_type> for &GenericBigUint<'a, E> {
                type Output = GenericBigUint<'a, E>;

                fn pow(self, $rhs_param: &$rhs_type) -> GenericBigUint<'a, E> {
                    GenericBigUint(Pow::pow(&self.0, $rhs_ref_expr))
                }
            }
        }
    };
}
macro_rules! impl_pow_traits_for_ref {
    ($rhs_type:ty, $rhs_param:ident, $rhs_expr:expr, $rhs_ref_expr:expr) => {
        paste! {
            impl<'a, E: Encoding<'a, Big = BigUint>> Pow<&$rhs_type> for GenericBigUint<'a, E> {
                type Output = GenericBigUint<'a, E>;

                fn pow(self, $rhs_param: &$rhs_type) -> GenericBigUint<'a, E> {
                    GenericBigUint(self.0.pow($rhs_expr))
                }
            }

            impl<'a, E: Encoding<'a, Big = BigUint, Unsigned = E>> Pow<&$rhs_type> for &GenericBigUint<'a, E> {
                type Output = GenericBigUint<'a, E>;

                fn pow(self, $rhs_param: &$rhs_type) -> GenericBigUint<'a, E> {
                    GenericBigUint((&self.0).pow($rhs_ref_expr))
                }
            }
        }
    };
}

// Implementations of numeric traits for `GenericBigUint`.  The number of
// implementations is quite large, so we use macros to generate them.  The
// reason for implementing so many variants is to allow `GenericBigUint` to serve
// as a drop-in replacement for `BigUint`, which implements the same traits.
duplicate_arith_ops! {
    paste! {
        impl_binary_op_traits!(op_trait, op_fn,
             GenericBigUint<'a, E>, self, self.0, &self.0,
             GenericBigUint<'a, E>, rhs,  rhs.0,  &rhs.0);
        impl_binary_assign_op_trait!(op_trait, op_fn, GenericBigUint<'a, E>, rhs, rhs.0, &rhs.0);
        impl_binary_assign_ref_op_trait!(op_trait, op_fn);
    }
    duplicate_uprims! { paste! {
        impl_binary_op_traits!(op_trait, op_fn,
            GenericBigUint<'a, E>, self, self.0, &self.0,
            prim, rhs, rhs, rhs);
        impl_binary_op_traits!(op_trait, op_fn,
            prim, self, self, self,
            GenericBigUint<'a, E>, rhs, rhs.0, &rhs.0);
        impl_binary_assign_op_trait!(op_trait, op_fn, prim, rhs, rhs, rhs);
    } }
}
duplicate_shift_ops! {
    duplicate_prims! { paste! {
        impl_binary_op_traits!(op_trait, op_fn,
             GenericBigUint<'a, E>, self, self.0, &self.0,
             prim, rhs, rhs, rhs);
        impl_binary_assign_op_trait!(op_trait, op_fn, prim, rhs, rhs, rhs);
    } }
}
duplicate_bit_ops! {
    paste! {
        impl_binary_op_traits!(op_trait, op_fn,
             GenericBigUint<'a, E>, self, self.0, &self.0,
             GenericBigUint<'a, E>, rhs,  rhs.0,  &rhs.0);
        impl_binary_assign_op_trait!(op_trait, op_fn, GenericBigUint<'a, E>, rhs, rhs.0, &rhs.0);
        impl_binary_assign_ref_op_trait!(op_trait, op_fn);
    }
}

// impl_pow_traits!(GenericBigUint<'a, E>, exponent, exponent.0, &exponent.0);
// impl_pow_traits_for_ref!(GenericBigUint<'a, E>, exponent, exponent.0, &exponent.0);
// duplicate_uprims! { paste! {
//     impl_pow_traits!(prim, exponent, exponent, &exponent);
// } }
