use crate::big_number::BigNumberDigits;
use crate::small_num::SmallNumber;
use crate::{big_number::BigNumber, duplicate_prims};
use num_bigint::{BigInt, BigUint};
use num_integer::Roots;
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, PrimInt, Zero};
use std::{borrow::Cow, hash::Hash, rc::Rc};

/// A decoded big number, which may be either small or big.  Also used to
/// represent various other decoded values, such as the pairs of numbers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Decoded<S, B> {
    Small(S),
    Big(B),
}

/// A type that can be decoded into a small or big value.  This this applies to
/// encoded big numbers, but it also applies to types that can be trivially
/// decoded, such as integer types and bignum types.
pub trait Decode<'enc, S>: Sized + Clone
where
    S: SmallNumber,
{
    /// The main method of this trait, which decodes the value into either a
    /// small or big value without ever cloning a big value.
    fn decode<'a>(&'a self) -> Decoded<S, Cow<'a, S::Big>>;

    /// Decodes the value, consuming self.
    fn into_decoded(self) -> Decoded<S, Cow<'enc, S::Big>>;

    /// Gets the big value as a `Cow`, creating a bignum if necessary.
    #[inline(always)]
    fn big_cow<'a>(&'a self) -> Cow<'a, S::Big> {
        match self.decode() {
            Decoded::Small(s) => Cow::Owned(s.to_big()),
            Decoded::Big(b) => b,
        }
    }

    /// Gets the big value as an owned value while consuming self.  Compared to
    /// [`Self::big`], using this method may allow for more efficient code when
    /// the encoding is big, since it can avoid an extra clone of the big value.
    #[inline(always)]
    fn into_big(self) -> S::Big {
        self.big_cow().into_owned()
    }

    #[inline(always)]
    fn into_bigint(self) -> BigInt {
        self.into_big().into()
    }

    /// Gets the small value if it the encoding is small, or `None` if it is big.
    #[inline(always)]
    fn small(&self) -> Option<S> {
        match self.decode() {
            Decoded::Small(s) => Some(s),
            Decoded::Big(_) => None,
        }
    }

    /// A helper method for working with two decoded values at the same time,
    /// where the big values need to be passed as `Cow`s.
    #[inline(always)]
    fn big_cows<'a>(
        lhs: &'a impl Decode<'enc, S>,
        rhs: &'a impl Decode<'enc, S>,
    ) -> (Cow<'a, S::Big>, Cow<'a, S::Big>) {
        (lhs.big_cow(), rhs.big_cow())
    }

    /// A helper method for working with two decoded values at the same time,
    /// where the big values need to be passed as `Cow`s, and if both values are
    /// small, they can be passed as smalls without converting to bigs.
    #[inline(always)]
    fn matching_size<'a>(
        lhs: &'a impl Decode<'enc, S>,
        rhs: &'a impl Decode<'enc, S>,
    ) -> Decoded<(S, S), (Cow<'a, S::Big>, Cow<'a, S::Big>)> {
        match (lhs.decode(), rhs.decode()) {
            (Decoded::Small(s1), Decoded::Small(s2)) => Decoded::Small((s1, s2)),
            (Decoded::Small(s1), Decoded::Big(b2)) => Decoded::Big((Cow::Owned(s1.to_big()), b2)),
            (Decoded::Big(b1), Decoded::Small(s2)) => Decoded::Big((b1, Cow::Owned(s2.to_big()))),
            (Decoded::Big(b1), Decoded::Big(b2)) => Decoded::Big((b1, b2)),
        }
    }
}

