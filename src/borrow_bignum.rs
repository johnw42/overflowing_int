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
        Self: 'a;
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
        Self: 'a;
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

impl<'enc, S, E> BorrowBignum for Int<'enc, E>
where
    E: Encoding<'enc, Small = S, Big = BigInt>,
    S: SmallNumber,
{
    type Borrowed<'a>
        = <Self as Encoding<'enc>>::WithLifetime<'a>
    where
        Self: 'a;
    type Static = Int<'static, E::Static>;

    #[inline]
    fn borrow<'a>(&'a self) -> Self::Borrowed<'a> {
        Int::borrow(self)
    }

    #[inline]
    fn into_static(self) -> Self::Static {
        Int::into_static(self)
    }
}

impl<'enc, S, E> BorrowBignum for Uint<'enc, E>
where
    E: Encoding<'enc, Small = S, Big = BigUint>,
    S: SmallNumber,
{
    type Borrowed<'a>
        = <Self as Encoding<'enc>>::WithLifetime<'a>
    where
        Self: 'a;
    type Static = Uint<'static, E::Static>;

    #[inline]
    fn borrow<'a>(&'a self) -> Self::Borrowed<'a> {
        Uint::borrow(self)
    }

    #[inline]
    fn into_static(self) -> Self::Static {
        Uint::into_static(self)
    }
}
