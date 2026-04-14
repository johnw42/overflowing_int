use crate::big_number::BigNumberDigits;
use crate::small_num::SmallNumber;
use crate::{big_number::BigNumber, duplicate_prims};
use num_bigint::{BigInt, BigUint};
use num_integer::Roots;
use num_traits::{One, PrimInt, Zero};
use std::{borrow::Cow, hash::Hash, rc::Rc};

/// A decoded big number, which may be either small or big.  Also used to
/// represent various other decoded values, such as the pairs of numbers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Decoded<S, B> {
    Small(S),
    Big(B),
}

pub enum EncodingKind {
    Box,
    Rc,
    Cow,
    Trivial,
    Primitive,
}

/// A type that can be decoded into a small or big value.  This this applies to
/// encoded big numbers, but it also applies to types that can be trivially
/// decoded, such as integer types and bignum types.
pub trait Decode<'a, S>: Sized + Clone
where
    S: SmallNumber,
{
    fn kind() -> EncodingKind;

    /// The main method of this trait, which decodes the value into either a
    /// small or big value without ever cloning a big value.  This method is
    /// preferable to [`Self::decode`] in most cases, but if `f` needs to
    /// clone the big value, using `decode` may avoid cloning.
    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T;

    /// Decodes the value, consuming self.
    fn decode(self) -> Decoded<S, Cow<'a, S::Big>>;

    /// Gets the big value as a `Cow`, creating a bignum if necessary.
    fn into_big_cow(self) -> Cow<'a, S::Big> {
        match self.decode() {
            Decoded::Small(s) => Cow::Owned(s.to_big()),
            Decoded::Big(b) => b,
        }
    }

    /// Gets the big value as an owned value while consuming self.  Compared to
    /// [`Self::big`], using this method may allow for more efficient code when
    /// the encoding is big, since it can avoid an extra clone of the big value.
    fn into_big(self) -> S::Big {
        self.into_big_cow().into_owned()
    }

    fn into_bigint(self) -> BigInt {
        self.into_big().into()
    }

    /// Gets the small value if it the encoding is small, or `None` if it is big.
    fn small(&self) -> Option<S> {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => Some(s),
            Decoded::Big(_) => None,
        })
    }

    /// Tests whether this encoding owns its value as a bignum.
    fn owns_bignum(&self) -> bool;

    /// Gets the big value as a `Cow`, creating a bignum if necessary, and passes it to the provided function.
    fn with_big_cow<T>(&self, f: impl FnOnce(Cow<S::Big>) -> T) -> T {
        self.with_decoded(|decoded| match decoded {
            Decoded::Small(s) => f(Cow::Owned(s.to_big())),
            Decoded::Big(b) => f(b),
        })
    }

    /// A helper method for working with two decoded values at the same time.
    /// This calls [`Self::with_big_cow`] for both `self` and `other`, and
    /// passes the resulting big `Cow`s to the provided function.  This is useful
    /// for implementing binary operations on big numbers, since it allows us to
    /// avoid cloning the big values when both encodings are big.
    fn with_big_cows<T>(
        &self,
        other: &impl Decode<'a, S>,
        f: impl FnOnce(Cow<S::Big>, Cow<S::Big>) -> T,
    ) -> T {
        self.with_big_cow(|lhs| other.with_big_cow(|rhs| f(lhs, rhs)))
    }

    /// A helper method for working with two decoded values at the same time,
    /// where the values need to both be small or both be big.  This calls
    /// [`Self::with_decoded`] for both `self` and `other`, and passes the
    /// resulting decoded values to the provided function.  This is useful for
    /// implementing binary operations on big numbers, since it allows us to
    /// avoid cloning the big values when both encodings are big, while still
    /// allowing us to work with the small values when both encodings are small.
    fn with_matching_size<T>(
        &self,
        other: &impl Decode<'a, S>,
        f: impl FnOnce(Decoded<(S, S), (Cow<S::Big>, Cow<S::Big>)>) -> T,
    ) -> T {
        self.with_decoded(|lhs| {
            other.with_decoded(|rhs| {
                let enc = match (lhs, rhs) {
                    (Decoded::Small(s1), Decoded::Small(s2)) => Decoded::Small((s1, s2)),
                    (Decoded::Small(s1), Decoded::Big(b2)) => {
                        Decoded::Big((Cow::Owned(s1.to_big()), b2))
                    }
                    (Decoded::Big(b1), Decoded::Small(s2)) => {
                        Decoded::Big((b1, Cow::Owned(s2.to_big())))
                    }
                    (Decoded::Big(b1), Decoded::Big(b2)) => Decoded::Big((b1, b2)),
                };
                f(enc)
            })
        })
    }
}

