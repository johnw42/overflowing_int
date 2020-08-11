use num_bigint::{BigInt, BigUint};
use num_integer::{Integer, Roots};
pub use num_traits::{Num, One, Signed, ToPrimitive, Zero};
use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

pub use num_bigint::{ParseBigIntError, Sign, TryFromBigIntError};
use std::fmt::{Display, Formatter};

type SmallInt = i128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CBigInt {
    Small(SmallInt),
    Positive(BigUint),
    Negative(BigUint),
}

impl CBigInt {
    /// Creates and initializes a BigInt.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    pub fn new(sign: Sign, digits: Vec<u32>) -> CBigInt {
        if sign == Sign::NoSign {
            return CBigInt::Small(0);
        }
        if digits.len() <= 4 {
            let mut value = 0;
            for &digit in &digits {
                value = (value << 32) | digit as i128;
            }
            if value >= 0 {
                if sign == Sign::Minus {
                    value = -value;
                }
                return CBigInt::Small(value);
            }
        }
        let magnitude = BigUint::new(digits);
        if sign == Sign::Plus {
            CBigInt::Positive(magnitude)
        } else {
            CBigInt::Negative(magnitude)
        }
    }

    #[inline]
    pub fn from_bigint(data: BigInt) -> CBigInt {
        match data.to_i128() {
            Some(value) => CBigInt::Small(value),
            None => {
                let (sign, data) = data.into_parts();
                Self::from_biguint(sign, data)
            }
        }
    }

    /// Creates and initializes a `BigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    pub fn from_biguint(sign: Sign, data: BigUint) -> CBigInt {
        match sign {
            Sign::NoSign => CBigInt::Small(0),
            Sign::Plus => match data.to_i128() {
                Some(value) => CBigInt::Small(value),
                None => CBigInt::Positive(data),
            },
            Sign::Minus => match data.to_i128() {
                Some(value) => CBigInt::Small(-value),
                None => CBigInt::Negative(data),
            },
        }
    }

    /// Creates and initializes a `BigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    pub fn from_slice(sign: Sign, slice: &[u32]) -> CBigInt {
        Self::new(sign, Vec::from(slice))
    }

    /// Reinitializes a `BigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    #[inline]
    pub fn assign_from_slice(&mut self, sign: Sign, slice: &[u32]) {
        *self = Self::from_slice(sign, slice);
    }

    /// Creates and initializes a `BigInt`.
    ///
    /// The bytes are in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, Sign};
    ///
    /// assert_eq!(BigInt::from_bytes_be(Sign::Plus, b"A"),
    ///            BigInt::parse_bytes(b"65", 10).unwrap());
    /// assert_eq!(BigInt::from_bytes_be(Sign::Plus, b"AA"),
    ///            BigInt::parse_bytes(b"16705", 10).unwrap());
    /// assert_eq!(BigInt::from_bytes_be(Sign::Plus, b"AB"),
    ///            BigInt::parse_bytes(b"16706", 10).unwrap());
    /// assert_eq!(BigInt::from_bytes_be(Sign::Plus, b"Hello world!"),
    ///            BigInt::parse_bytes(b"22405534230753963835153736737", 10).unwrap());
    /// ```
    #[inline]
    pub fn from_bytes_be(sign: Sign, bytes: &[u8]) -> CBigInt {
        Self::from_biguint(sign, BigUint::from_bytes_be(bytes))
    }

    /// Creates and initializes a `BigInt`.
    ///
    /// The bytes are in little-endian byte order.
    #[inline]
    pub fn from_bytes_le(sign: Sign, bytes: &[u8]) -> CBigInt {
        Self::from_biguint(sign, BigUint::from_bytes_le(bytes))
    }

    /// Creates and initializes a `BigInt` from an array of bytes in
    /// two's complement binary representation.
    ///
    /// The digits are in big-endian base 2<sup>8</sup>.
    #[inline]
    pub fn from_signed_bytes_be(digits: &[u8]) -> CBigInt {
        Self::from_bigint(BigInt::from_signed_bytes_be(digits))
    }

    /// Creates and initializes a `BigInt` from an array of bytes in two's complement.
    ///
    /// The digits are in little-endian base 2<sup>8</sup>.
    #[inline]
    pub fn from_signed_bytes_le(digits: &[u8]) -> CBigInt {
        Self::from_bigint(BigInt::from_signed_bytes_le(digits))
    }

    /// Creates and initializes a `BigInt`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, ToBigInt};
    ///
    /// assert_eq!(BigInt::parse_bytes(b"1234", 10), ToBigInt::to_bigint(&1234));
    /// assert_eq!(BigInt::parse_bytes(b"ABCD", 16), ToBigInt::to_bigint(&0xABCD));
    /// assert_eq!(BigInt::parse_bytes(b"G", 16), None);
    /// ```
    #[inline]
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Option<CBigInt> {
        BigInt::parse_bytes(buf, radix).map(Self::from_bigint)
    }

    /// Creates and initializes a `BigInt`. Each u8 of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in big-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, Sign};
    ///
    /// let inbase190 = vec![15, 33, 125, 12, 14];
    /// let a = BigInt::from_radix_be(Sign::Minus, &inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_be(190), (Sign:: Minus, inbase190));
    /// ```
    pub fn from_radix_be(sign: Sign, buf: &[u8], radix: u32) -> Option<CBigInt> {
        BigInt::from_radix_be(sign, buf, radix).map(Self::from_bigint)
    }

    /// Creates and initializes a `BigInt`. Each u8 of the input slice is
    /// interpreted as one digit of the number
    /// and must therefore be less than `radix`.
    ///
    /// The bytes are in little-endian byte order.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, Sign};
    ///
    /// let inbase190 = vec![14, 12, 125, 33, 15];
    /// let a = BigInt::from_radix_be(Sign::Minus, &inbase190, 190).unwrap();
    /// assert_eq!(a.to_radix_be(190), (Sign::Minus, inbase190));
    /// ```
    pub fn from_radix_le(sign: Sign, buf: &[u8], radix: u32) -> Option<CBigInt> {
        BigInt::from_radix_le(sign, buf, radix).map(Self::from_bigint)
    }

    /// Returns the sign and the byte representation of the `BigInt` in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{ToBigInt, Sign};
    ///
    /// let i = -1125.to_bigint().unwrap();
    /// assert_eq!(i.to_bytes_be(), (Sign::Minus, vec![4, 101]));
    /// ```
    #[inline]
    pub fn to_bytes_be(&self) -> (Sign, Vec<u8>) {
        self.to_bigint().to_bytes_be()
    }

    /// Returns the sign and the byte representation of the `BigInt` in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{ToBigInt, Sign};
    ///
    /// let i = -1125.to_bigint().unwrap();
    /// assert_eq!(i.to_bytes_le(), (Sign::Minus, vec![101, 4]));
    /// ```
    #[inline]
    pub fn to_bytes_le(&self) -> (Sign, Vec<u8>) {
        self.to_bigint().to_bytes_le()
    }

    /// Returns the sign and the `u32` digits representation of the `BigInt` ordered least
    /// significant digit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, Sign};
    ///
    /// assert_eq!(BigInt::from(-1125).to_u32_digits(), (Sign::Minus, vec![1125]));
    /// assert_eq!(BigInt::from(4294967295u32).to_u32_digits(), (Sign::Plus, vec![4294967295]));
    /// assert_eq!(BigInt::from(4294967296u64).to_u32_digits(), (Sign::Plus, vec![0, 1]));
    /// assert_eq!(BigInt::from(-112500000000i64).to_u32_digits(), (Sign::Minus, vec![830850304, 26]));
    /// assert_eq!(BigInt::from(112500000000i64).to_u32_digits(), (Sign::Plus, vec![830850304, 26]));
    /// ```
    #[inline]
    pub fn to_u32_digits(&self) -> (Sign, Vec<u32>) {
        todo!()
    }

    /// Returns the two's-complement byte representation of the `BigInt` in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::ToBigInt;
    ///
    /// let i = -1125.to_bigint().unwrap();
    /// assert_eq!(i.to_signed_bytes_be(), vec![251, 155]);
    /// ```
    #[inline]
    pub fn to_signed_bytes_be(&self) -> Vec<u8> {
        todo!()
    }

    /// Returns the two's-complement byte representation of the `BigInt` in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::ToBigInt;
    ///
    /// let i = -1125.to_bigint().unwrap();
    /// assert_eq!(i.to_signed_bytes_le(), vec![155, 251]);
    /// ```
    #[inline]
    pub fn to_signed_bytes_le(&self) -> Vec<u8> {
        todo!()
    }

    /// Returns the integer formatted as a string in the given radix.
    /// `radix` must be in the range `2...36`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigInt;
    ///
    /// let i = BigInt::parse_bytes(b"ff", 16).unwrap();
    /// assert_eq!(i.to_str_radix(16), "ff");
    /// ```
    #[inline]
    pub fn to_str_radix(&self, radix: u32) -> String {
        todo!()
    }

    /// Returns the integer in the requested base in big-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based u8 number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, Sign};
    ///
    /// assert_eq!(BigInt::from(-0xFFFFi64).to_radix_be(159),
    ///            (Sign::Minus, vec![2, 94, 27]));
    /// // 0xFFFF = 65535 = 2*(159^2) + 94*159 + 27
    /// ```
    #[inline]
    pub fn to_radix_be(&self, radix: u32) -> (Sign, Vec<u8>) {
        todo!()
    }

    /// Returns the integer in the requested base in little-endian digit order.
    /// The output is not given in a human readable alphabet but as a zero
    /// based u8 number.
    /// `radix` must be in the range `2...256`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, Sign};
    ///
    /// assert_eq!(BigInt::from(-0xFFFFi64).to_radix_le(159),
    ///            (Sign::Minus, vec![27, 94, 2]));
    /// // 0xFFFF = 65535 = 27 + 94*159 + 2*(159^2)
    /// ```
    #[inline]
    pub fn to_radix_le(&self, radix: u32) -> (Sign, Vec<u8>) {
        todo!()
    }

    /// Returns the sign of the `BigInt` as a `Sign`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, Sign};
    /// use num_traits::Zero;
    ///
    /// assert_eq!(BigInt::from(1234).sign(), Sign::Plus);
    /// assert_eq!(BigInt::from(-4321).sign(), Sign::Minus);
    /// assert_eq!(BigInt::zero().sign(), Sign::NoSign);
    /// ```
    #[inline]
    pub fn sign(&self) -> Sign {
        match self {
            Self::Small(n) => {
                if *n > 0 {
                    Sign::Plus
                } else if *n < 0 {
                    Sign::Minus
                } else {
                    Sign::NoSign
                }
            }
            Self::Positive(_) => Sign::Plus,
            Self::Negative(_) => Sign::Minus,
        }
    }

    /// Returns the magnitude of the `BigInt` as a `BigUint`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, BigUint};
    /// use num_traits::Zero;
    ///
    /// assert_eq!(BigInt::from(1234).magnitude(), &BigUint::from(1234u32));
    /// assert_eq!(BigInt::from(-4321).magnitude(), &BigUint::from(4321u32));
    /// assert!(BigInt::zero().magnitude().is_zero());
    /// ```
    #[inline]
    pub fn magnitude(&self) -> Cow<BigUint> {
        match self {
            Self::Small(n) => Cow::Owned(BigInt::from(*n).into_parts().1),
            Self::Positive(mag) => Cow::Borrowed(mag),
            Self::Negative(mag) => Cow::Borrowed(mag),
        }
    }

    /// Convert this `BigInt` into its `Sign` and `BigUint` magnitude,
    /// the reverse of `BigInt::from_biguint`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::{BigInt, BigUint, Sign};
    /// use num_traits::Zero;
    ///
    /// assert_eq!(BigInt::from(1234).into_parts(), (Sign::Plus, BigUint::from(1234u32)));
    /// assert_eq!(BigInt::from(-4321).into_parts(), (Sign::Minus, BigUint::from(4321u32)));
    /// assert_eq!(BigInt::zero().into_parts(), (Sign::NoSign, BigUint::zero()));
    /// ```
    #[inline]
    pub fn into_parts(self) -> (Sign, BigUint) {
        todo!()
    }

    /// Determines the fewest bits necessary to express the `BigInt`,
    /// not including the sign.
    #[inline]
    pub fn bits(&self) -> u64 {
        todo!()
    }

    /// Converts this `CBigInt` into a `BigInt`.
    #[inline]
    fn into_bigint(self) -> BigInt {
        match self {
            Self::Small(n) => BigInt::from(n),
            Self::Positive(uint) => BigInt::from_biguint(Sign::Plus, uint),
            Self::Negative(uint) => BigInt::from_biguint(Sign::Minus, uint),
        }
    }

    /// Converts this `CBigInt` into a `BigInt`.
    #[inline]
    pub fn to_bigint(&self) -> BigInt {
        self.clone().into_bigint()
    }

    /// Converts this `BigInt` into a `BigUint`, if it's not negative.
    pub fn to_biguint(&self) -> Option<BigUint> {
        match self {
            Self::Small(n) => {
                if *n >= 0 {
                    Some(BigUint::from(*n as u128))
                } else {
                    None
                }
            }
            Self::Positive(uint) => Some(uint.clone()),
            Self::Negative(_) => None,
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
        if let Self::Small(a) = &self {
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
        if let Self::Positive(uint) = self {
            if let Ok(exponent) = Cow::<BigUint>::try_from(exponent) {
                if let Ok(modulus) = Cow::<BigUint>::try_from(modulus) {
                    return uint.modpow(exponent.borrow(), modulus.borrow()).into();
                }
            }
        }
        BigInt::from(self.clone())
            .modpow(&exponent.clone().into(), &modulus.clone().into())
            .into()
    }

    /// Returns the number of least-significant bits that are zero,
    /// or `None` if the entire number is zero.
    pub fn trailing_zeros(&self) -> Option<u64> {
        todo!()
    }
}

impl Default for CBigInt {
    fn default() -> Self {
        CBigInt::Small(0)
    }
}

impl Display for CBigInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Small(n) => write!(f, "{}", n),
            Self::Positive(n) => write!(f, "{}", n),
            Self::Negative(n) => write!(f, "-{}", n),
        }
    }
}