pub trait Encode<'enc, S>: Sized + Clone
where
    S: SmallNumber,
{
    /// Encodes a small value.
    fn from_small(s: S) -> Self;

    /// Encodes an owned big value.
    fn from_big(b: S::Big) -> Self;

    /// Encodes a big value from a `Cow`.
    fn from_big_ref(b: &'enc S::Big) -> Self;
}

#[inline(always)]
fn checked_op<'enc, 'a, E>(
    lhs: &'a E,
    rhs: &'a E,
    f_small: impl FnOnce(&E::Small, &E::Small) -> Option<E::Small>,
    f_big: impl FnOnce(&E::Big, &E::Big) -> Option<E::Big>,
) -> Option<E>
where
    E: Encoding<'enc>,
{
    match E::matching_size(lhs, rhs) {
        Decoded::Small((s1, s2)) => match f_small(&s1, &s2) {
            Some(result) => Some(E::from_small(result)),
            None => f_big(&s1.to_big(), &s2.to_big()).map(E::from_big),
        },
        Decoded::Big((b1, b2)) => f_big(b1.as_ref(), b2.as_ref()).map(E::from_big),
    }
}

/// An encoding of a big number, where small values are encoded directly in the
/// representation, and big values are encoded as a separate big type.  The
/// encoding must be able to be updated in place, and must be able to be
/// compared for equality and hashed without decoding.
pub trait Encoding<'enc>: Decode<'enc, Self::Small> + Encode<'enc, Self::Small>
where
    Self: Eq,
    Self: Hash,
    Self::Big: Into<BigInt>,
    Self::Big: Into<BigInt>,
{
    /// The small type that can be encoded directly in the representation.
    type Small: SmallNumber<Big = Self::Big>;

    /// The big type that is used when the value is too large to fit in the
    /// small representation.  This type is completely determined by the small
    /// type, but having it is a convenience for writing code that is generic
    /// over the encoding.
    type Big: BigNumber;

    /// A version of this encoding that uses an unsigned representation.
    type Unsigned: Encoding<'enc, Small = <Self::Small as SmallNumber>::Unsigned, Big = BigUint>;

    type Owned: Encoding<'static, Small = Self::Small, Big = Self::Big, Owned = Self::Owned>;

    type Borrowed<'a>: Encoding<'a, Small = Self::Small, Big = Self::Big>
    where
        Self: 'a;

    const ZERO: Self;

    /// Converts this encoding to a version with a static lifetime.
    fn into_owned(self) -> Self::Owned;

    /// Converts this encoding into a version with a shorter lifetime.
    fn borrow<'a>(&'a self) -> Self::Borrowed<'a>
    where
        Self: 'a;

    fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self> {
        Self::Big::parse_bytes(buf, radix).map(Self::from_big)
    }

    fn to_str_radix(&self, radix: u32) -> String {
        self.big_cow().to_str_radix(radix)
    }

    fn bit(&self, bit: u64) -> bool {
        match self.decode() {
            Decoded::Small(small) => {
                if bit < Self::Small::BITS as u64 {
                    (small >> (bit as u32)) & Self::Small::one() == Self::Small::one()
                } else {
                    small < Self::Small::zero()
                }
            }
            Decoded::Big(big) => big.bit(bit),
        }
    }

    fn bits(&self) -> u64 {
        match self.decode() {
            Decoded::Small(n) => {
                if n >= Self::Small::zero() {
                    Self::Small::BITS - n.leading_zeros()
                } else {
                    Self::Small::BITS - n.unsigned_abs().leading_zeros()
                }
            }
            .into(),
            Decoded::Big(n) => n.bits(),
        }
    }

    fn checked_add(&self, v: &Self) -> Option<Self> {
        checked_op(self, v, CheckedAdd::checked_add, CheckedAdd::checked_add)
    }

    fn checked_sub(&self, v: &Self) -> Option<Self> {
        checked_op(self, v, CheckedSub::checked_sub, CheckedSub::checked_sub)
    }

    fn checked_mul(&self, v: &Self) -> Option<Self> {
        checked_op(self, v, CheckedMul::checked_mul, CheckedMul::checked_mul)
    }

    fn checked_div(&self, v: &Self) -> Option<Self> {
        checked_op(self, v, CheckedDiv::checked_div, CheckedDiv::checked_div)
    }

    // TODO: Add `checked_rem` and `checked_pow` when those methods are added to `BigNumber`.

    fn modpow(&self, exponent: &Self, modulus: &Self) -> Self {
        let (lhs, rhs) = Self::big_cows(self, exponent);
        Self::from_big(lhs.modpow(&rhs, modulus.big_cow().as_ref()))
    }

    fn sqrt(&self) -> Self {
        match self.decode() {
            Decoded::Small(n) => Self::from_small(n.sqrt()),
            Decoded::Big(n) => Self::from_big(Roots::sqrt(&n)),
        }
    }

    fn cbrt(&self) -> Self {
        match self.decode() {
            Decoded::Small(n) => Self::from_small(n.cbrt()),
            Decoded::Big(n) => Self::from_big(Roots::cbrt(&n)),
        }
    }

    fn nth_root(&self, n: u32) -> Self {
        match self.decode() {
            Decoded::Small(x) => Self::from_small(x.nth_root(n)),
            Decoded::Big(x) => Self::from_big(Roots::nth_root(&x, n)),
        }
    }

    fn trailing_zeros(&self) -> Option<u64> {
        match self.decode() {
            Decoded::Small(n) if n.is_zero() => None,
            Decoded::Small(n) => Some(n.trailing_zeros() as u64),
            Decoded::Big(n) => n.trailing_zeros(),
        }
    }

    fn iter_u32_digits(&self) -> impl BigNumberDigits<'_, u32> {
        self.big_cow()
            .iter_u32_digits()
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn iter_u64_digits(&self) -> impl BigNumberDigits<'_, u64> {
        self.big_cow()
            .iter_u64_digits()
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn modinv(&self, modulus: &Self) -> Option<Self> {
        let (lhs, rhs) = Self::big_cows(self, modulus);
        lhs.modinv(&rhs).map(Self::from_big)
    }
}

