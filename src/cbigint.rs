use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Debug, Display, Formatter};
use std::mem::{size_of, ManuallyDrop};
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};
use std::panic::catch_unwind;

use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};
use num_integer::{Integer, Roots};
use num_traits::{Num, One, Signed, ToPrimitive, Zero};

use CBigInt::*;

use crate::Sign::*;
use crate::{Digit, Udigit};

type Accum = Digit;
type Uaccum = Udigit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CBigInt {
    Small(Digit),
    Positive(BigUint),
    Negative(BigUint),
}

pub enum GenInt {
    Big(BigInt),
    Small(Digit),
}

enum GenIntRef<'a> {
    Big(&'a BigInt),
    Small(Digit),
}

enum GenIntCow<'a> {
    Big(Cow<'a, BigInt>),
    Small(Digit),
}

impl From<CBigInt> for GenInt {
    fn from(arg: CBigInt) -> Self {
        let (sign, mag) = match arg {
            Small(value) => return GenInt::Small(value),
            Positive(mag) => (Plus, mag),
            Negative(mag) => (Minus, mag),
        };
        GenInt::Big(BigInt::from_biguint(sign, mag))
    }
}

impl<'a> From<GenInt> for GenIntCow<'a> {
    fn from(value: GenInt) -> Self {
        match value {
            GenInt::Small(x) => GenIntCow::Small(x),
            GenInt::Big(x) => GenIntCow::Big(Cow::Owned(x)),
        }
    }
}

impl<'a> From<GenIntRef<'a>> for GenIntCow<'a> {
    fn from(value: GenIntRef<'a>) -> Self {
        match value {
            GenIntRef::Small(x) => GenIntCow::Small(x),
            GenIntRef::Big(x) => GenIntCow::Big(Cow::Borrowed(x)),
        }
    }
}

// fn maybe_bigint<T>(arg: CBigInt, f: impl FnOnce(GenInt) -> T) -> T {
//     let (sign, mag) = match arg {
//         Small(value) => return f(GenInt::Small(value)),
//         Positive(mag) => (Plus, mag),
//         Negative(mag) => (Minus, mag),
//     };
//     f(GenInt::Big(BigInt::from_biguint(sign, mag)))
// }
//
// fn maybe_bigint_ref<T>(arg: &CBigInt, f: impl FnOnce(GenIntRef) -> T) -> T {
//     let (sign, mag) = match arg {
//         Small(value) => return f(GenIntRef::Small(*value)),
//         Positive(mag) => (Plus, mag),
//         Negative(mag) => (Minus, mag),
//     };
//     let bigint = ManuallyDrop::new(BigInt::from_biguint(sign, unsafe { std::ptr::read(mag) }));
//     f(GenIntRef::Big(&*bigint))
// }
//
// fn maybe_bigint_cow<T>(arg: Cow<CBigInt>, f: impl FnOnce(GenIntCow) -> T) -> T {
//     match arg {
//         Cow::Borrowed(arg) => maybe_bigint_ref(arg, f),
//         Cow::Owned(arg) => maybe_bigint(arg, f),
//     }
// }

struct GenIntFn<F>(F);
struct GenIntRefFn<F>(F);
struct GenIntCowFn<F>(F);

const DIGIT_BITS: usize = size_of::<Digit>() * 8;

// struct GenIntCowFn1<F, T>(F)
// where
//     F: FnOnce(GenIntCow) -> T;

trait CBigIntFn<T>
where
    Self: Sized,
{
    fn apply(self, arg: CBigInt) -> T;

    fn apply_ref(self, arg: &CBigInt) -> T {
        self.apply(arg.clone())
    }

    fn apply_cow(self, arg: Cow<CBigInt>) -> T {
        match arg {
            Cow::Borrowed(arg) => self.apply_ref(arg),
            Cow::Owned(arg) => self.apply(arg),
        }
    }
}

impl<F, T> CBigIntFn<T> for GenIntFn<F>
where
    F: FnOnce(GenInt) -> T,
{
    fn apply(self, arg: CBigInt) -> T {
        self.0(arg.into())
    }
}

impl<FB, FS, T> CBigIntFn<T> for GenIntFn<(FB, FS)>
where
    FB: FnOnce(BigInt) -> T,
    FS: FnOnce(Digit) -> T,
{
    fn apply(self, arg: CBigInt) -> T {
        let (fb, fs) = self.0;
        GenIntFn(|arg| match arg {
            GenInt::Small(x) => fs(x),
            GenInt::Big(x) => fb(x),
        })
        .apply(arg)
    }
}