impl From<BigInt> for CBigInt {
    fn from(value: BigInt) -> Self {
        Self::from_bigint(value)
    }
}

impl From<BigUint> for CBigInt {
    fn from(value: BigUint) -> Self {
        Self::from_biguint(Sign::Plus, value)
    }
}

impl From<CBigInt> for BigInt {
    fn from(value: CBigInt) -> Self {
        value.into_bigint()
    }
}

impl<'a> TryFrom<&'a CBigInt> for Cow<'a, BigUint> {
    type Error = ();
    fn try_from(value: &'a CBigInt) -> Result<Self, Self::Error> {
        match value {
            CBigInt::Small(i) => BigUint::try_from(*i).map_err(|_| ()).map(Cow::Owned),
            CBigInt::Positive(uint) => Ok(Cow::Borrowed(uint)),
            CBigInt::Negative(_) => Err(()),
        }
    }
}

impl Zero for CBigInt {
    fn zero() -> Self {
        CBigInt::from(0)
    }

    fn is_zero(&self) -> bool {
        if let Self::Small(n) = self {
            n.is_zero()
        } else {
            false
        }
    }
}

impl One for CBigInt {
    fn one() -> Self {
        CBigInt::from(1)
    }

    fn is_one(&self) -> bool {
        if let Self::Small(n) = self {
            n.is_one()
        } else {
            false
        }
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
        match self {
            Self::Small(a) => {
                if let (b, false) = a.overflowing_abs() {
                    b.into()
                } else {
                    BigInt::from(*a).abs().into()
                }
            }
            Self::Positive(a) => Self::Positive(a.clone()),
            Self::Negative(a) => Self::Positive(a.clone()),
        }
    }