pub trait EncodingMut<'enc>: Encoding<'enc> {
    /// Updates the encoding in place using the provided function.
    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<Self::Small, Cow<Self::Big>>));

    fn set_bit(&mut self, bit: u64, value: bool) {
        self.update_encoding(|encoding| match encoding {
            Decoded::Small(n) if (bit as u32) < Self::Small::BITS - 1 => {
                let to_set = Self::Small::one() << bit as u32;
                if value {
                    *n = *n | to_set;
                } else {
                    *n = *n & !to_set;
                }
            }
            Decoded::Small(n) => {
                let mut big = n.to_big();
                big.set_bit(bit, value);
                *encoding = Decoded::Big(Cow::Owned(big));
            }
            Decoded::Big(n) => n.to_mut().set_bit(bit, value),
        })
    }
}

// =============================================================================
// Implementations of `Decode` for various foreign and built-in types
// =============================================================================

impl<'enc, S> Decode<'enc, S> for Cow<'enc, S::Big>
where
    S: SmallNumber,
{
    fn into_decoded(self) -> Decoded<S, Cow<'enc, S::Big>> {
        Decoded::Big(self)
    }

    fn decode<'a>(&'a self) -> Decoded<S, Cow<'a, S::Big>> {
        Decoded::Big(Cow::Borrowed(self.as_ref()))
    }

    fn big_cow<'a>(&'a self) -> Cow<'a, S::Big> {
        Cow::Borrowed(self.as_ref())
    }
}

impl<'enc, S> Decode<'enc, S> for Rc<S::Big>
where
    S: SmallNumber,
{
    fn into_decoded(self) -> Decoded<S, Cow<'enc, S::Big>> {
        Decoded::Big(Cow::Owned((*self).clone()))
    }

    fn decode<'a>(&'a self) -> Decoded<S, Cow<'a, <S as SmallNumber>::Big>> {
        Decoded::Big(Cow::Borrowed(self.as_ref()))
    }

    fn big_cow<'a>(&'a self) -> Cow<'a, <S as SmallNumber>::Big> {
        Cow::Borrowed(self.as_ref())
    }
}

duplicate_prims! {
    impl<'enc, S> Decode<'enc, S> for prim
    where
        S: SmallNumber,
        S::Big: BigNumber + From<prim>
    {
        fn into_decoded(self) -> Decoded<S, Cow<'enc, S::Big>> {
            #[allow(irrefutable_let_patterns)]
            #[allow(clippy::unnecessary_fallible_conversions)]
            if let Ok(small) = S::try_from(self) {
                Decoded::Small(small)
            } else {
                Decoded::Big(Cow::Owned(S::Big::from(self)))
            }
        }

        fn decode<'a>(&'a self) -> Decoded<S, Cow<'a, <S as SmallNumber>::Big>> {
            #[allow(irrefutable_let_patterns)]
            #[allow(clippy::unnecessary_fallible_conversions)]
            if let Ok(small) = S::try_from(*self) {
                Decoded::Small(small)
            } else {
                Decoded::Big(Cow::Owned(S::Big::from(*self)))
            }
        }
    }

    impl<'enc, S: SmallNumber> Decode<'enc, S> for &prim
    where
        S::Big: From<prim>,
        S: TryFrom<prim>
    {
        fn into_decoded(self) -> Decoded<S, Cow<'enc, S::Big>> {
            #[allow(irrefutable_let_patterns)]
            #[allow(clippy::unnecessary_fallible_conversions)]
            if let Ok(small) = S::try_from(*self) {
                Decoded::Small(small)
            } else {
                Decoded::Big(Cow::Owned(S::Big::from(*self)))
            }
        }

        fn decode<'a>(&'a self) -> Decoded<S, Cow<'a, <S as SmallNumber>::Big>> {
            #[allow(irrefutable_let_patterns)]
            #[allow(clippy::unnecessary_fallible_conversions)]
            if let Ok(small) = S::try_from(**self) {
                Decoded::Small(small)
            } else {
                Decoded::Big(Cow::Owned(S::Big::from(**self)))
            }
        }
    }
}