pub trait Encode<'a, S>: Sized + Clone
where
    S: SmallNumber,
{
    /// Encodes a small value.
    fn from_small(s: S) -> Self;

    /// Encodes an owned big value.
    fn from_big(b: S::Big) -> Self {
        Self::from_big_cow(Cow::Owned(b))
    }

    /// Encodes a big value from a `Cow`.
    fn from_big_cow(b: Cow<'a, S::Big>) -> Self;

    /// Encodes a value.  Prefer [`Self::from_small`] or [`Self::from_big`] when possible.
    fn from_decoded(enc: Decoded<S, Cow<'a, S::Big>>) -> Self {
        match enc {
            Decoded::Small(s) => Self::from_small(s),
            Decoded::Big(b) => Self::from_big_cow(b),
        }
    }
}

/// An encoding of a big number, where small values are encoded directly in the
/// representation, and big values are encoded as a separate big type.  The
/// encoding must be able to be updated in place, and must be able to be
/// compared for equality and hashed without decoding.
pub trait Encoding<'a>: Decode<'a, Self::Small> + Encode<'a, Self::Small>
where
    Self: Eq,
    Self: Hash,
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
    type Unsigned: Encoding<'a, Small = <Self::Small as SmallNumber>::Unsigned, Big = BigUint>;

    /// A version of this encoding that has a static lifetime.
    type Static: Encoding<'static, Small = Self::Small, Big = Self::Big>;

    const ZERO: Self;

    /// Updates the encoding in place using the provided function.
    fn update_encoding(&mut self, f: impl FnOnce(&mut Decoded<Self::Small, Cow<Self::Big>>));

    /// Converts this encoding to a version with a static lifetime.
    fn into_static(self) -> Self::Static;

    fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self> {
        Self::Big::parse_bytes(buf, radix).map(Self::from_big)
    }

    fn to_str_radix(&self, radix: u32) -> String {
        self.with_big_cow(|cow| cow.to_str_radix(radix))
    }

    fn bit(&self, bit: u64) -> bool {
        self.with_decoded(|encoded| match encoded {
            Decoded::Small(small) => {
                if bit < Self::Small::BITS as u64 {
                    (small >> (bit as u32)) & Self::Small::one() == Self::Small::one()
                } else {
                    small < Self::Small::zero()
                }
            }
            Decoded::Big(big) => big.bit(bit),
        })
    }

    fn bits(&self) -> u64 {
        self.with_decoded(|encoded| match encoded {
            Decoded::Small(n) => {
                if n >= Self::Small::zero() {
                    Self::Small::BITS - n.leading_zeros()
                } else {
                    Self::Small::BITS - n.unsigned_abs().leading_zeros()
                }
            }
            .into(),
            Decoded::Big(n) => n.bits(),
        })
    }

    fn checked_add(&self, v: &Self) -> Option<Self> {
        self.with_big_cows(v, |lhs, rhs| lhs.checked_add(&rhs).map(Self::from_big))
    }

    fn checked_sub(&self, v: &Self) -> Option<Self> {
        self.with_big_cows(v, |lhs, rhs| lhs.checked_sub(&rhs).map(Self::from_big))
    }

    fn checked_mul(&self, v: &Self) -> Option<Self> {
        self.with_big_cows(v, |lhs, rhs| lhs.checked_mul(&rhs).map(Self::from_big))
    }

    fn checked_div(&self, v: &Self) -> Option<Self> {
        self.with_big_cows(v, |lhs, rhs| lhs.checked_div(&rhs).map(Self::from_big))
    }

    fn pow(&self, exponent: u32) -> Self {
        if let Some(a) = self.small()
            && let (a, false) = a.overflowing_pow(exponent)
        {
            return Self::from_small(a);
        }
        self.with_big_cow(|big| Self::from_big(big.pow(exponent)))
    }

    fn modpow(&self, exponent: &Self, modulus: &Self) -> Self {
        self.with_big_cows(exponent, |lhs, rhs| {
            modulus.with_big_cow(|modulus| Self::from_big(lhs.modpow(&rhs, &modulus)))
        })
    }

    fn sqrt(&self) -> Self {
        self.with_decoded(|encoded| match encoded {
            Decoded::Small(n) => Self::from_small(n.sqrt()),
            Decoded::Big(n) => Self::from_big(Roots::sqrt(&n)),
        })
    }

    fn cbrt(&self) -> Self {
        self.with_decoded(|encoded| match encoded {
            Decoded::Small(n) => Self::from_small(n.cbrt()),
            Decoded::Big(n) => Self::from_big(Roots::cbrt(&n)),
        })
    }

    fn nth_root(&self, n: u32) -> Self {
        self.with_decoded(|encoded| match encoded {
            Decoded::Small(x) => Self::from_small(x.nth_root(n)),
            Decoded::Big(x) => Self::from_big(Roots::nth_root(&x, n)),
        })
    }

    fn trailing_zeros(&self) -> Option<u64> {
        self.with_big_cow(|cow| cow.trailing_zeros())
    }

    fn iter_u32_digits(&self) -> impl BigNumberDigits<'_, u32> {
        self.with_big_cow(|cow| cow.iter_u32_digits().collect::<Vec<_>>().into_iter())
    }

    fn iter_u64_digits(&self) -> impl BigNumberDigits<'_, u64> {
        self.with_big_cow(|cow| cow.iter_u64_digits().collect::<Vec<_>>().into_iter())
    }

    fn modinv(&self, modulus: &Self) -> Option<Self> {
        self.with_big_cows(modulus, |lhs, rhs| lhs.modinv(&rhs).map(Self::from_big))
    }

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