impl<F, T> CBigIntFn<T> for GenIntRefFn<F>
where
    F: for<'a> FnOnce(GenIntRef<'a>) -> T,
{
    fn apply(self, arg: CBigInt) -> T {
        GenIntFn(|arg| {
            self.0(match &arg {
                GenInt::Small(x) => GenIntRef::Small(*x),
                GenInt::Big(x) => GenIntRef::Big(x),
            })
        })
        .apply(arg)
    }

    fn apply_ref(self, arg: &CBigInt) -> T {
        let (sign, mag) = match arg {
            Small(value) => return self.0(GenIntRef::Small(*value)),
            Positive(mag) => (Plus, mag),
            Negative(mag) => (Minus, mag),
        };
        let bigint = ManuallyDrop::new(BigInt::from_biguint(sign, unsafe { std::ptr::read(mag) }));
        self.0(GenIntRef::Big(&*bigint))
    }
}

impl<FB, FS, T> CBigIntFn<T> for GenIntRefFn<(FB, FS)>
where
    FB: for<'a> FnOnce(&'a BigInt) -> T,
    FS: FnOnce(Digit) -> T,
{
    fn apply(self, arg: CBigInt) -> T {
        let (fb, fs) = self.0;
        GenIntRefFn(|arg: GenIntRef| match arg {
            GenIntRef::Small(x) => fs(x),
            GenIntRef::Big(x) => fb(x),
        })
        .apply(arg)
    }

    fn apply_ref(self, arg: &CBigInt) -> T {
        let (fb, fs) = self.0;
        GenIntRefFn(|arg: GenIntRef| match arg {
            GenIntRef::Small(x) => fs(x),
            GenIntRef::Big(x) => fb(x),
        })
        .apply_ref(arg)
    }
}

// impl<T> CBigIntFn<T> for GenIntRefFn<(fn(&BigInt) -> T, fn(Digit) -> T)> {
//     fn apply(self, arg: CBigInt) -> T {
//         let (fb, fs) = self.0;
//         GenIntRefFn(|arg: GenIntRef| -> T {
//             match arg {
//                 GenIntRef::Small(x) => fs(x),
//                 GenIntRef::Big(x) => fb(x),
//             }
//         })
//         .apply(arg)
//     }
//
//     fn apply_ref(self, arg: &CBigInt) -> T {
//         let (fb, fs) = self.0;
//         GenIntRefFn(|arg: GenIntRef| -> T {
//             match arg {
//                 GenIntRef::Small(x) => fs(x),
//                 GenIntRef::Big(x) => fb(x),
//             }
//         })
//         .apply_ref(arg)
//     }
// }

impl<F, T> CBigIntFn<T> for GenIntCowFn<F>
where
    F: for<'a> FnOnce(GenIntCow<'a>) -> T,
{
    fn apply(self, arg: CBigInt) -> T {
        self.0(GenInt::from(arg).into())
    }

    fn apply_ref(self, arg: &CBigInt) -> T {
        GenIntRefFn(|arg: GenIntRef| -> T {
            self.0(match arg {
                GenIntRef::Small(x) => GenIntCow::Small(x),
                GenIntRef::Big(x) => GenIntCow::Big(Cow::Borrowed(x)),
            })
        })
        .apply_ref(arg)
    }
}

// impl<FB, FS, T> CBigIntFn1<T> for (FB, FS)
// where
//     FB: FnOnce(BigInt),
//     FS: FnOnce(Digit),
// {
//     fn apply_small(self, arg: CBigInt) -> T {
//         let (fb, fs) = self;
//         let (sign, mag) = match arg {
//             Small(value) => return fs(value),
//             Positive(mag) => (Plus, mag),
//             Negative(mag) => (Minus, mag),
//         };
//         fb(GenInt::Big(BigInt::from_biguint(sign, mag)))
//     }
//
//     fn apply_big(self, arg: CBigInt) -> T {
//         let (sign, mag) = match arg {
//             Small(value) => return f(GenIntRef::Small(*value)),
//             Positive(mag) => (Plus, mag),
//             Negative(mag) => (Minus, mag),
//         };
//         let bigint = ManuallyDrop::new(BigInt::from_biguint(sign, unsafe { std::ptr::read(mag) }));
//         f(GenIntRef::Big(&*bigint))
//     }
//
//     fn apply_big_ref(self, arg: &CBigInt) -> T {
//         unimplemented!()
//     }
// }

