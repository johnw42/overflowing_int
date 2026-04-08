use crate::big_number::BigNumberDigits;
use crate::generic_bignum::encoding::{Decode, Decoded, Encode, Encoding};
use crate::small_num::SmallNumber;
use crate::{
    duplicate_arith_ops, duplicate_bit_ops, duplicate_prims, duplicate_shift_ops, duplicate_uprims,
};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{Pow, Zero};
use paste::paste;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

/// A signed big integer type that can be used with any encoding that implements `Encoding` with `Big = BigInt`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GenericSignedBigNum<'a, E>(pub(crate) E, PhantomData<&'a ()>)
where
    E: Encoding<'a, Big = BigInt>;

impl<'a, E: Encoding<'a, Big = BigInt>> GenericSignedBigNum<'a, E> {
    fn from_encoding(encoding: E) -> Self {
        Self(encoding, PhantomData)
    }

    pub(crate) fn is_signed() -> bool {
        true
    }

    /// Converts this big integer to a version with a static lifetime.  This may require cloning a `BigInt`.
    pub fn into_static(self) -> GenericSignedBigNum<'static, E::Static> {
        GenericSignedBigNum::<'static, E::Static>::from_encoding(self.0.into_static())
    }

    /// Creates and initializes a E::Big.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn new(sign: Sign, digits: Vec<u32>) -> Self {
        Self::from_encoding(if sign == Sign::NoSign {
            E::from_small(E::Small::zero())
        } else {
            E::from_big(E::Big::new(sign, digits))
        })
    }

    /// Creates and initializes a bigint.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn from_slice(sign: Sign, slice: &[u32]) -> Self {
        Self::from_encoding(E::from_big(E::Big::from_slice(sign, slice)))
    }

    /// Reinitializes a bigint.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn assign_from_slice(&mut self, sign: Sign, slice: &[u32]) {
        *self = Self::from_slice(sign, slice);
    }

    /// Creates and initializes a bigint.
    ///
    /// The bytes are in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CowBigInt, Sign};
    ///
    /// assert_eq!(CowBigInt::from_bytes_be(Sign::Plus, b"A"),
    ///            CowBigInt::parse_bytes(b"65", 10).unwrap());
    /// assert_eq!(CowBigInt::from_bytes_be(Sign::Plus, b"AA"),
    ///            CowBigInt::parse_bytes(b"16705", 10).unwrap());
    /// assert_eq!(CowBigInt::from_bytes_be(Sign::Plus, b"AB"),
    ///            CowBigInt::parse_bytes(b"16706", 10).unwrap());
    /// assert_eq!(CowBigInt::from_bytes_be(Sign::Plus, b"Hello world!"),
    ///            CowBigInt::parse_bytes(b"22405534230753963835153736737", 10).unwrap());
    /// ```
    #[inline]
    pub fn from_bytes_be(sign: Sign, bytes: &[u8]) -> Self
    where
        E::Small: Neg<Output = E::Small>,
    {
        if let Some(from_bytes) = SmallNumber::from_bytes_be(bytes) {
            let unsigned = Self::from_encoding(E::from_small(from_bytes));
            match sign {
                Sign::Plus => unsigned,
                Sign::Minus => -unsigned,
                Sign::NoSign => Self::zero(),
            }
        } else {
            Self::from_encoding(E::from_big(E::Big::from_bytes_be(sign, bytes)))
        }
    }

    /// Creates and initializes a bigint.
    ///
    /// The bytes are in little-endian byte order.
    #[inline]
    pub fn from_bytes_le(sign: Sign, bytes: &[u8]) -> Self
    where
        E::Small: Neg<Output = E::Small>,
    {
        if let Some(from_bytes) = SmallNumber::from_bytes_le(bytes) {
            let unsigned = Self::from_encoding(E::from_small(from_bytes));
            match sign {
                Sign::Plus => unsigned,
                Sign::Minus => -unsigned,
                Sign::NoSign => Self::zero(),
            }
        } else {
            Self::from_encoding(E::from_big(E::Big::from_bytes_le(sign, bytes)))
        }
    }

    /// Creates and initializes a bigint from an array of bytes in
    /// two's complement binary representation.
    ///
    /// The digits are in big-endian base 2<sup>8</sup>.
    #[inline]
    pub fn from_signed_bytes_be(digits: &[u8]) -> Self {
        Self::from_encoding(E::from_big(E::Big::from_signed_bytes_be(digits)))
    }

    /// Creates and initializes a bigint from an array of bytes in two's complement.
    ///
    /// The digits are in little-endian base 2<sup>8</sup>.
    #[inline]
    pub fn from_signed_bytes_le(digits: &[u8]) -> Self {
        Self::from_encoding(E::from_big(E::Big::from_signed_bytes_le(digits)))
    }

    /// Creates and initializes a bigint.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{CowBigInt};
    ///
    /// assert_eq!(CowBigInt::parse_bytes(b"1234", 10), Some(CowBigInt::from(1234)));
    /// assert_eq!(CowBigInt::parse_bytes(b"ABCD", 16), Some(CowBigInt::from(0xABCD)));
    /// assert_eq!(CowBigInt::parse_bytes(b"G", 16), None);
    /// ```
    #[inline]
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self> {
        E::parse_bytes(buf, radix).map(Self::from_encoding)
    }

    /// Creates and initializes a bigint. Each u8 of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in big-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{RcBigInt, Sign};
    ///
    /// let inbase190 = vec![15, 33, 125, 12, 14];
    /// let a = RcBigInt::from_radix_be(Sign::Minus, &inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_be(190), (Sign::Minus, inbase190));
    /// ```
    #[inline]
    pub fn from_radix_be(sign: Sign, buf: &[u8], radix: u32) -> Option<Self> {
        E::Big::from_radix_be(sign, buf, radix)
            .map(E::from_big)
            .map(Self::from_encoding)
    }

    /// Creates and initializes a bigint. Each u8 of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in little-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{RcBigInt, Sign};
    ///
    /// let inbase190 = vec![14, 12, 125, 33, 15];
    /// let a = RcBigInt::from_radix_le(Sign::Minus, &inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_le(190), (Sign::Minus, inbase190));
    /// ```
    #[inline]
    pub fn from_radix_le(sign: Sign, buf: &[u8], radix: u32) -> Option<Self> {
        E::Big::from_radix_le(sign, buf, radix)
            .map(E::from_big)
            .map(Self::from_encoding)
    }

    /// Returns the sign and the byte representation of the bigint in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{RcBigInt, Sign};
    ///
    /// let i = RcBigInt::from(-1125);
    /// assert_eq!(i.to_bytes_be(), (Sign::Minus, vec![4, 101]));
    /// ```
    #[inline]
    pub fn to_bytes_be(&self) -> (Sign, Vec<u8>) {
        self.0.with_big_cow(|big| big.as_ref().to_bytes_be())
    }

    /// Returns the sign and the byte representation of the bigint in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{RcBigInt, Sign};
    ///
    /// let i = RcBigInt::from(-1125);
    /// assert_eq!(i.to_bytes_le(), (Sign::Minus, vec![101, 4]));
    /// ```
    #[inline]
    pub fn to_bytes_le(&self) -> (Sign, Vec<u8>) {
        self.0.with_big_cow(|big| big.as_ref().to_bytes_le())
    }

    /// Returns the sign and the `u32` digits representation of the bigint ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::compact_bigint::{RcBigInt, Sign};
    ///
    /// assert_eq!(RcBigInt::from(-1125).to_u32_digits(), (Sign::Minus, vec![1125]));
    /// assert_eq!(RcBigInt::from(4294967295u32).to_u32_digits(), (Sign::Plus, vec![4294967295]));
    /// assert_eq!(RcBigInt::from(4294967296u64).to_u32_digits(), (Sign::Plus, vec![0, 1]));
    /// assert_eq!(RcBigInt::from(-112500000000i64).to_u32_digits(), (Sign::Minus, vec![830850304, 26]));
    /// assert_eq!(RcBigInt::from(112500000000i64).to_u32_digits(), (Sign::Plus, vec![830850304, 26]));
    /// ```
    #[inline]
    pub fn to_u32_digits(&self) -> (Sign, Vec<u32>) {
        self.0.with_big_cow(|big| big.as_ref().to_u32_digits())
    }

    #[inline]
    pub fn to_u64_digits(&self) -> (Sign, Vec<u64>) {
        self.0.with_big_cow(|big| big.as_ref().to_u64_digits())
    }

    /// Returns the two's-complement byte representation of the bigint in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{RcBigInt};
    ///
    /// let i = RcBigInt::from(-1125);
    /// assert_eq!(i.to_signed_bytes_be(), vec![251, 155]);
    /// ```
    #[inline]
    pub fn to_signed_bytes_be(&self) -> Vec<u8> {
        self.0.with_big_cow(|big| big.as_ref().to_signed_bytes_be())
    }

    /// Returns the two's-complement byte representation of the bigint in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{RcBigInt};
    ///
    /// let i = RcBigInt::from(-1125);
    /// assert_eq!(i.to_signed_bytes_le(), vec![155, 251]);
    /// ```
    #[inline]
    pub fn to_signed_bytes_le(&self) -> Vec<u8> {
        self.0.with_big_cow(|big| big.as_ref().to_signed_bytes_le())
    }

    /// Returns the integer formatted as a string in the given radix.
    /// `radix` must be in the range `2...36`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{RcBigInt};
    ///
    /// let i = RcBigInt::parse_bytes(b"ff", 16).unwrap();
    /// assert_eq!(i.to_str_radix(16), "ff");
    /// ```
    #[inline]
    pub fn to_str_radix(&self, radix: u32) -> String {
        <Self as Encoding>::to_str_radix(self, radix)
    }

    /// Returns the integer in the requested base in big-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based u8 number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::{RcBigInt, Sign};
    ///
    /// assert_eq!(RcBigInt::from(-0xFFFFi64).to_radix_be(159),
    ///            (Sign::Minus, vec![2, 94, 27]));
    /// // 0xFFFF = 65535 = 2*(159^2) + 94*159 + 27
    /// ```
    #[inline]
    pub fn to_radix_be(&self, radix: u32) -> (Sign, Vec<u8>) {
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
    /// use compact_bigint::{RcBigInt, Sign};
    ///
    /// assert_eq!(RcBigInt::from(-0xFFFFi64).to_radix_le(159),
    ///            (Sign::Minus, vec![27, 94, 2]));
    /// // 0xFFFF = 65535 = 27 + 94*159 + 2*(159^2)
    /// ```
    #[inline]
    pub fn to_radix_le(&self, radix: u32) -> (Sign, Vec<u8>) {
        self.0.with_big_cow(|big| big.as_ref().to_radix_le(radix))
    }

    #[inline]
    pub fn sign(&self) -> Sign {
        self.0.with_decoded(|encoded| match encoded {
            Decoded::Small(n) => match n.cmp(&E::Small::zero()) {
                Ordering::Equal => Sign::NoSign,
                Ordering::Greater => Sign::Plus,
                Ordering::Less => Sign::Minus,
            },
            Decoded::Big(n) => n.sign(),
        })
    }

    #[inline]
    pub fn into_parts(self) -> (Sign, BigUint) {
        self.0.into_bigint().into_parts()
    }

    #[inline]
    pub fn bit(&self, bit: u64) -> bool {
        self.0.bit(bit)
    }

    #[inline]
    pub fn bits(&self) -> u64 {
        self.0.bits()
    }

    #[inline]
    pub fn to_biguint(&self) -> Option<BigUint> {
        self.0.with_big_cow(|big| big.as_ref().to_biguint())
    }

    #[inline]
    pub fn checked_add(&self, v: &Self) -> Option<Self> {
        self.0.checked_add(&v.0).map(Self::from_encoding)
    }

    #[inline]
    pub fn checked_sub(&self, v: &Self) -> Option<Self> {
        self.0.checked_sub(&v.0).map(Self::from_encoding)
    }

    #[inline]
    pub fn checked_mul(&self, v: &Self) -> Option<Self> {
        self.0.checked_mul(&v.0).map(Self::from_encoding)
    }

    #[inline]
    pub fn checked_div(&self, v: &Self) -> Option<Self> {
        self.0.checked_div(&v.0).map(Self::from_encoding)
    }

    #[inline]
    pub fn pow(&self, exponent: u32) -> Self {
        #[allow(clippy::needless_borrow)]
        Self::from_encoding((&self.0).pow(exponent))
    }

    #[inline]
    pub fn modpow(&self, exponent: Self, modulus: Self) -> Self {
        Self::from_encoding(self.0.modpow(&exponent.0, &modulus.0))
    }

    #[inline]
    pub fn sqrt(&self) -> Self {
        Self::from_encoding(self.0.sqrt())
    }

    #[inline]
    pub fn cbrt(&self) -> Self {
        Self::from_encoding(self.0.cbrt())
    }

    #[inline]
    pub fn nth_root(&self, n: u32) -> Self {
        Self::from_encoding(self.0.nth_root(n))
    }

    #[inline]
    pub fn trailing_zeros(&self) -> Option<u64> {
        self.0.trailing_zeros()
    }

    #[inline]
    pub fn iter_u32_digits(&self) -> impl BigNumberDigits<'_, u32> {
        self.0.iter_u32_digits()
    }

    #[inline]
    pub fn iter_u64_digits(&self) -> impl BigNumberDigits<'_, u64> {
        self.0.iter_u64_digits()
    }

    #[inline]
    pub fn modinv(&self, modulus: Self) -> Option<Self> {
        self.0.modinv(&modulus.0).map(Self::from_encoding)
    }

    #[inline]
    pub fn set_bit(&mut self, bit: u64, value: bool) {
        self.0.set_bit(bit, value);
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Decode<'a, E::Small> for GenericSignedBigNum<'a, E> {
    fn decode(self) -> Decoded<E::Small, Cow<'a, <E::Small as SmallNumber>::Big>> {
        self.0.decode()
    }

    fn with_decoded<T>(
        &self,
        f: impl FnOnce(Decoded<E::Small, Cow<<E::Small as SmallNumber>::Big>>) -> T,
    ) -> T {
        self.0.with_decoded(f)
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Decode<'a, E::Small> for &GenericSignedBigNum<'a, E> {
    fn decode(self) -> Decoded<E::Small, Cow<'a, <E::Small as SmallNumber>::Big>> {
        self.0.clone().decode()
    }

    fn with_decoded<T>(
        &self,
        f: impl FnOnce(Decoded<E::Small, Cow<<E::Small as SmallNumber>::Big>>) -> T,
    ) -> T {
        self.0.with_decoded(f)
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Encode<'a, E::Small> for GenericSignedBigNum<'a, E> {
    fn from_small(s: E::Small) -> Self {
        Self::from_encoding(E::from_small(s))
    }

    fn from_big_cow(b: Cow<'a, E::Big>) -> Self {
        Self::from_encoding(E::from_big_cow(b))
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Encoding<'a> for GenericSignedBigNum<'a, E> {
    type Small = E::Small;
    type Big = E::Big;
    type Unsigned = E::Unsigned;
    type Static = GenericSignedBigNum<'static, E::Static>;

    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<E::Small, Cow<E::Big>>)) {
        self.0.update_encoding(f);
    }

    fn into_static(self) -> Self::Static {
        GenericSignedBigNum::from_encoding(self.0.into_static())
    }
}

// impl<'a, E: Encoding<'a, Big = BigInt>> From<GenericBigNum<'a, E>> for GenericSignedBigNum<'a, E> {
//     fn from(value: GenericBigNum<'a, E>) -> Self {
//         Self(value)
//     }
// }

// impl<'a, E: Encoding<'a, Big = BigInt>> From<GenericSignedBigNum<'a, E>> for GenericBigNum<'a, E> {
//     fn from(value: GenericSignedBigNum<'a, E>) -> Self {
//         value.0
//     }
// }

// macro_rules! impl_binary_op_traits {
//     ($trait:ident,
//         $op_fn:ident,
//         $lhs_type:ty,
//         $lhs_param:ident,
//         $lhs_expr:expr,
//         $lhs_ref_expr:expr,
//         $rhs_type:ty,
//         $rhs_param:ident,
//         $rhs_expr:expr,
//         $rhs_ref_expr:expr
//     ) => {
//         impl<'a, E: Encoding<'a, Big = BigInt>> $trait<$rhs_type> for $lhs_type {
//             type Output = GenericSignedBigNum<'a, E>;

//             fn $op_fn($lhs_param: Self, $rhs_param: $rhs_type) -> GenericSignedBigNum<'a, E> {
//                 GenericSignedBigNum::from_encoding($lhs_expr.$op_fn($rhs_expr))
//             }
//         }

//         impl<'a, E: Encoding<'a, Big = BigInt>> $trait<&$rhs_type> for $lhs_type {
//             type Output = GenericSignedBigNum<'a, E>;

//             fn $op_fn($lhs_param: Self, $rhs_param: &$rhs_type) -> GenericSignedBigNum<'a, E> {
//                 GenericSignedBigNum::from_encoding($lhs_expr.$op_fn($rhs_ref_expr))
//             }
//         }

//         impl<'a, E: Encoding<'a, Big = BigInt>> $trait<$rhs_type> for &$lhs_type {
//             type Output = GenericSignedBigNum<'a, E>;

//             fn $op_fn($lhs_param: Self, $rhs_param: $rhs_type) -> GenericSignedBigNum<'a, E> {
//                 GenericSignedBigNum::from_encoding($lhs_ref_expr.$op_fn($rhs_expr))
//             }
//         }

//         impl<'a, E: Encoding<'a, Big = BigInt>> $trait<&$rhs_type> for &$lhs_type {
//             type Output = GenericSignedBigNum<'a, E>;

//             fn $op_fn($lhs_param: Self, $rhs_param: &$rhs_type) -> GenericSignedBigNum<'a, E> {
//                 GenericSignedBigNum::from_encoding($lhs_ref_expr.$op_fn($rhs_ref_expr))
//             }
//         }
//     };
// }

// macro_rules! impl_binary_assign_op_trait {
//     ($trait:ident, $op_fn:ident, $rhs_type:ty, $rhs_param:ident, $rhs_expr:expr, $rhs_ref_expr:expr) => {
//         paste! {
//             impl<'a, E: Encoding<'a, Big = BigInt>> [<$trait Assign>]<$rhs_type> for GenericSignedBigNum<'a, E> {
//                 fn [<$op_fn _assign>](&mut self, $rhs_param: $rhs_type) {
//                     self.0.[<$op_fn _assign>]($rhs_expr)
//                 }
//             }
//         }
//     };
// }

// macro_rules! impl_binary_assign_ref_op_trait {
//     ($trait:ident, $op_fn:ident) => {
//         paste! {
//             impl<'a, E: Encoding<'a, Big = BigInt>> [<$trait Assign>]<&GenericSignedBigNum<'a, E>> for GenericSignedBigNum<'a, E> {
//                 fn [<$op_fn _assign>](&mut self, rhs: &GenericSignedBigNum<'a, E>) {
//                     self.0.[<$op_fn _assign>](&rhs.0)
//                 }
//             }
//         }
//     };
// }

// macro_rules! impl_pow_traits {
//     ($rhs_type:ty, $rhs_param:ident, $rhs_expr:expr, $rhs_ref_expr:expr) => {
//         paste! {
//             impl<'a, E: Encoding<'a>> Pow<$rhs_type> for GenericSignedBigNum<'a, E> {
//                 type Output = GenericSignedBigNum<'a, E>;

//                 fn pow(self, $rhs_param: $rhs_type) -> GenericSignedBigNum<'a, E> {
//                     GenericSignedBigNum::from_encoding(Pow::pow(&self.0, $rhs_ref_expr))
//                 }
//             }

//             impl<'a, E: Encoding<'a>> Pow<&$rhs_type> for GenericSignedBigNum<'a, E> {
//                 type Output = GenericSignedBigNum<'a, E>;

//                 fn pow(self, $rhs_param: &$rhs_type) -> GenericSignedBigNum<'a, E> {
//                     GenericSignedBigNum::from_encoding(Pow::pow(&self.0, $rhs_ref_expr))
//                 }
//             }
//         }
//     };
// }
// macro_rules! impl_pow_traits_for_ref {
//     ($rhs_type:ty, $rhs_param:ident, $rhs_expr:expr, $rhs_ref_expr:expr) => {
//         paste! {
//             impl<'a, E: Encoding<'a>> Pow<$rhs_type> for &GenericSignedBigNum<'a, E> {
//                 type Output = GenericSignedBigNum<'a, E>;

//                 fn pow(self, $rhs_param: $rhs_type) -> GenericSignedBigNum<'a, E> {
//                     GenericSignedBigNum(Pow::pow(&self.0, $rhs_ref_expr))
//                 }
//             }

//             impl<'a, E: Encoding<'a>> Pow<&$rhs_type> for &GenericSignedBigNum<'a, E> {
//                 type Output = GenericSignedBigNum<'a, E>;

//                 fn pow(self, $rhs_param: &$rhs_type) -> GenericSignedBigNum<'a, E> {
//                     GenericSignedBigNum(Pow::pow(&self.0, $rhs_ref_expr))
//                 }
//             }
//         }
//     };
// }

// // Implementations of numeric traits for `GenericSignedBigNum`.  The number of
// // implementations is quite large, so we use macros to generate them.  The
// // reason for implementing so many variants is to allow `GenericSignedBigNum` to serve
// // as a drop-in replacement for `BigInt`, which implements the same traits.
// duplicate_arith_ops! {
//     paste! {
//         impl_binary_op_traits!(op_trait, op_fn,
//              GenericSignedBigNum<'a, E>, self, self.0, &self.0,
//              GenericSignedBigNum<'a, E>, rhs,  rhs.0,  &rhs.0);
//         impl_binary_assign_op_trait!(op_trait, op_fn, GenericSignedBigNum<'a, E>, rhs, rhs.0, &rhs.0);
//         impl_binary_assign_ref_op_trait!(op_trait, op_fn);
//     }
//     duplicate_prims! { paste! {
//         impl_binary_op_traits!(op_trait, op_fn,
//              GenericSignedBigNum<'a, E>, self, self.0, &self.0,
//              prim, rhs, rhs, rhs);
//         impl_binary_op_traits!(op_trait, op_fn,
//              prim, self, self, self,
//              GenericSignedBigNum<'a, E>, rhs, rhs.0, &rhs.0);
//         impl_binary_assign_op_trait!(op_trait, op_fn, prim, rhs, rhs, rhs);
//     } }
// }
// duplicate_shift_ops! {
//     duplicate_prims! { paste! {
//         impl_binary_op_traits!(op_trait, op_fn,
//              GenericSignedBigNum<'a, E>, self, self.0, &self.0,
//              prim, rhs, rhs, rhs);
//         impl_binary_assign_op_trait!(op_trait, op_fn, prim, rhs, rhs, rhs);
//     } }
// }
// duplicate_bit_ops! {
//     paste! {
//         impl_binary_op_traits!(op_trait, op_fn,
//              GenericSignedBigNum<'a, E>, self, self.0, &self.0,
//              GenericSignedBigNum<'a, E>, rhs,  rhs.0,  &rhs.0);
//         impl_binary_assign_op_trait!(op_trait, op_fn, GenericSignedBigNum<'a, E>, rhs, rhs.0, &rhs.0);
//         impl_binary_assign_ref_op_trait!(op_trait, op_fn);
//     }
// }

// impl_pow_traits!(
//     GenericSignedBigNum<'a, E::Unsigned>,
//     exponent,
//     exponent.0,
//     &exponent.0
// );
// impl_pow_traits_for_ref!(
//     GenericSignedBigNum<'a, E::Unsigned>,
//     exponent,
//     exponent.0,
//     &exponent.0
// );
// duplicate_uprims! { paste! {
//     impl_pow_traits!(prim, exponent, exponent, exponent);
// } }
