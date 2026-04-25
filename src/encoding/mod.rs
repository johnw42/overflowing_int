use crate::{
    duplicate_prims,
    num_traits::{
        big_number::{BigNumber, BigNumberDigits},
        small_number::{SmallNumber, Widen},
    },
};
use num_bigint::{BigInt, BigUint};
use num_integer::Roots;
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, PrimInt, Zero};
use std::{borrow::Cow, hash::Hash};

pub mod arc;
pub mod bignum;
pub mod cow;
pub mod decoded;
mod shifted;

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
}

#[inline(always)]
fn checked_op<'enc, E>(
    lhs: &E,
    rhs: &E,
    f_small: impl FnOnce(&E::Small, &E::Small) -> Option<E::Small>,
    f_big: impl FnOnce(&E::Big, &E::Big) -> Option<E::Big>,
) -> Option<E>
where
    E: OwnedEncoding<'enc>,
{
    let same_size = match (lhs.decode(), rhs.decode()) {
        (Decoded::Small(s1), Decoded::Small(s2)) => Decoded::Small((s1, s2)),
        (Decoded::Small(s1), Decoded::Big(b2)) => Decoded::Big((Cow::Owned(s1.to_big()), b2)),
        (Decoded::Big(b1), Decoded::Small(s2)) => Decoded::Big((b1, Cow::Owned(s2.to_big()))),
        (Decoded::Big(b1), Decoded::Big(b2)) => Decoded::Big((b1, b2)),
    };

    Some(match same_size {
        Decoded::Small((s1, s2)) => match f_small(&s1, &s2) {
            Some(result) => E::from_small(result),
            None => E::from_big(f_big(&s1.to_big(), &s2.to_big())?),
        },
        Decoded::Big((b1, b2)) => E::from_big(f_big(b1.as_ref(), b2.as_ref())?),
    })
}