trait CBigIntFnArg {
    fn apply_to<F, T>(self, f: F) -> T
    where
        F: CBigIntFn<T>;
}

impl CBigIntFnArg for CBigInt {
    fn apply_to<F, T>(self, f: F) -> T
    where
        F: CBigIntFn<T>,
    {
        f.apply(self)
    }
}

impl CBigIntFnArg for &CBigInt {
    fn apply_to<F, T>(self, f: F) -> T
    where
        F: CBigIntFn<T>,
    {
        f.apply_ref(self)
    }
}

trait CBigIntFn2<T> {
    fn apply(arg1: CBigInt, arg2: CBigInt) -> T;
    fn apply_ref1(arg1: &CBigInt, arg2: CBigInt) -> T;
    fn apply_ref2(arg1: CBigInt, arg2: &CBigInt) -> T;
    fn apply_ref_ref(arg1: &CBigInt, arg2: &CBigInt) -> T;
}

// fn maybe_bigint_mut<T>(
//     arg: &mut CBigInt,
//     f: impl FnOnce(&mut BigInt) -> T,
//     g: impl FnOnce(&mut Digit) -> T,
// ) -> T {
//     let (sign, mag) = match arg {
//         Small(value) => return g(value),
//         Positive(mag) => (Plus, mag),
//         Negative(mag) => (Minus, mag),
//     };
//     let mut bigint = ManuallyDrop::new(BigInt::from_biguint(sign, unsafe { std::ptr::read(mag) }));
//     let result = f(&mut *bigint);
//     unsafe {
//         std::ptr::write(arg, ManuallyDrop::take(&mut bigint).into());
//     }
//     result
// }
//
// /// Calls [f] if either [lhs] or [rhs] is large, [g] if both are small.  Falls
// /// back to calling [f] if [g] returns [None].
// fn maybe_bigint_refs<T>(
//     lhs: &CBigInt,
//     rhs: &CBigInt,
//     f: impl FnOnce(BigOrSmall, BigOrSmall) -> T,
// ) -> T {
//     maybe_bigint_ref(
//         lhs,
//         |big_lhs| {
//             maybe_bigint_ref(
//                 rhs,
//                 |big_rhs| f(Cow::Borrowed(big_lhs), Cow::Borrowed(big_rhs)),
//                 |&small_rhs| f(Cow::Borrowed(big_lhs), Cow::Owned(small_rhs.into())),
//             )
//         },
//         |&small_lhs| {
//             maybe_bigint_ref(
//                 rhs,
//                 |big_rhs| f(Cow::Owned(small_lhs.into()), Cow::Borrowed(big_rhs)),
//                 |&small_rhs| {
//                     g(&small_lhs, &small_rhs).unwrap_or_else(|| {
//                         f(
//                             Cow::Owned(BigInt::from(small_lhs)),
//                             Cow::Owned(BigInt::from(small_rhs)),
//                         )
//                     })
//                 },
//             )
//         },
//     )
// }
//
// fn maybe_bigint_cows<T>(
//     lhs: Cow<CBigInt>,
//     rhs: Cow<CBigInt>,
//     f: impl FnOnce(BigOrSmall, BigOrSmall) -> T,
// ) -> T {
//     maybe_bigint_cow(lhs, |lhs| maybe_bigint_cow(rhs, |rhs| f(lhs, rhs)))
// }

impl CBigInt {
    #[inline(always)]
    pub(crate) fn from_small_int(n: Digit) -> CBigInt {
        Small(n)
    }

    /// Creates and initializes a BigInt.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    pub fn new(sign: Sign, digits: Vec<u32>) -> CBigInt {
        if sign == NoSign {
            return Small(0);
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
        let magnitude = BigUint::new(digits);
        if sign == Plus {
            Positive(magnitude)
        } else {
            Negative(magnitude)
        }
    }

    #[inline]
    pub fn from_bigint(data: BigInt) -> CBigInt {
        match data.to_i128() {
            Some(value) => value.into(),
            None => {
                let (sign, data) = data.into_parts();
                Self::from_biguint(sign, data)
            }
        }
    }

    /// Creates and initializes a `CBigInt`.
    ///
    /// The base 2<sup>32</sup> digits are ordered least significant digit first.
    pub fn from_biguint(sign: Sign, data: BigUint) -> CBigInt {
        match sign {
            NoSign => Small(0),
            Plus => match Digit::try_from(&data) {
                Ok(value) => {
                    debug_assert!(value >= 0);
                    Small(value)
                }
                Err(_) => Positive(data),
            },
            Minus => match Digit::try_from(&data) {
                Ok(value) => {
                    debug_assert!(value >= 0);
                    Small(-value)
                }
                Err(_) => Negative(data),
            },
        }
    }

