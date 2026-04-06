use crate::big_number::{BigNumber, BigNumberDigits};
use crate::generic_bignum::GenericBigNum;
use crate::generic_bignum::encoding::{Decode, Decoded, Encoding};
use crate::small_num::SmallNumber;
use crate::{duplicate_arith_ops, duplicate_bit_ops, duplicate_prims, duplicate_shift_ops};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::Zero;
use paste::paste;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

/// A signed big integer type that can be used with any encoding that implements `Encoding` with `Big = BigInt`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GenericBigInt<'a, E: Encoding<'a>>(pub(crate) GenericBigNum<'a, E>);

impl<'a, E: Encoding<'a, Big = BigInt>> GenericBigInt<'a, E> {
    /// Returns the magnitude of the as a `BigUint`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_bigint::CowBigInt;
    /// use num_traits::Zero;
    /// use std::borrow::Borrow;
    /// use num_bigint::BigUint;
    ///
    /// assert_eq!(CowBigInt::from(1234).magnitude(), BigUint::from(1234u32));
    /// assert_eq!(CowBigInt::from(-4321).magnitude(), BigUint::from(4321u32));
    /// assert!(CowBigInt::zero().magnitude().is_zero());
    /// ```
    pub fn magnitude(&self) -> BigUint {
        // TODO change return type
        self.0.with_decoded(|encoded| match encoded {
            Decoded::Small(n) => n.to_bigint().magnitude().clone(),
            Decoded::Big(n) => n.magnitude().clone(),
        })
    }

    /// Creates and initializes a E::Big.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    pub fn new(sign: Sign, digits: Vec<u32>) -> Self {
        Self(if sign == Sign::NoSign {
            GenericBigNum::from_small(E::Small::zero())
        } else {
            GenericBigNum::from_big(E::Big::new(sign, digits))
        })
    }

    /// Creates and initializes a bigint.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    pub(crate) fn from_biguint(sign: Sign, data: BigUint) -> Self {
        Self(GenericBigNum::from_big(E::Big::from_biguint(sign, data)))
    }

    /// Creates and initializes a bigint.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    pub fn from_slice(sign: Sign, slice: &[u32]) -> Self {
        Self(GenericBigNum::from_big(E::Big::from_slice(sign, slice)))
    }

    /// Reinitializes a bigint.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
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
    pub fn from_bytes_be(sign: Sign, bytes: &[u8]) -> Self
    where
        E::Small: Neg<Output = E::Small>,
    {
        Self(
            if let Some(from_bytes) = SmallNumber::from_bytes_be(bytes) {
                GenericBigNum::from_small(from_bytes)
            } else {
                GenericBigNum::from_big(E::Big::from_bytes_be(sign, bytes))
            },
        )
    }

    /// Creates and initializes a bigint.
    ///
    /// The bytes are in little-endian byte order.
    pub fn from_bytes_le(sign: Sign, bytes: &[u8]) -> Self
    where
        E::Small: Neg<Output = E::Small>,
    {
        Self(
            if let Some(from_bytes) = SmallNumber::from_bytes_le(bytes) {
                GenericBigNum::from_small(from_bytes)
            } else {
                GenericBigNum::from_big(E::Big::from_bytes_le(sign, bytes))
            },
        )
    }

    /// Creates and initializes a bigint from an array of bytes in
    /// two's complement binary representation.
    ///
    /// The digits are in big-endian base 2<sup>8</sup>.
    pub fn from_signed_bytes_be(digits: &[u8]) -> Self {
        Self(GenericBigNum::from_big(E::Big::from_signed_bytes_be(
            digits,
        )))
    }

    /// Creates and initializes a bigint from an array of bytes in two's complement.
    ///
    /// The digits are in little-endian base 2<sup>8</sup>.
    pub fn from_signed_bytes_le(digits: &[u8]) -> Self {
        Self(GenericBigNum::from_big(E::Big::from_signed_bytes_le(
            digits,
        )))
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
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self> {
        E::Big::parse_bytes(buf, radix)
            .map(GenericBigNum::from_big)
            .map(Self)
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
    pub fn from_radix_be(sign: Sign, buf: &[u8], radix: u32) -> Option<Self> {
        E::Big::from_radix_be(sign, buf, radix)
            .map(GenericBigNum::from_big)
            .map(Self)
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
    pub fn from_radix_le(sign: Sign, buf: &[u8], radix: u32) -> Option<Self> {
        E::Big::from_radix_le(sign, buf, radix)
            .map(GenericBigNum::from_big)
            .map(Self)
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
    pub fn to_u32_digits(&self) -> (Sign, Vec<u32>) {
        self.0.with_big_cow(|big| big.as_ref().to_u32_digits())
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
    pub fn to_str_radix(&self, radix: u32) -> String {
        self.0.with_big_cow(|big| big.as_ref().to_str_radix(radix))
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
    pub fn to_radix_le(&self, radix: u32) -> (Sign, Vec<u8>) {
        self.0.with_big_cow(|big| big.as_ref().to_radix_le(radix))
    }

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

    pub fn into_parts(self) -> (Sign, BigUint) {
        self.0.into_bigint().into_parts()
    }

    pub fn bit(&self, bit: u64) -> bool {
        self.0.bit(bit)
    }

    pub fn bits(&self) -> u64 {
        self.0.bits()
    }

    pub fn to_biguint(&self) -> Option<BigUint> {
        self.0.with_big_cow(|big| big.as_ref().to_biguint())
    }

    pub fn checked_add(&self, v: &'a Self) -> Option<Self> {
        self.0.checked_add(&v.0).map(Self)
    }

    pub fn checked_sub(&self, v: &'a Self) -> Option<Self> {
        self.0.checked_sub(&v.0).map(Self)
    }

    pub fn checked_mul(&self, v: &'a Self) -> Option<Self> {
        self.0.checked_mul(&v.0).map(Self)
    }

    pub fn checked_div(&self, v: &'a Self) -> Option<Self> {
        self.0.checked_div(&v.0).map(Self)
    }

    pub fn pow(&self, exponent: u32) -> Self {
        Self(self.0.pow(exponent))
    }

    pub fn modpow(&self, exponent: &'a Self, modulus: &'a Self) -> Self {
        Self(self.0.modpow(&exponent.0, &modulus.0))
    }

    pub fn sqrt(&self) -> Self {
        Self(self.0.sqrt())
    }

    pub fn cbrt(&self) -> Self {
        Self(self.0.cbrt())
    }

    pub fn nth_root(&self, n: u32) -> Self {
        Self(self.0.nth_root(n))
    }

    pub fn trailing_zeros(&self) -> Option<u64> {
        self.0.trailing_zeros()
    }

    pub fn to_u64_digits(&self) -> (Sign, Vec<u64>) {
        self.0.with_big_cow(|big| big.as_ref().to_u64_digits())
    }

    pub fn iter_u32_digits(&self) -> impl BigNumberDigits<'_, u32> {
        self.0.iter_u32_digits()
    }

    pub fn iter_u64_digits(&self) -> impl BigNumberDigits<'_, u64> {
        self.0.iter_u64_digits()
    }

    pub fn modinv(&self, modulus: &'a Self) -> Option<Self> {
        self.0.modinv(&modulus.0).map(Self)
    }

    pub fn set_bit(&'a mut self, bit: u64, value: bool) {
        self.0.set_bit(bit, value);
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> From<GenericBigNum<'a, E>> for GenericBigInt<'a, E> {
    fn from(value: GenericBigNum<'a, E>) -> Self {
        Self(value)
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> From<BigInt> for GenericBigInt<'a, E> {
    fn from(value: BigInt) -> Self {
        Self(GenericBigNum::from_big(value))
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> From<GenericBigInt<'a, E>> for GenericBigNum<'a, E> {
    fn from(value: GenericBigInt<'a, E>) -> Self {
        value.0
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Decode<'a, E::Small> for GenericBigInt<'a, E>
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

impl<'a, E: Encoding<'a, Big = BigInt>> Encoding<'a> for GenericBigInt<'a, E>
where
    E::Small: SmallNumber<Big = E::Big>,
    E::Big: BigNumber,
{
    type Small = E::Small;
    type Big = E::Big;
    //type Repr = E::Repr;

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
        impl<'a, E: Encoding<'a, Big = BigInt>> $trait<$rhs_type> for $lhs_type {
            type Output = GenericBigInt<'a, E>;

            fn $op_fn($lhs_param: Self, $rhs_param: $rhs_type) -> GenericBigInt<'a, E> {
                GenericBigInt($lhs_expr.$op_fn($rhs_expr))
            }
        }

        impl<'a, E: Encoding<'a, Big = BigInt>> $trait<&$rhs_type> for $lhs_type {
            type Output = GenericBigInt<'a, E>;

            fn $op_fn($lhs_param: Self, $rhs_param: &$rhs_type) -> GenericBigInt<'a, E> {
                GenericBigInt($lhs_expr.$op_fn($rhs_ref_expr))
            }
        }

        impl<'a, E: Encoding<'a, Big = BigInt>> $trait<$rhs_type> for &$lhs_type {
            type Output = GenericBigInt<'a, E>;

            fn $op_fn($lhs_param: Self, $rhs_param: $rhs_type) -> GenericBigInt<'a, E> {
                GenericBigInt($lhs_ref_expr.$op_fn($rhs_expr))
            }
        }

        impl<'a, E: Encoding<'a, Big = BigInt>> $trait<&$rhs_type> for &$lhs_type {
            type Output = GenericBigInt<'a, E>;

            fn $op_fn($lhs_param: Self, $rhs_param: &$rhs_type) -> GenericBigInt<'a, E> {
                GenericBigInt($lhs_ref_expr.$op_fn($rhs_ref_expr))
            }
        }
    };
}

macro_rules! impl_binary_assign_op_trait {
    ($trait:ident, $op_fn:ident, $rhs_type:ty, $rhs_param:ident, $rhs_expr:expr, $rhs_ref_expr:expr) => {
        paste! {
            impl<'a, E: Encoding<'a, Big = BigInt>> [<$trait Assign>]<$rhs_type> for GenericBigInt<'a, E> {
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
            impl<'a, E: Encoding<'a, Big = BigInt>> [<$trait Assign>]<&GenericBigInt<'a, E>> for GenericBigInt<'a, E> {
                fn [<$op_fn _assign>](&mut self, rhs: &GenericBigInt<'a, E>) {
                    self.0.[<$op_fn _assign>](&rhs.0)
                }
            }
        }
    };
}

duplicate_prims! {
    impl<'a, E: Encoding<'a, Big = BigInt>> From<prim> for GenericBigInt<'a, E> {
        fn from(value: prim) -> Self {
            Self(GenericBigNum::from(value))
        }
    }
}

// Implementations of numeric traits for `GenericBigInt`.  The number of
// implementations is quite large, so we use macros to generate them.  The
// reason for implementing so many variants is to allow `GenericBigInt` to serve
// as a drop-in replacement for `BigInt`, which implements the same traits.
duplicate_arith_ops! {
    paste! {
        impl_binary_op_traits!(op_trait, op_fn,
             GenericBigInt<'a, E>, self, self.0, &self.0,
             GenericBigInt<'a, E>, rhs,  rhs.0,  &rhs.0);
        impl_binary_assign_op_trait!(op_trait, op_fn, GenericBigInt<'a, E>, rhs, rhs.0, &rhs.0);
        impl_binary_assign_ref_op_trait!(op_trait, op_fn);
    }
    duplicate_prims! { paste! {
        impl_binary_op_traits!(op_trait, op_fn,
             GenericBigInt<'a, E>, self, self.0, &self.0,
             prim, rhs, rhs, rhs);
        impl_binary_op_traits!(op_trait, op_fn,
             prim, self, self, self,
             GenericBigInt<'a, E>, rhs, rhs.0, &rhs.0);
        impl_binary_assign_op_trait!(op_trait, op_fn, prim, rhs, rhs, rhs);
    } }
}
duplicate_shift_ops! {
    duplicate_prims! { paste! {
        impl_binary_op_traits!(op_trait, op_fn,
             GenericBigInt<'a, E>, self, self.0, &self.0,
             prim, rhs, rhs, rhs);
        impl_binary_assign_op_trait!(op_trait, op_fn, prim, rhs, rhs, rhs);
    } }
}
duplicate_bit_ops! {
    paste! {
        impl_binary_op_traits!(op_trait, op_fn,
             GenericBigInt<'a, E>, self, self.0, &self.0,
             GenericBigInt<'a, E>, rhs,  rhs.0,  &rhs.0);
        impl_binary_assign_op_trait!(op_trait, op_fn, GenericBigInt<'a, E>, rhs, rhs.0, &rhs.0);
        impl_binary_assign_ref_op_trait!(op_trait, op_fn);
    }
}

// TODO

// fn pow_self_and_ref_biguint(lhs: Self, rhs: &BigUint) -> Self;
// duplicate_uprims! { paste! {
//     fn [<pow_self_and_ref_ prim>](lhs: Self, rhs: &prim) -> Self;
// } }
