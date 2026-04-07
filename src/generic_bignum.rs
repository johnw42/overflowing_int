use crate::big_number::{BigNumber, BigNumberDigits};
use crate::generic_bignum::encoding::{Decode, Decoded, Encode, Encoding};
use crate::small_num::SmallNumber;
use num_bigint::BigInt;
use num_integer::Roots;
use num_traits::{One, PrimInt, Zero};
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

pub mod encoding;
pub mod num_ops;
pub mod signed;
mod trait_impls;
pub mod unsigned;

/// A signed big integer type that can be used with any encoding that implements
/// `Encoding` with `Big = BigInt`.  Implements the same methods and traits as
/// `BigInt`, and can be used as a drop-in replacement for `BigInt` in most
/// cases.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct GenericBigNum<'a, E: Encoding<'a>>(E, PhantomData<&'a ()>);

impl<'a, E: Encoding<'a>> GenericBigNum<'a, E> {
    pub fn from_encoding(enc: E) -> Self {
        Self(enc, PhantomData)
    }

    pub fn into_big(self) -> E::Big {
        self.into_big_cow().into_owned()
    }

    pub fn into_bigint(self) -> BigInt {
        self.into_big_cow().into_owned().into()
    }

    pub fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self> {
        E::Big::parse_bytes(buf, radix).map(Self::from_big)
    }

    pub fn to_str_radix(&self, radix: u32) -> String {
        self.with_big_cow(|cow| cow.to_str_radix(radix))
    }

    pub fn bit(&self, bit: u64) -> bool {
        self.with_decoded(|encoded| match encoded {
            Decoded::Small(small) => {
                if bit < E::Small::BITS as u64 {
                    (small >> (bit as u32)) & E::Small::one() == E::Small::one()
                } else {
                    small < E::Small::zero()
                }
            }
            Decoded::Big(big) => big.bit(bit),
        })
    }

    pub fn bits(&self) -> u64 {
        self.with_decoded(|encoded| match encoded {
            Decoded::Small(n) => {
                if n >= E::Small::zero() {
                    E::Small::BITS - n.leading_zeros()
                } else {
                    E::Small::BITS - n.unsigned_abs().leading_zeros()
                }
            }
            .into(),
            Decoded::Big(n) => n.bits(),
        })
    }

    pub fn checked_add(&self, v: &Self) -> Option<Self> {
        self.with_big_cows(v, |lhs, rhs| lhs.checked_add(&rhs).map(Self::from_big))
    }

    pub fn checked_sub(&self, v: &Self) -> Option<Self> {
        self.with_big_cows(v, |lhs, rhs| lhs.checked_sub(&rhs).map(Self::from_big))
    }

    pub fn checked_mul(&self, v: &Self) -> Option<Self> {
        self.with_big_cows(v, |lhs, rhs| lhs.checked_mul(&rhs).map(Self::from_big))
    }

    pub fn checked_div(&self, v: &Self) -> Option<Self> {
        self.with_big_cows(v, |lhs, rhs| lhs.checked_div(&rhs).map(Self::from_big))
    }

    pub fn pow(&self, exponent: u32) -> Self {
        if let Some(a) = self.small()
            && let (a, false) = a.overflowing_pow(exponent)
        {
            return Self::from_small(a);
        }
        self.with_big_cow(|big| Self::from_big(big.pow(exponent)))
    }

    pub fn modpow(&self, exponent: &Self, modulus: &Self) -> Self {
        self.with_big_cows(exponent, |lhs, rhs| {
            modulus.with_big_cow(|modulus| Self::from_big(lhs.modpow(&rhs, &modulus)))
        })
    }

    pub fn sqrt(&self) -> Self {
        self.with_decoded(|encoded| match encoded {
            Decoded::Small(n) => Self::from_small(n.sqrt()),
            Decoded::Big(n) => Self::from_big(Roots::sqrt(&n)),
        })
    }

    pub fn cbrt(&self) -> Self {
        self.with_decoded(|encoded| match encoded {
            Decoded::Small(n) => Self::from_small(n.cbrt()),
            Decoded::Big(n) => Self::from_big(Roots::cbrt(&n)),
        })
    }

    pub fn nth_root(&self, n: u32) -> Self {
        self.with_decoded(|encoded| match encoded {
            Decoded::Small(x) => Self::from_small(x.nth_root(n)),
            Decoded::Big(x) => Self::from_big(Roots::nth_root(&x, n)),
        })
    }

    pub fn trailing_zeros(&self) -> Option<u64> {
        self.with_big_cow(|cow| cow.trailing_zeros())
    }

    pub fn iter_u32_digits(&self) -> impl BigNumberDigits<'_, u32> {
        self.with_big_cow(|cow| cow.iter_u32_digits().collect::<Vec<_>>().into_iter())
    }

    pub fn iter_u64_digits(&self) -> impl BigNumberDigits<'_, u64> {
        self.with_big_cow(|cow| cow.iter_u64_digits().collect::<Vec<_>>().into_iter())
    }

    pub fn modinv(&self, modulus: &Self) -> Option<Self> {
        self.with_big_cows(modulus, |lhs, rhs| lhs.modinv(&rhs).map(Self::from_big))
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
                let mut big: <E as Encoding<'a>>::Big = n.to_big();
                big.set_bit(bit, value);
                *encoding = Decoded::Big(Cow::Owned(big));
            }
            Decoded::Big(n) => n.to_mut().set_bit(bit, value),
        })
    }
}

impl<'a, E: Encoding<'a>> Decode<'a, E::Small> for GenericBigNum<'a, E> {
    fn decode(self) -> Decoded<E::Small, Cow<'a, E::Big>> {
        self.0.decode()
    }

    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<E::Small, Cow<E::Big>>) -> T) -> T {
        self.0.with_decoded(f)
    }
}

impl<'a, E: Encoding<'a>> Decode<'a, E::Small> for &GenericBigNum<'a, E> {
    fn decode(self) -> Decoded<E::Small, Cow<'a, E::Big>> {
        self.0.clone().decode()
    }

    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<E::Small, Cow<E::Big>>) -> T) -> T {
        self.0.with_decoded(f)
    }
}

impl<'a, E: Encoding<'a>> Encode<'a, E::Small> for GenericBigNum<'a, E> {
    fn from_small(s: E::Small) -> Self {
        Self::from_encoding(E::from_small(s))
    }

    fn from_big_cow(b: Cow<'a, E::Big>) -> Self {
        Self::from_encoding(E::from_big_cow(b))
    }
}

impl<'a, E: Encoding<'a>> Encoding<'a> for GenericBigNum<'a, E> {
    type Small = E::Small;
    type Big = E::Big;
    type Unsigned = E::Unsigned;
    type Static = GenericBigNum<'static, E::Static>;

    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<E::Small, Cow<E::Big>>)) {
        self.0.update_encoding(f);
    }

    fn into_static(self) -> GenericBigNum<'static, E::Static> {
        GenericBigNum(self.0.into_static(), PhantomData)
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Display for GenericBigNum<'a, E> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self.with_decoded(|encoded| match encoded {
            Decoded::Small(n) => Display::fmt(&n, f),
            Decoded::Big(n) => Display::fmt(&n, f),
        })
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> Debug for GenericBigNum<'a, E> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self.with_decoded(|encoded| match encoded {
            Decoded::Small(n) => Debug::fmt(&n, f),
            Decoded::Big(n) => Debug::fmt(&n, f),
        })
    }
}

// TODO: Implement numeric traits for `GenericBigNum`?