    #[inline(always)]
    fn from_accum(sign: Sign, accum: Udigit) -> Option<CBigInt> {
        let accum = accum as Digit;
        if accum >= 0 {
            Some(match sign {
                Plus => Small(accum),
                Minus => Small(-accum),
                NoSign => Small(0),
            })
        } else {
            None
        }
    }

    #[inline(always)]
    fn accum_be(bytes: &[u8]) -> Option<Udigit> {
        if bytes.len() <= 16 {
            let mut accum = 0;
            for &byte in bytes {
                accum = accum << 8 | byte as Udigit;
            }
            Some(accum)
        } else {
            None
        }
    }

    #[inline(always)]
    fn accum_le(bytes: &[u8]) -> Option<Udigit> {
        if bytes.len() <= 16 {
            let mut accum = 0;
            for (i, &byte) in bytes.iter().enumerate() {
                accum |= (byte as Udigit) << 8 * i;
            }
            Some(accum)
        } else {
            None
        }
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
        if let Some(accum) = Self::accum_be(bytes) {
            if let Some(result) = Self::from_accum(sign, accum) {
                return result;
            }
        }
        Self::from_biguint(sign, BigUint::from_bytes_be(bytes))
    }

    /// Creates and initializes a `CBigInt`.
    ///
    /// The bytes are in little-endian byte order.
    pub fn from_bytes_le(sign: Sign, bytes: &[u8]) -> CBigInt {
        if let Some(accum) = Self::accum_le(bytes) {
            if let Some(result) = Self::from_accum(sign, accum) {
                return result;
            }
        }
        Self::from_biguint(sign, BigUint::from_bytes_le(bytes))
    }

    /// Creates and initializes a `CBigInt` from an array of bytes in
    /// two's complement binary representation.
    ///
    /// The digits are in big-endian base 2<sup>8</sup>.
    pub fn from_signed_bytes_be(digits: &[u8]) -> CBigInt {
        if let Some(accum) = Self::accum_be(digits) {
            Small(accum as Digit)
        } else {
            Self::from_bigint(BigInt::from_signed_bytes_be(digits))
        }
    }