    fn abs_sub(&self, other: &Self) -> Self {
        self.sub(other).abs()
    }

    fn signum(&self) -> Self {
        match self {
            Self::Small(a) => a.signum().into(),
            Self::Positive(_) => 1.into(),
            Self::Negative(_) => (-1).into(),
        }
    }

    fn is_positive(&self) -> bool {
        match self {
            Self::Small(a) => a.is_positive(),
            Self::Positive(_) => true,
            Self::Negative(_) => false,
        }
    }

    fn is_negative(&self) -> bool {
        match self {
            Self::Small(a) => a.is_negative(),
            Self::Negative(_) => true,
            Self::Positive(_) => false,
        }
    }
}

impl PartialOrd for CBigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CBigInt {
    fn cmp(&self, other: &Self) -> Ordering {
        if *self == *other {
            Ordering::Equal
        } else {
            match (self, other) {
                (Self::Small(a), Self::Small(b)) => a.cmp(b),
                (Self::Positive(a), Self::Positive(b)) => a.cmp(b),
                (Self::Negative(a), Self::Negative(b)) => b.cmp(a),
                (Self::Positive(_), _) => Ordering::Greater,
                (_, Self::Positive(_)) => Ordering::Less,
                (Self::Negative(_), _) => Ordering::Less,
                (_, Self::Negative(_)) => Ordering::Greater,
            }
        }
    }
}