/// An encoding of a big number, where small values are encoded directly in the
/// representation, and big values are encoded as a separate big type.  The
/// encoding must be able to be updated in place, and must be able to be
/// compared for equality and hashed without decoding.
pub trait Encoding<'enc>: Decode<'enc, Self::Small>
where
    Self: 'enc,
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

    /// A version of this encoding that is capable of owning its bigint value.
    type Owned: OwnedEncoding<'enc, Small = Self::Small, Big = Self::Big>;

    /// A version of this encoding that has a static lifetime.
    type Static: OwnedEncoding<'static, Small = Self::Small, Big = Self::Big>;

    /// A variant of this encoding that can be cloned cleaply because it shares
    /// data with another encoding instance.
    type Borrowed<'a>: Encoding<'a, Small = Self::Small, Big = Self::Big>
    where
        Self: 'a;

    const ZERO: Self::Owned;

    /// Encodes a small value.
    fn from_small(s: Self::Small) -> Self::Owned;

    /// Encodes an owned big value.
    fn from_big(b: Self::Big) -> Self::Owned;

    /// Encodes a big value by reference.
    fn from_big_ref(b: &'enc Self::Big) -> Self::Borrowed<'enc>;

    /// Converts an encoding of one type into an encoding of another type with the same big representation.
    fn reencode_from<'e2, E2>(other: E2) -> Self::Owned
    where
        E2: Encoding<'e2, Big = Self::Big>,
        E2::Small: Widen<Self::Small>,
        Self::Small: TryFrom<<E2::Small as Widen<Self::Small>>::Output>,
        'e2: 'enc,
    {
        match other.into_decoded() {
            Decoded::Small(s) => match Self::Small::try_from(s.widen()).ok() {
                Some(s) => Self::from_small(s),
                None => Self::from_big(s.to_big()),
            },
            Decoded::Big(b) => Self::from_big(b.into_owned()),
        }
    }

    /// Converts this encoding to a version with a static lifetime.
    fn into_static(self) -> Self::Static;

    /// Converts this encoding into an owned version of the same encoding.
    fn into_owned(self) -> Self::Owned;

    /// Converts this encoding into a version with a shorter lifetime.
    fn borrow<'a>(&'a self) -> Self::Borrowed<'a>;

    fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self::Owned> {
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

    fn checked_add(&self, v: &Self) -> Option<Self>
    where
        Self: OwnedEncoding<'enc>,
    {
        checked_op(self, v, CheckedAdd::checked_add, CheckedAdd::checked_add)
    }

    fn checked_sub(&self, v: &Self) -> Option<Self>
    where
        Self: OwnedEncoding<'enc>,
    {
        checked_op(self, v, CheckedSub::checked_sub, CheckedSub::checked_sub)
    }

    fn checked_mul(&self, v: &Self) -> Option<Self>
    where
        Self: OwnedEncoding<'enc>,
    {
        checked_op(self, v, CheckedMul::checked_mul, CheckedMul::checked_mul)
    }

    fn checked_div(&self, v: &Self) -> Option<Self>
    where
        Self: OwnedEncoding<'enc>,
    {
        checked_op(self, v, CheckedDiv::checked_div, CheckedDiv::checked_div)
    }

    fn modpow(&self, exponent: &Self, modulus: &Self) -> Self::Owned {
        Self::from_big(
            self.big_cow()
                .modpow(&exponent.big_cow(), &modulus.big_cow()),
        )
    }

    fn sqrt(&self) -> Self::Owned {
        match self.decode() {
            Decoded::Small(n) => Self::from_small(n.sqrt()),
            Decoded::Big(n) => Self::from_big(Roots::sqrt(&n)),
        }
    }

    fn cbrt(&self) -> Self::Owned {
        match self.decode() {
            Decoded::Small(n) => Self::from_small(n.cbrt()),
            Decoded::Big(n) => Self::from_big(Roots::cbrt(&n)),
        }
    }

    fn nth_root(&self, n: u32) -> Self::Owned {
        match self.decode() {
            Decoded::Small(x) => {
                // Corner case: Computing the nth root of the minimum value of a
                // small number type causes an overflow when the implemention
                // attempts to negate the value.
                if x < Self::Small::zero() && x == Self::Small::MIN {
                    Self::from_big(Self::Small::MIN.to_big().nth_root(n))
                } else {
                    Self::from_small(x.nth_root(n))
                }
            }
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

    fn modinv(&self, modulus: &Self) -> Option<Self::Owned> {
        Some(Self::from_big(self.big_cow().modinv(&modulus.big_cow())?))
    }
}

pub trait OwnedEncoding<'enc>: Encoding<'enc>
where
    Self: Encoding<'enc, Owned = Self>,
{
    fn decode_mut(&mut self) -> Decoded<Self::Small, &mut Self::Big>;

    fn set_bit(&mut self, bit: u64, value: bool) {
        match self.decode_mut() {
            Decoded::Small(n) if bit < (Self::Small::BITS - 1) as u64 => {
                let to_set = Self::Small::one() << bit as u32;
                *self = Self::from_small(if value { n | to_set } else { n & !to_set });
            }
            Decoded::Small(n) => {
                let mut big = n.to_big();
                big.set_bit(bit, value);
                *self = Self::from_big(big);
            }
            Decoded::Big(n) => n.set_bit(bit, value),
        }
    }
}

impl<'enc, E> Decode<'enc, E::Small> for &'enc E
where
    E: Encoding<'enc>,
{
    fn decode<'a>(&'a self) -> Decoded<E::Small, Cow<'a, <E::Small as SmallNumber>::Big>> {
        (*self).decode()
    }

    fn into_decoded(self) -> Decoded<E::Small, Cow<'enc, <E::Small as SmallNumber>::Big>> {
        self.clone().into_decoded()
    }
}

impl<'enc, E> Encoding<'enc> for &'enc E
where
    E: OwnedEncoding<'enc>,
{
    type Small = E::Small;
    type Big = E::Big;
    type Unsigned = E::Unsigned;
    type Static = E::Static;
    type Owned = E;

    type Borrowed<'a>
        = E::Borrowed<'a>
    where
        Self: 'a;

    const ZERO: E = E::ZERO;

    fn from_small(s: Self::Small) -> E {
        E::from_small(s)
    }

    fn from_big(b: Self::Big) -> E {
        E::from_big(b)
    }

    fn from_big_ref(b: &'enc Self::Big) -> Self::Borrowed<'enc> {
        E::from_big_ref(b)
    }

    fn into_static(self) -> Self::Static {
        E::into_static(self.clone())
    }

    fn into_owned(self) -> E {
        E::into_owned(self.clone())
    }

    fn borrow<'a>(&'a self) -> Self::Borrowed<'a> {
        E::borrow(self)
    }
}

// =============================================================================
// Implementations of `Decode` for primitive types
// =============================================================================

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

    impl<'enc, 'r, S: SmallNumber> Decode<'enc, S> for &'r prim
    where
        'r: 'enc,
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