    /// Creates and initializes a `CBigInt` from an array of bytes in two's complement.
    ///
    /// The digits are in little-endian base 2<sup>8</sup>.
    pub fn from_signed_bytes_le(digits: &[u8]) -> CBigInt {
        if let Some(accum) = Self::accum_le(digits) {
            Small(accum as Digit)
        } else {
            Self::from_bigint(BigInt::from_signed_bytes_le(digits))
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
        BigInt::parse_bytes(buf, radix).map(Self::from_bigint)
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
        BigInt::from_radix_be(sign, buf, radix).map(Self::from_bigint)
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
        BigInt::from_radix_le(sign, buf, radix).map(Self::from_bigint)
    }

    fn make_accum(value: Digit) -> (Sign, Udigit) {
        if value == 0 {
            (NoSign, 0)
        } else if value >= 0 {
            (Plus, value as Udigit)
        } else if value == Digit::MIN {
            (Minus, value as Udigit)
        } else {
            (Minus, (-value) as Udigit)
        }
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
        match self {
            &Small(n) => match Self::make_accum(n) {
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
            Positive(mag) => (Plus, mag.to_bytes_be()),
            Negative(mag) => (Minus, mag.to_bytes_be()),
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
        match self {
            &Small(n) => {
                let (sign, accum) = Self::make_accum(n);
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
            Positive(mag) => (Plus, mag.to_bytes_le()),
            Negative(mag) => (Minus, mag.to_bytes_le()),
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
        match self {
            &Small(n) => match Self::make_accum(n) {
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
            Positive(mag) => (Plus, mag.to_u32_digits()),
            Negative(mag) => (Minus, mag.to_u32_digits()),
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
        match self {
            &Small(0) => Vec::new(),
            &Small(n) => {
                let bytes = n.to_be_bytes();
                let to_discard = if n >= 0 { 0 } else { 0xff };
                let mut i = 0;
                while i < bytes.len() && bytes[i] == to_discard {
                    i += 1
                }
                bytes[i..].to_vec()
            }
            Positive(mag) => mag.to_bytes_be(),
            Negative(_) => BigInt::from(self.clone()).to_signed_bytes_be(),
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
        match self {
            &Small(0) => Vec::new(),
            &Small(n) => {
                let bytes = n.to_le_bytes();
                let to_discard = if n >= 0 { 0 } else { 0xff };
                let mut i = 16;
                while i > 0 && bytes[i - 1] == to_discard {
                    i -= 1
                }
                bytes[..i].to_vec()
            }
            Positive(mag) => mag.to_bytes_le(),
            Negative(_) => BigInt::from(self.clone()).to_signed_bytes_le(),
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
        BigInt::from(self.clone()).to_str_radix(radix)
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
        BigInt::from(self.clone()).to_radix_be(radix)
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
        BigInt::from(self.clone()).to_radix_le(radix)
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
        match self {
            &Small(n) => {
                if n > 0 {
                    Plus
                } else if n < 0 {
                    Minus
                } else {
                    NoSign
                }
            }
            Positive(_) => Plus,
            Negative(_) => Minus,
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
        match self {
            Small(n) => Cow::Owned(BigInt::from(*n).into_parts().1),
            Positive(mag) => Cow::Borrowed(mag),
            Negative(mag) => Cow::Borrowed(mag),
        }
    }

    /// Returns the magnitude of the `CBigInt` as a `BigUint` if the necessary
    /// `BigUint` already exists.
    #[inline]
    pub fn try_magnitude(&self) -> Option<&BigUint> {
        match self {
            Small(_) => None,
            Positive(mag) => Some(mag),
            Negative(mag) => Some(mag),
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
        match self {
            &Small(n) => {
                if n >= 0 {
                    DIGIT_BITS as u32 - 1 - n.leading_zeros()
                } else if n == Digit::MIN {
                    DIGIT_BITS as u32
                } else {
                    (-n).leading_zeros()
                }
            }
            .into(),
            Positive(mag) => mag.bits(),
            Negative(mag) => mag.bits(),
        }
    }

    /// Converts this `CBigInt` into a `BigInt`.
    #[inline]
    fn into_bigint(self) -> BigInt {
        match self {
            Small(n) => BigInt::from(n),
            Positive(uint) => BigInt::from_biguint(Plus, uint),
            Negative(uint) => BigInt::from_biguint(Minus, uint),
        }
    }

    /// Converts this `CBigInt` into a `BigInt`.
    #[inline]
    pub fn to_bigint(&self) -> BigInt {
        self.clone().into_bigint()
    }

    /// Converts this `CBigInt` into a `BigUint`, if it's not negative.
    pub fn to_biguint(&self) -> Option<BigUint> {
        match self {
            Small(n) => {
                if *n >= 0 {
                    Some(BigUint::from(*n as u128))
                } else {
                    None
                }
            }
            Positive(uint) => Some(uint.clone()),
            Negative(_) => None,
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
        if let Small(a) = &self {
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
        if let Positive(uint) = self {
            // Possibly avoid some cloning by operating directly on unsigned values.
            if !exponent.is_negative() && !modulus.is_negative() {
                return uint
                    .modpow(exponent.magnitude().borrow(), modulus.magnitude().borrow())
                    .into();
            }
        }
        BigInt::from(self.clone())
            .modpow(&exponent.clone().into(), &modulus.clone().into())
            .into()
    }

    /// Returns the number of least-significant bits that are zero,
    /// or `None` if the entire number is zero.
    pub fn trailing_zeros(&self) -> Option<u64> {
        match self {
            &Small(0) => None,
            &Small(n) if n > 0 => Some(n.trailing_zeros() as u64),
            &Small(Digit::MIN) => Some(DIGIT_BITS as u64),
            &Small(n) => Some((-n).trailing_zeros() as u64),
            Positive(mag) => mag.trailing_zeros(),
            Negative(mag) => mag.trailing_zeros(),
        }
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
        Small(0)
    }
}

impl Display for CBigInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Small(n) => write!(f, "{}", n),
            Positive(n) => write!(f, "{}", n),
            Negative(n) => write!(f, "-{}", n),
        }
    }
}

impl ToBigInt for CBigInt {
    fn to_bigint(&self) -> Option<BigInt> {
        Some(self.clone().into())
    }
}

impl ToBigUint for CBigInt {
    fn to_biguint(&self) -> Option<BigUint> {
        self.clone().try_into().ok()
    }
}

impl From<BigInt> for CBigInt {
    fn from(value: BigInt) -> Self {
        Self::from_bigint(value)
    }
}

impl From<BigUint> for CBigInt {
    fn from(value: BigUint) -> Self {
        Self::from_biguint(Plus, value)
    }
}

impl From<CBigInt> for BigInt {
    fn from(value: CBigInt) -> Self {
        value.into_bigint()
    }
}

impl TryFrom<CBigInt> for BigUint {
    type Error = TryFromBigIntError<()>;
    fn try_from(value: CBigInt) -> Result<Self, Self::Error> {
        match value {
            Small(n) => n.to_biguint().ok_or_else(try_into_bigint_error),
            Positive(mag) => Ok(mag),
            Negative(_) => Err(try_into_bigint_error()),
        }
    }
}

impl TryFrom<&CBigInt> for BigUint {
    type Error = TryFromBigIntError<()>;
    fn try_from(value: &CBigInt) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}

impl Zero for CBigInt {
    fn zero() -> Self {
        CBigInt::from(0)
    }

    fn is_zero(&self) -> bool {
        if let Small(n) = self {
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
        if let Small(n) = self {
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
            Small(a) => {
                if let (b, false) = a.overflowing_abs() {
                    b.into()
                } else {
                    BigInt::from(*a).abs().into()
                }
            }
            Positive(a) => Positive(a.clone()),
            Negative(a) => Positive(a.clone()),
        }
    }

    fn abs_sub(&self, other: &Self) -> Self {
        self.sub(other).abs()
    }

    fn signum(&self) -> Self {
        match self {
            Small(a) => a.signum().into(),
            Positive(_) => 1.into(),
            Negative(_) => (-1).into(),
        }
    }

    fn is_positive(&self) -> bool {
        match self {
            Small(a) => a.is_positive(),
            Positive(_) => true,
            Negative(_) => false,
        }
    }

    fn is_negative(&self) -> bool {
        match self {
            Small(a) => a.is_negative(),
            Negative(_) => true,
            Positive(_) => false,
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
        match (self, other) {
            (Small(a), Small(b)) => a.cmp(b),
            (Positive(a), Positive(b)) => a.cmp(b),
            (Negative(a), Negative(b)) => b.cmp(a),
            (Positive(_), _) => Ordering::Greater,
            (_, Positive(_)) => Ordering::Less,
            (Negative(_), _) => Ordering::Less,
            (_, Negative(_)) => Ordering::Greater,
        }
    }
}

impl Integer for CBigInt {
    fn div_floor(&self, other: &Self) -> Self {
        if let (&Small(lhs), &Small(rhs)) = (self, other) {
            if (lhs, rhs) != (Digit::MIN, -1) {
                return lhs.div_floor(&rhs).into();
            }
        }
        BigInt::from(self.clone())
            .div_floor(&BigInt::from(other.clone()))
            .into()
    }

    fn mod_floor(&self, other: &Self) -> Self {
        if let (&Small(lhs), &Small(rhs)) = (self, other) {
            lhs.mod_floor(&rhs).into()
        } else {
            BigInt::from(self.clone())
                .mod_floor(&BigInt::from(other.clone()))
                .into()
        }
    }

    fn gcd(&self, other: &Self) -> Self {
        todo!()
        // maybe_bigints(
        //     self,
        //     other,
        //     |x, y| x.gcd(&*y).into(),
        //     |x, y| Some(x.gcd(y).into()),
        // )
    }

    fn lcm(&self, other: &Self) -> Self {
        todo!()
        // maybe_bigints(
        //     self,
        //     other,
        //     |x, y| x.gcd(&*y).into(),
        //     |x, y| {
        //         if !x.overflowing_mul(*y).1 {
        //             Some(x.gcd(y).into())
        //         } else {
        //             None
        //         }
        //     },
        // )
    }

    fn divides(&self, other: &Self) -> bool {
        if let (&Small(lhs), &Small(rhs)) = (self, other) {
            lhs.divides(&rhs)
        } else {
            BigInt::from(self.clone()).divides(&BigInt::from(other.clone()))
        }
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        if let (&Small(lhs), &Small(rhs)) = (self, other) {
            lhs.is_multiple_of(&rhs)
        } else {
            BigInt::from(self.clone()).is_multiple_of(&BigInt::from(other.clone()))
        }
    }

    fn is_even(&self) -> bool {
        match self {
            Small(n) => n.is_even(),
            Positive(mag) => mag.is_even(),
            Negative(mag) => mag.is_even(),
        }
    }

    fn is_odd(&self) -> bool {
        match self {
            Small(n) => n.is_odd(),
            Positive(mag) => mag.is_odd(),
            Negative(mag) => mag.is_odd(),
        }
    }

    fn div_rem(&self, other: &Self) -> (Self, Self) {
        if let (&Small(lhs), &Small(rhs)) = (self, other) {
            if (lhs, rhs) != (Digit::MIN, -1) {
                let (q, r) = lhs.div_rem(&rhs);
                return (q.into(), r.into());
            }
        }
        let (q, r) = BigInt::from(self.clone()).div_rem(&BigInt::from(other.clone()));
        return (q.into(), r.into());
    }
}

#[test]
fn gcd_test() {
    // let small = CBigInt::from(5);
    // let huge = CBigInt::from(i128::MAX).pow(2);
    // catch_unwind(|| {
    //     maybe_bigint_ref(
    //         &huge,
    //         |_| {
    //             panic!();
    //         },
    //         |_| {},
    //     );
    // })
    // .unwrap_err();
    // assert_eq!(huge.gcd(&small), CBigInt::from(1));
    // assert_eq!(small.gcd(&huge), CBigInt::from(1));
    panic!()
}

impl Roots for CBigInt {
    fn nth_root(&self, n: u32) -> Self {
        if let Small(a) = self {
            return a.nth_root(n).into();
        }
        Self::from_biguint(self.sign(), (&*self.magnitude()).sqrt())
    }
}

impl Neg for CBigInt {
    type Output = CBigInt;

    fn neg(self) -> Self::Output {
        if let Small(a) = self {
            if let (b, false) = a.overflowing_neg() {
                return b.into();
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
        if let Small(a) = self {
            return a.not().into();
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

// We can't constructor a TryFromBigIntError directly, so we get sneaky.
fn try_into_bigint_error() -> TryFromBigIntError<()> {
    BigUint::try_from(-1).expect_err("converting -1 to BigUint fails")
}

macro_rules! each_prim {
    [int_prim, [$prim:ident, $to_prim:ident]] => {
        impl From<$prim> for CBigInt {
            fn from(value: $prim) -> Self {
                if let Ok(converted) = Digit::try_from(value) {
                    CBigInt::from_small_int(converted)
                } else {
                    BigInt::from(value).into()
                }
            }
        }
        impl TryFrom<CBigInt> for $prim {
            type Error = TryFromBigIntError<BigInt>;
            fn try_from(value: CBigInt) -> Result<Self, Self::Error> {
                if let Small(n) = value {
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
    };
    [float_prim, $prim_attrs:tt] => {
    };
}

macro_rules! to_prim_method {
    [int_prim, [$prim:ident, $to_prim:ident]] => {
        fn $to_prim(&self) -> Option<$prim> {
            if let Small(value) = self {
                $prim::try_from(*value).ok()
            } else {
                $prim::try_from(BigInt::from(self.clone())).ok()
            }
        }
    };
    [float_prim, [$prim:ident, $to_prim:ident]] => {
        fn $to_prim(&self) -> Option<$prim> {
            if let Small(value) = self {
                Some(*value as $prim)
            } else {
                BigInt::from(self.clone()).$to_prim()
            }
        }
    };
}

macro_rules! each_op {
    [arith_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident, $overflowing_op:ident]] => {
        // impl<L, R> $trait<R> for L where L: CBigIntFnArg<CBigInt>, R: CBigIntFnArg<CBigInt> {
        //     fn $op(self, other: R) -> CBigInt {
        //
        //     }
        // }

        impl $trait for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: Self) -> Self::Output {
                if let (Small(a), Small(b)) = (&self, &rhs) {
                    if let (c, false) = a.$overflowing_op(*b) {
                        return c.into();
                    }
                }
                BigInt::from(self).$op(BigInt::from(rhs)).into()
            }
        }
        assign_op!($trait, $op, $assign_trait, $assign_op);
        ref_op!($trait<CBigInt> for CBigInt, $op);
        //
        // impl $trait for &CBigInt {
        //     type Output = CBigInt;
        //     fn $op(self, rhs: Self) -> Self::Output {
        //         maybe_bigints(self, rhs, |lhs, rhs| {
        //                 lhs.$op(&*rhs).into()
        //             }, |lhs, rhs| {
        //                 if let (c, false) = lhs.$overflowing_op(*rhs) {
        //                     Some(c.into())
        //                 } else {
        //                     None
        //                 }
        //             }
        //         )
        //     }
        // }
        // impl $trait for CBigInt {
        //     type Output = CBigInt;
        //     fn $op(self, rhs: Self) -> Self::Output {
        //         maybe_bigints(&self, &rhs, |lhs, rhs| {
        //                 std::mem::take(lhs).$op(std::mem::take(rhs)).into()
        //             }, |lhs, rhs| {
        //                 if let (c, false) = lhs.$overflowing_op(*rhs) {
        //                     Some(c.into())
        //                 } else {
        //                     None
        //                 }
        //             }
        //         )
        //     }
        // }
        //
        // assign_op!($trait, $op, $assign_trait, $assign_op);
        // //ref_op!($trait<CBigInt> for CBigInt, $op);
    };
    [shift_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident, $overflowing_op:ident]] => {
        assign_op!($trait, $op, $assign_trait, $assign_op);
    };
    [bit_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        impl $trait for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: Self) -> Self::Output {
                if let (Small(a), Small(b)) = (&self, &rhs) {
                    return a.$op(*b).into();
                }
                BigInt::from(self).$op(BigInt::from(rhs)).into()
            }
        }
        assign_op![$trait, $op, $assign_trait, $assign_op];
        ref_op![$trait<CBigInt> for CBigInt, $op];
    };
}

macro_rules! ref_op {
    [$trait:ident<$rhs_type:ty> for $lhs_type:ty, $op:ident] => {
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

macro_rules! assign_op {
    [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident] => {
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
    [
        int_prim, [$prim:ident, $to_prim:ident],
        arith_op, [
            $trait:ident,
            $op:ident,
            $assign_trait:ident,
            $assign_op:ident,
            $overflowing_op:ident
        ]
    ] => {
        impl $trait<$prim> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: $prim) -> Self::Output {
                if let Small(prim) = &self {
                    if let Ok(promoted) = Digit::try_from(rhs) {
                        if let (result, false) = prim.$overflowing_op(promoted) {
                            return result.into();
                        }
                    }
                }
                BigInt::from(self).$op(rhs).into()
            }
        }
        impl $trait<CBigInt> for $prim {
            type Output = CBigInt;
            fn $op(self, rhs: CBigInt) -> Self::Output {
                if let Small(prim) = &rhs {
                    if let Ok(promoted) = Digit::try_from(self) {
                        if let (result, false) = promoted.$overflowing_op(*prim) {
                            return result.into();
                        }
                    }
                }
                self.$op(BigInt::from(rhs)).into()
            }
        }
        ref_op!($trait<$prim> for CBigInt, $op);
        ref_op!($trait<CBigInt> for $prim, $op);
    };
    [
        int_prim, [$prim:ident, $to_prim:ident],
        shift_op, [
            $trait:ident,
            $op:ident,
            $assign_trait:ident,
            $assign_op:ident,
            $overflowing_op:ident
        ]
    ] => {
        impl $trait<$prim> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: $prim) -> Self::Output {
                if let Small(lhs) = &self {
                    if let Ok(rhs) = u32::try_from(rhs) {
                        if let (result, false) = lhs.$overflowing_op(rhs) {
                            return result.into();
                        }
                    }
                }
                BigInt::from(self).$op(rhs).into()
            }
        }
        ref_op!($trait<$prim> for CBigInt, $op);
    };
    [$prim_type:tt, $prim_attrs:tt, $op_type:tt, $op_attrs:tt] => {};
}

impl ToPrimitive for CBigInt {
    with_prims!(to_prim_method, []);
}

with_prims!(each_prim, []);
with_prims_and_ops!(each_prim_and_op, []);
with_ops!(each_op, []);

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
                    assert_eq!(
                        BigInt::from(cop(CBigInt::from(a.clone()), CBigInt::from(b.clone()))),
                        op(a.clone(), b.clone()),
                        "failed: {} {} {}",
                        a,
                        op_name,
                        b,
                    );
                }
            }
        }
    }
}
//
// #[test]
// fn macro_test() {
//     macro_rules! call_macro {
//         ($name:ident, $init_args:tt, $($final_args:tt),*) => {
//             $(call_macro!(@internal $name, $init_args, $final_args);)*
//         };
//         (@internal $name:ident, [$($init_arg:tt),*], [$($final_arg:tt),*]) => {
//             $name!($($init_arg,)*$($final_arg),*)
//         }
//     }
//
//     call_macro!(println, ["{} {} {}", 1], [2, 3], [4, 5]);
// }