impl Integer for CBigInt {
    fn div_floor(&self, other: &Self) -> Self {
        unimplemented!()
    }

    fn mod_floor(&self, other: &Self) -> Self {
        unimplemented!()
    }

    fn gcd(&self, other: &Self) -> Self {
        unimplemented!()
    }

    fn lcm(&self, other: &Self) -> Self {
        unimplemented!()
    }

    fn divides(&self, other: &Self) -> bool {
        unimplemented!()
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        unimplemented!()
    }

    fn is_even(&self) -> bool {
        unimplemented!()
    }

    fn is_odd(&self) -> bool {
        unimplemented!()
    }

    fn div_rem(&self, other: &Self) -> (Self, Self) {
        unimplemented!()
    }
}

impl Roots for CBigInt {
    fn nth_root(&self, n: u32) -> Self {
        if let Self::Small(a) = self {
            return a.nth_root(n).into();
        }
        Self::from_biguint(self.sign(), (self.magnitude().borrow() as &BigUint).sqrt())
    }
}

impl Neg for CBigInt {
    type Output = CBigInt;

    fn neg(self) -> Self::Output {
        if let Self::Small(a) = self {
            if let (b, false) = a.overflowing_neg() {
                return Self::Small(b);
            }
        }
        BigInt::from(self).neg().into()
    }
}

