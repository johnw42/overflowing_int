use num_bigint::{BigInt, BigUint};

use crate::{encoding::Encoding, signed::Int, small_num::SmallNumber, unsigned::Uint};

pub trait BorrowBignum {
    type Borrowed<'a>
    where
        Self: 'a;
    type Static;

    fn borrow<'a>(&'a self) -> Self::Borrowed<'a>;

    fn into_static(self) -> Self::Static;
}

impl<'a> BorrowBignum for BigInt {
    type Borrowed<'b>
        = &'b BigInt
    where
        Self: 'b;
    type Static = Self;

    #[inline]
    fn borrow<'b>(&'b self) -> Self::Borrowed<'b> {
        self
    }

    #[inline]
    fn into_static(self) -> Self::Static {
        self
    }
}

impl<'a> BorrowBignum for BigUint {
    type Borrowed<'b>
        = &'b BigUint
    where
        Self: 'b;
    type Static = Self;

    #[inline]
    fn borrow<'b>(&'b self) -> Self::Borrowed<'b> {
        self
    }

    #[inline]
    fn into_static(self) -> Self::Static {
        self
    }
}

impl<'a, S, E> BorrowBignum for Int<'a, E>
where
    E: Encoding<'a, Small = S, Big = BigInt>,
    S: SmallNumber,
{
    type Borrowed<'b>
        = <Self as Encoding<'a>>::WithLifetime<'b>
    where
        Self: 'b;
    type Static = Int<'static, E::Static>;

    #[inline]
    fn borrow<'b>(&'b self) -> Self::Borrowed<'b> {
        Int::borrow(self)
    }

    #[inline]
    fn into_static(self) -> Self::Static {
        Int::into_static(self)
    }
}

impl<'a, S, E> BorrowBignum for Uint<'a, E>
where
    E: Encoding<'a, Small = S, Big = BigUint>,
    S: SmallNumber,
{
    type Borrowed<'b>
        = <Self as Encoding<'a>>::WithLifetime<'b>
    where
        Self: 'b;
    type Static = Uint<'static, E::Static>;

    #[inline]
    fn borrow<'b>(&'b self) -> Self::Borrowed<'b> {
        Uint::borrow(self)
    }

    #[inline]
    fn into_static(self) -> Self::Static {
        Uint::into_static(self)
    }
}
