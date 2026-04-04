use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::Neg;

use num_bigint::{BigInt, BigUint, Sign, ToBigInt};
use num_integer::Roots;
use num_traits::{One, PrimInt, Zero};

use crate::big_number::{BigNumber, BigNumberDigits};
use crate::generic_bignum::encoding::{Decoded, EncodedBigNum};
use crate::small_num::SmallNumber;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct GenericBigNum<'a, E: EncodedBigNum<'a>>(pub(crate) E, PhantomData<&'a ()>);

impl<'a, E: EncodedBigNum<'a>> GenericBigNum<'a, E> {
    pub fn from_encoding(enc: E) -> Self {
        Self(enc, PhantomData)
    }

    pub fn small_with<'b, E2: EncodedBigNum<'b, Small = E::Small, Big = E::Big>>(
        &self,
        right: &GenericBigNum<'b, E2>,
    ) -> Option<(E::Small, E::Small)> {
        match (self.small(), right.small()) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }

    pub fn to_big(&self) -> E::Big {
        self.big_cow().into_owned()
    }

    pub fn into_big(self) -> E::Big {
        self.into_big_cow().into_owned()
    }

    pub fn big_cow(&self) -> Cow<'a, E::Big> {
        self.0.big_cow()
    }

    fn big_ref(&self) -> Option<&E::Big> {
        self.0.big_ref()
    }

    fn to_bigint(&self) -> BigInt {
        self.to_big().into()
    }

    pub fn into_bigint(self) -> BigInt {
        self.into_big_cow().into_owned().into()
    }

    pub fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self> {
        E::Big::parse_bytes(buf, radix).map(Self::from_big)
    }

    pub fn to_str_radix(&self, radix: u32) -> String {
        self.big_cow().to_str_radix(radix)
    }

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

    pub fn checked_add(&self, v: &'a Self) -> Option<Self> {
        self.big_cow().checked_add(&v.big_cow()).map(Self::from_big)
    }

    pub fn checked_sub(&self, v: &'a Self) -> Option<Self> {
        self.big_cow().checked_sub(&v.big_cow()).map(Self::from_big)
    }

    pub fn checked_mul(&self, v: &'a Self) -> Option<Self> {
        self.big_cow().checked_mul(&v.big_cow()).map(Self::from_big)
    }

    pub fn checked_div(&self, v: &'a Self) -> Option<Self> {
        self.big_cow().checked_div(&v.big_cow()).map(Self::from_big)
    }

    pub fn pow(&self, exponent: u32) -> Self {
        if let Some(a) = self.small()
            && let (a, false) = a.overflowing_pow(exponent)
        {
            return Self::from_small(a);
        }
        Self::from_big(self.big_cow().pow(exponent))
    }

    pub fn modpow(&self, exponent: &'a Self, modulus: &'a Self) -> Self {
        Self::from_big(
            self.big_cow()
                .modpow(&exponent.big_cow(), &modulus.big_cow()),
        )
    }

    pub fn sqrt(&self) -> Self {
        match self.decode_ref() {
            Decoded::Small(n) => Self::from_small(n.sqrt()),
            Decoded::Big(n) => Self::from_big(Roots::sqrt(&n)),
        }
    }

    pub fn cbrt(&self) -> Self {
        match self.decode_ref() {
            Decoded::Small(n) => Self::from_small(n.cbrt()),
            Decoded::Big(n) => Self::from_big(Roots::cbrt(&n)),
        }
    }

    pub fn nth_root(&self, n: u32) -> Self {
        match self.decode_ref() {
            Decoded::Small(x) => Self::from_small(x.nth_root(n)),
            Decoded::Big(x) => Self::from_big(Roots::nth_root(&x, n)),
        }
    }

    pub fn trailing_zeros(&self) -> Option<u64> {
        self.big_cow().trailing_zeros()
    }

    pub fn iter_u32_digits(&self) -> impl BigNumberDigits<'a, u32> {
        let cow = self.big_cow();
        let digits = (*cow).iter_u32_digits().collect::<Vec<_>>();
        digits.into_iter()
    }

    pub fn iter_u64_digits(&self) -> impl BigNumberDigits<'a, u64> {
        let cow = self.big_cow();
        let digits = (*cow).iter_u64_digits().collect::<Vec<_>>();
        digits.into_iter()
    }

    pub fn modinv(&self, modulus: &'a Self) -> Option<Self> {
        self.big_cow()
            .modinv(&modulus.big_cow())
            .map(Self::from_big)
    }

    pub fn set_bit(&'a mut self, bit: u64, value: bool) {
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
                let mut big: <E as EncodedBigNum<'a>>::Big = n.to_big();
                big.set_bit(bit, value);
                *encoding = Decoded::Big(big);
            }
            Decoded::Big(n) => n.set_bit(bit, value),
        })
    }
}

impl<'a, E: EncodedBigNum<'a>> EncodedBigNum<'a> for GenericBigNum<'a, E> {
    type Small = E::Small;
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

    fn into_big_cow(self) -> Cow<'a, E::Big> {
        self.0.into_big_cow()
    }

    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<E::Small, E::Big>)) {
        self.0.update_encoding(f);
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> Default for GenericBigNum<'a, E> {
    fn default() -> Self {
        GenericBigNum::from(0i32)
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> Display for GenericBigNum<'a, E> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.decode_ref() {
            Decoded::Small(n) => Display::fmt(&n, f),
            Decoded::Big(n) => Display::fmt(&n, f),
        }
    }
}

impl<'a, E: EncodedBigNum<'a, Big = BigInt>> Debug for GenericBigNum<'a, E> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.decode_ref() {
            Decoded::Small(n) => Debug::fmt(&n, f),
            Decoded::Big(n) => Debug::fmt(&n, f),
        }
    }
}