impl Neg for &CBigInt {
    type Output = CBigInt;

    fn neg(self) -> Self::Output {
        self.clone().neg()
    }
}

impl Not for CBigInt {
    type Output = CBigInt;

    fn not(self) -> Self::Output {
        if let Self::Small(a) = self {
            return Self::Small(a.not());
        }
        BigInt::from(self).not().into()
    }
}

impl Not for &CBigInt {
    type Output = CBigInt;

    fn not(self) -> Self::Output {
        self.clone().not()
    }
}

macro_rules! ref_op {
    ($trait:ident<$rhs_type:ty> for $lhs_type:ty, $op:ident) => {
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

macro_rules! prims_and_ops {
    (@prims $($prim:tt),*; @ops $($op:tt),*;) => {
        impl ToPrimitive for CBigInt {
            $(to_prim_method!($prim);)*
        }
        $(each_prim!($prim);)*
        $(each_op!($op);)*
        iter_ops_then_prims!(@prims {$($prim),*}; @ops $($op),*;);
    };
}

macro_rules! iter_ops_then_prims {
    (@prims $prims:tt; @ops $($op:tt),*;) => {
        $(iter_prims_for_op!(@prims $prims; $op);)*
    };
}

macro_rules! iter_prims_for_op {
    (@prims {$($prim:tt),*}; $op:tt) => {
        $(each_prim_and_op!($prim, $op);)*
    };
}

macro_rules! each_prim {
    ([@int_prim, $prim:ident, $to_prim:ident]) => {
        impl From<$prim> for CBigInt {
            fn from(value: $prim) -> Self {
                if let Ok(promoted) = SmallInt::try_from(value) {
                    CBigInt::Small(promoted)
                } else {
                    BigInt::from(value).into()
                }
            }
        }
        impl TryFrom<CBigInt> for $prim {
            type Error = (); // TODO
            fn try_from(value: CBigInt) -> Result<Self, ()> {
                if let CBigInt::Small(value) = value {
                    $prim::try_from(value).map_err(|_| ())
                } else {
                    $prim::try_from(BigInt::from(value)).map_err(|_| ())
                }
            }
        }
    };
    ([@float_prim, $prim:ident, $to_prim:ident]) => {};
}

macro_rules! to_prim_method {
    ([@int_prim, $prim:ident, $to_prim:ident]) => {
        fn $to_prim(&self) -> Option<$prim> {
            if let CBigInt::Small(value) = self {
                $prim::try_from(*value).ok()
            } else {
                $prim::try_from(BigInt::from(self.clone())).ok()
            }
        }
    };
    ([@float_prim, $prim:ident, $to_prim:ident]) => {
        fn $to_prim(&self) -> Option<$prim> {
            if let CBigInt::Small(value) = self {
                Some(*value as $prim)
            } else {
                BigInt::from(self.clone()).$to_prim()
            }
        }
    };
}

macro_rules! each_op {
    ([@arith_op, $trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident, $overflowing_op:ident]) => {
        impl $trait for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: Self) -> Self::Output {
                if let (Self::Small(a), Self::Small(b)) = (&self, &rhs) {
                    if let (c, false) = a.$overflowing_op(*b) {
                        return c.into();
                    }
                }
                BigInt::from(self).$op(BigInt::from(rhs)).into()
            }
        }
        assign_op!($trait, $op, $assign_trait, $assign_op);
        ref_op!($trait<CBigInt> for CBigInt, $op);
    };
    ([@shift_op, $trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident, $overflowing_op:ident]) => {
        assign_op!($trait, $op, $assign_trait, $assign_op);
    };
    ([@bit_op, $trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]) => {
        impl $trait for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: Self) -> Self::Output {
                if let (Self::Small(a), Self::Small(b)) = (&self, &rhs) {
                    return a.$op(*b).into();
                }
                BigInt::from(self).$op(BigInt::from(rhs)).into()
            }
        }
        assign_op!($trait, $op, $assign_trait, $assign_op);
        ref_op!($trait<CBigInt> for CBigInt, $op);
    };
}