pub trait BorrowingEncoding<'a>: Encoding<'a> + 'a {
    type WithLifetime<'b>: Encoding<'b, Small = Self::Small, Big = Self::Big>
    where
        'a: 'b;

    /// Converts this encoding into a version with a shorter lifetime.
    fn borrow<'b>(&'b self) -> Self::WithLifetime<'b>
    where
        'a: 'b;
}

// =============================================================================
// Implementations of `Decode` for various foreign and built-in types
// =============================================================================

impl<'a, S> Decode<'a, S> for Cow<'a, S::Big>
where
    S: SmallNumber,
{
    fn kind() -> EncodingKind {
        EncodingKind::Cow
    }

    fn decode(self) -> Decoded<S, Cow<'a, S::Big>> {
        Decoded::Big(self)
    }

    fn into_big_cow(self) -> Cow<'a, S::Big> {
        self
    }

    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T {
        f(Decoded::Big(Cow::Borrowed(self.as_ref())))
    }

    fn owns_bignum(&self) -> bool {
        match self {
            Cow::Borrowed(_) => false,
            Cow::Owned(_) => true,
        }
    }
}

impl<'a, S> Decode<'a, S> for Rc<S::Big>
where
    S: SmallNumber,
{
    fn kind() -> EncodingKind {
        EncodingKind::Rc
    }

    fn decode(self) -> Decoded<S, Cow<'a, S::Big>> {
        Decoded::Big(Cow::Owned((*self).clone()))
    }

    fn into_big_cow(self) -> Cow<'a, S::Big> {
        match Rc::try_unwrap(self) {
            Ok(b) => Cow::Owned(b),
            Err(rc) => Cow::Owned((*rc).clone()),
        }
    }

    fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T {
        f(Decoded::Big(Cow::Borrowed(self.as_ref())))
    }

    fn owns_bignum(&self) -> bool {
        true
    }
}

duplicate_prims! {
    impl<'a, S> Decode<'a, S> for prim
    where
        S: SmallNumber,
        S::Big: BigNumber + From<prim>
    {
        fn kind() -> EncodingKind {
            EncodingKind::Primitive
        }

        fn decode(self) -> Decoded<S, Cow<'a, S::Big>> {
            #[allow(irrefutable_let_patterns)]
            #[allow(clippy::unnecessary_fallible_conversions)]
            if let Ok(small) = S::try_from(self) {
                Decoded::Small(small)
            } else {
                Decoded::Big(Cow::Owned(S::Big::from(self)))
            }
        }

        fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T {
            #[allow(clippy::unnecessary_fallible_conversions)]
            match S::try_from(*self) {
                Ok(small) => f(Decoded::Small(small)),
                Err(_) => f(Decoded::Big(Cow::Owned(S::Big::from(*self)))),
            }
        }

        fn owns_bignum(&self) -> bool {
            false
        }
    }

    impl<'a, S: SmallNumber> Decode<'a, S> for &prim
    where
        S::Big: From<prim>,
        S: TryFrom<prim>
    {
        fn kind() -> EncodingKind {
            EncodingKind::Primitive
        }

        fn decode(self) -> Decoded<S, Cow<'a, S::Big>> {
            #[allow(irrefutable_let_patterns)]
            #[allow(clippy::unnecessary_fallible_conversions)]
            if let Ok(small) = S::try_from(*self) {
                Decoded::Small(small)
            } else {
                Decoded::Big(Cow::Owned(S::Big::from(*self)))
            }
        }

        fn with_decoded<T>(&self, f: impl FnOnce(Decoded<S, Cow<S::Big>>) -> T) -> T {
            #[allow(clippy::unnecessary_fallible_conversions)]
            match S::try_from(**self) {
                Ok(small) => f(Decoded::Small(small)),
                Err(_) => f(Decoded::Big(Cow::Owned(S::Big::from(**self)))),
            }
        }

        fn owns_bignum(&self) -> bool {
            false
        }
    }
}