macro_rules! assign_op {
    ($trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident) => {
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
    (
        [@int_prim, $prim:ident, $to_prim:ident],
        [@arith_op, $trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident, $overflowing_op:ident]
    ) => {
        impl $trait<$prim> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: $prim) -> Self::Output {
                if let CBigInt::Small(prim) = &self {
                    if let Ok(promoted) = SmallInt::try_from(rhs) {
                        if let (result, false) = prim.$overflowing_op(promoted) {
                            return CBigInt::Small(result);
                        }
                    }
                }
                BigInt::from(self).$op(rhs).into()
            }
        }
        impl $trait<CBigInt> for $prim {
            type Output = CBigInt;
            fn $op(self, rhs: CBigInt) -> Self::Output {
                if let CBigInt::Small(prim) = &rhs {
                    if let Ok(promoted) = SmallInt::try_from(self) {
                        if let (result, false) = promoted.$overflowing_op(*prim) {
                            return CBigInt::Small(result);
                        }
                    }
                }
                self.$op(BigInt::from(rhs)).into()
            }
        }
        ref_op!($trait<$prim> for CBigInt, $op);
        ref_op!($trait<CBigInt> for $prim, $op);
    };
    (
        [@int_prim, $prim:ident, $to_prim:ident],
        [@shift_op, $trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident, $overflowing_op:ident]
    ) => {
        impl $trait<$prim> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: $prim) -> Self::Output {
                if let CBigInt::Small(lhs) = &self {
                    if let Ok(rhs) = u32::try_from(rhs) {
                        if let (result, false) = lhs.$overflowing_op(rhs) {
                            return CBigInt::Small(result);
                        }
                    }
                }
                BigInt::from(self).$op(rhs).into()
            }
        }
        ref_op!($trait<$prim> for CBigInt, $op);
    };
    ($prim:tt, [@bit_op, $trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]) => {};
    ([@float_prim $(, $_1:tt)*], $_2:tt) => {};
}

prims_and_ops! {
    @prims
    [@int_prim, i8, to_i8],
    [@int_prim, i16, to_i16],
    [@int_prim, i32, to_i32],
    [@int_prim, i64, to_i64],
    [@int_prim, i128, to_i128],
    [@int_prim, isize, to_isize],
    [@int_prim, u8, to_u8],
    [@int_prim, u16, to_u16],
    [@int_prim, u32, to_u32],
    [@int_prim, u64, to_u64],
    [@int_prim, u128, to_u128],
    [@int_prim, usize, to_usize],
    [@float_prim, f32, to_f32],
    [@float_prim, f64, to_f64];
    @ops
    [@arith_op, Add, add, AddAssign, add_assign, overflowing_add],
    [@arith_op, Sub, sub, SubAssign, sub_assign, overflowing_sub],
    [@arith_op, Mul, mul, MulAssign, mul_assign, overflowing_mul],
    [@arith_op, Div, div, DivAssign, div_assign, overflowing_div],
    [@arith_op, Rem, rem, RemAssign, rem_assign, overflowing_div],
    [@shift_op, Shl, shl, ShlAssign, shl_assign, overflowing_shl],
    [@shift_op, Shr, shr, ShrAssign, shr_assign, overflowing_shr],
    [@bit_op, BitAnd, bitand, BitAndAssign, bitand_assign],
    [@bit_op, BitOr, bitor, BitOrAssign, bitor_assign],
    [@bit_op, BitXor, bitxor, BitXorAssign, bitxor_assign];
}
