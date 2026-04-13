#![allow(unused_imports)]
use crate::big_number::{BigNumber, BigSigned};
use crate::encoding::{Decode, Decoded, Encoding, EncodingKind};
use crate::signed::Int;
use crate::small_num::SmallNumber as _;
use crate::unsigned::Uint;
use crate::{
    duplicate_arith_ops, duplicate_bit_ops, duplicate_generic_big_num, duplicate_iprims,
    duplicate_prims, duplicate_shift_ops, duplicate_uprims, duplicate_uprims_and_iprims_if_signed,
};
use num_bigint::{BigInt, BigUint};
use num_integer::Integer;
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedSub, One, Pow, ToPrimitive,
};
use paste::paste;
use std::borrow::Cow;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

// MARK: Meta-Operator Trait Definitions
// -----------------------------------------------------------------------------
trait ArithOp<'e, E: Encoding<'e>> {
    fn on_big_small(lhs: Cow<E::Big>, rhs: E::Small) -> E::Big;
    fn on_small(lhs: E::Small, rhs: E::Small) -> Result<E::Small, ()>;
    fn on_small_big(lhs: E::Small, rhs: Cow<E::Big>) -> E::Big;
    fn on_big(lhs: Cow<E::Big>, rhs: Cow<E::Big>) -> E::Big;
    fn update_big(lhs: &mut E::Big, rhs: Cow<E::Big>);
    fn update_small(lhs: &mut E::Big, rhs: E::Small);

    /// Calls a version of the binary operator that returns a new number.
    fn call<L, R>(lhs: L, rhs: R) -> E
    where
        L: Decode<'e, E::Small>,
        R: Decode<'e, E::Small>,
    {
        match E::kind() {
            EncodingKind::Cow => match (lhs.decode(), rhs.decode()) {
                (Decoded::Small(lhs), Decoded::Small(rhs)) => {
                    if let Ok(out) = Self::on_small(lhs, rhs) {
                        E::from_small(out)
                    } else {
                        E::from_big(Self::on_big_small(Cow::Owned(lhs.to_big()), rhs))
                    }
                }
                (Decoded::Small(small_lhs), Decoded::Big(big_rhs)) => {
                    E::from_big(Self::on_small_big(small_lhs, big_rhs))
                }
                (Decoded::Big(big_lhs), Decoded::Small(small_rhs)) => {
                    E::from_big(Self::on_big_small(big_lhs, small_rhs))
                }
                (Decoded::Big(big_lhs), Decoded::Big(big_rhs)) => {
                    E::from_big(Self::on_big(big_lhs, big_rhs))
                }
            },
            _ => lhs.with_decoded(|lhs| {
                rhs.with_decoded(|rhs| match (lhs, rhs) {
                    (Decoded::Small(lhs), Decoded::Small(rhs)) => {
                        if let Ok(out) = Self::on_small(lhs, rhs) {
                            E::from_small(out)
                        } else {
                            E::from_big(Self::on_big_small(Cow::Owned(lhs.to_big()), rhs))
                        }
                    }
                    (Decoded::Small(small_lhs), Decoded::Big(big_rhs)) => {
                        E::from_big(Self::on_small_big(small_lhs, big_rhs))
                    }
                    (Decoded::Big(big_lhs), Decoded::Small(small_rhs)) => {
                        E::from_big(Self::on_big_small(big_lhs, small_rhs))
                    }
                    (Decoded::Big(big_lhs), Decoded::Big(big_rhs)) => {
                        E::from_big(Self::on_big(big_lhs, big_rhs))
                    }
                })
            }),
        }
    }

    /// Calls a version of the binary operator that updates a bigint argument in place.
    fn call_update<R>(lhs: &mut E, rhs: R)
    where
        R: Decode<'e, E::Small>,
    {
        lhs.update_encoding(|encoding| match encoding {
            Decoded::Small(small_lhs) => match rhs.decode() {
                Decoded::Small(small_rhs) => match Self::on_small(*small_lhs, small_rhs) {
                    Ok(out) => *encoding = Decoded::Small(out),
                    Err(()) => {
                        *encoding = Decoded::Big(Cow::Owned(Self::on_small_big(
                            *small_lhs,
                            Cow::Owned(small_rhs.to_big()),
                        )));
                    }
                },
                Decoded::Big(big_rhs) => {
                    *encoding = Decoded::Big(Cow::Owned(Self::on_small_big(*small_lhs, big_rhs)));
                }
            },
            Decoded::Big(big_lhs) => match rhs.decode() {
                Decoded::Small(small_rhs) => {
                    Self::update_small(big_lhs.to_mut(), small_rhs);
                }
                Decoded::Big(big_rhs) => {
                    Self::update_big(big_lhs.to_mut(), big_rhs);
                }
            },
        });
    }
}

trait BitOp<'e, E: Encoding<'e>> {
    fn on_big(lhs: Cow<E::Big>, rhs: Cow<E::Big>) -> E::Big;
    fn update_big(lhs: &mut E::Big, rhs: Cow<E::Big>);

    fn call<L, R>(lhs: L, rhs: R) -> E
    where
        L: Decode<'e, E::Small>,
        R: Decode<'e, E::Small>,
    {
        E::from_big(lhs.with_big_cows(&rhs, |lhs, rhs| Self::on_big(lhs, rhs)))
    }

    fn call_update<'a, 'c, R>(lhs: &'a mut E, rhs: R)
    where
        R: Decode<'c, E::Small>,
    {
        lhs.update_encoding(|encoding| match encoding {
            Decoded::Small(small_lhs) => {
                *encoding = Decoded::Big(Cow::Owned(Self::on_big(
                    Cow::Owned(small_lhs.to_big()),
                    rhs.into_big_cow(),
                )));
            }
            Decoded::Big(big_lhs) => {
                Self::update_big(big_lhs.to_mut(), rhs.into_big_cow());
            }
        });
    }
}
trait ShiftOp<'e, E: Encoding<'e>> {
    duplicate_prims! {
        paste! {
            fn [<on_big_ prim>](lhs: Cow<E::Big>, rhs: prim) -> E::Big;
            fn [<update_big_ prim>](lhs: &mut E::Big, rhs: prim);

            fn [<call_ prim>]<'a, L>(lhs: L, rhs: prim) -> E
            where
                L: Decode<'a, E::Small>,
            {
                E::from_big(Self::[<on_big_ prim>](lhs.into_big_cow(), rhs))
            }

            #[inline]
            fn [<call_update_big_ prim>](lhs: &mut E, rhs: prim) {
                lhs.update_encoding(|encoding| match encoding {
                    Decoded::Small(small_lhs) => {
                        *encoding = Decoded::Big(Cow::Owned(Self::[<on_big_ prim>](
                            Cow::Owned(small_lhs.to_big()),
                            rhs,
                        )));
                    }
                    Decoded::Big(big_lhs) => {
                        Self::[<update_big_ prim>](big_lhs.to_mut(), rhs);
                    }
                });
            }
        }
    }
}

// This trait has only one implementation, so it doesn't need to be generic over the operator.
// Using a trait isn't stricly necessary, but it helps organize the code in a way that's consistent
// with the other operations.
trait PowOpTrait<'e, E: Encoding<'e>> {
    fn on_big_small(lhs: Cow<E::Big>, rhs: <E::Unsigned as Encoding<'e>>::Small) -> E::Big;
    fn on_small(lhs: E::Small, rhs: <E::Unsigned as Encoding<'e>>::Small) -> Result<E::Small, ()>;
    fn on_big(lhs: Cow<E::Big>, rhs: Cow<<E::Unsigned as Encoding<'e>>::Big>) -> E::Big;

    /// Calls a version of the binary operator that returns a new number.
    fn call<L, R>(lhs: L, rhs: R) -> E
    where
        L: Decode<'e, E::Small>,
        R: Decode<'e, <E::Unsigned as Encoding<'e>>::Small>,
    {
        lhs.with_decoded(|lhs| {
            rhs.with_decoded(|rhs| match (lhs, rhs) {
                (Decoded::Small(lhs), Decoded::Small(rhs)) => {
                    if let Ok(out) = Self::on_small(lhs, rhs) {
                        E::from_small(out)
                    } else {
                        E::from_big(Self::on_big_small(Cow::Owned(lhs.to_big()), rhs))
                    }
                }
                (Decoded::Small(small_lhs), Decoded::Big(_)) => {
                    if small_lhs.is_one() {
                        E::from_small(E::Small::one())
                    } else {
                        panic!("Exponentiation would overflow memory")
                    }
                }
                (Decoded::Big(big_lhs), Decoded::Small(small_rhs)) => {
                    E::from_big(Self::on_big_small(big_lhs, small_rhs))
                }
                (Decoded::Big(big_lhs), Decoded::Big(big_rhs)) => {
                    E::from_big(Self::on_big(big_lhs, big_rhs))
                }
            })
        })
    }
}

// MARK: Meta-Operator Trait Implementations
// -----------------------------------------------------------------------------
duplicate_arith_ops! {
    paste! {
        struct [<op_trait Op>];

        impl<'e, E: Encoding<'e>> ArithOp<'e, E> for [<op_trait Op>] {

            fn on_big(lhs: Cow<E::Big>, rhs: Cow<E::Big>) -> E::Big {
                match (lhs, rhs) {
                    (Cow::Borrowed(lhs), Cow::Borrowed(rhs)) => E::Big::[<op_fn _ref_self_and_ref_self>](lhs, rhs),
                    (Cow::Borrowed(lhs), Cow::Owned(rhs)) => E::Big::[<op_fn _ref_self_and_self>](lhs, rhs),
                    (Cow::Owned(lhs), Cow::Borrowed(rhs)) => E::Big::[<op_fn _self_and_ref_self>](lhs, rhs),
                    (Cow::Owned(lhs), Cow::Owned(rhs)) => E::Big::[<op_fn _self_and_self>](lhs, rhs),
                }
            }

            fn on_small(lhs: E::Small, rhs: E::Small) -> Result<E::Small, ()> {
                lhs.[<checked_ op_fn>](&rhs).ok_or(())
            }

            fn on_big_small(lhs: Cow<E::Big>, rhs: E::Small) -> E::Big {
                match lhs {
                    Cow::Borrowed(lhs) => E::Small::[<op_fn _big_ref_small>](lhs, rhs),
                    Cow::Owned(lhs) => E::Small::[<op_fn _big_small>](lhs, rhs),
                }
            }

            fn on_small_big(lhs: E::Small, rhs: Cow<E::Big>) -> E::Big {
                match rhs {
                    Cow::Borrowed(rhs) => E::Small::[<op_fn _small_big_ref>](lhs, rhs),
                    Cow::Owned(rhs) => E::Small::[<op_fn _small_big>](lhs, rhs),
                }
            }

            fn update_big(lhs: &mut E::Big, rhs: Cow<E::Big>) {
                match rhs {
                    Cow::Borrowed(rhs) => E::Big::[<op_fn _assign_ref_self>](lhs, rhs),
                    Cow::Owned(rhs) => E::Big::[<op_fn _assign_self>](lhs, rhs),
                }

            }

            fn update_small(lhs: &mut E::Big, rhs: E::Small) {
                E::Small::[<op_fn _assign_small>](lhs, rhs);
            }
        }
    }
}

duplicate_bit_ops! {
    paste! {
        struct [<op_trait Op>];

        impl<'e, E: Encoding<'e>> BitOp<'e, E> for [<op_trait Op>] {

            fn on_big(lhs: Cow<E::Big>, rhs: Cow<E::Big>) -> E::Big
            {
                match (lhs, rhs) {
                    (Cow::Borrowed(lhs), Cow::Borrowed(rhs)) => E::Big::[<op_fn _ref_self_and_ref_self>](lhs, rhs),
                    (Cow::Borrowed(lhs), Cow::Owned(rhs)) => E::Big::[<op_fn _ref_self_and_self>](lhs, rhs),
                    (Cow::Owned(lhs), Cow::Borrowed(rhs)) => E::Big::[<op_fn _self_and_ref_self>](lhs, rhs),
                    (Cow::Owned(lhs), Cow::Owned(rhs)) => E::Big::[<op_fn _self_and_self>](lhs, rhs),
                }
            }

            fn update_big(lhs: &mut E::Big, rhs: Cow<E::Big>)
            {
                match rhs {
                    Cow::Borrowed(rhs) => E::Big::[<op_fn _assign_ref_self>](lhs, rhs),
                    Cow::Owned(rhs) => E::Big::[<op_fn _assign_self>](lhs, rhs),
                }
            }
        }
    }
}

duplicate_shift_ops! {
    paste! { struct [<op_trait Op>]; }

    impl<'e, E: Encoding<'e>> ShiftOp<'e, E> for paste! { [<op_trait Op>] } {
        duplicate_prims! {
            paste! {
                fn [<on_big_ prim>](lhs: Cow<E::Big>, rhs: prim) -> E::Big {
                    match lhs {
                        Cow::Borrowed(lhs) => E::Big::[<op_fn _ref_self_and_ prim>](lhs, rhs),
                        Cow::Owned(lhs) => E::Big::[<op_fn _self_and_ prim>](lhs, rhs),
                    }
                }

                fn [<update_big_ prim>](lhs: &mut E::Big, rhs: prim) {
                    E::Big::[<op_fn _assign_ prim>](lhs, rhs);
                }
            }
        }
    }
}

struct PowOp;

fn big_pow<L, R>(lhs: Cow<L>, rhs: &R) -> L
where
    L: BigNumber,
    R: ToPrimitive + Integer,
{
    if let Some(rhs) = rhs.to_u32() {
        lhs.pow(rhs)
    } else if lhs.is_one() {
        lhs.into_owned()
    } else if lhs.is_minus_one() {
        if rhs.is_even() {
            L::one()
        } else {
            lhs.into_owned()
        }
    } else {
        panic!("Exponentiation would overflow memory")
    }
}

impl<'e, E: Encoding<'e>> PowOpTrait<'e, E> for PowOp {
    fn on_big(lhs: Cow<E::Big>, rhs: Cow<<E::Unsigned as Encoding<'e>>::Big>) -> E::Big {
        big_pow(lhs, rhs.as_ref())
    }

    fn on_small(lhs: E::Small, rhs: <E::Unsigned as Encoding<'e>>::Small) -> Result<E::Small, ()> {
        if let Some(rhs) = rhs.to_u32()
            && let (result, false) = lhs.overflowing_pow(rhs)
        {
            Ok(result)
        } else {
            Err(())
        }
    }

    fn on_big_small(lhs: Cow<E::Big>, rhs: <E::Unsigned as Encoding<'e>>::Small) -> E::Big {
        big_pow(lhs, &rhs)
    }
}

// MARK: Operator Trait Implementations
// -----------------------------------------------------------------------------
duplicate_generic_big_num! {

duplicate_arith_ops! {
    paste! {
        impl<'a, T, E: Encoding<'a, Big = RawType>> op_trait<T> for GenericBigNum<'a, E>
        where
            T: Decode<'a, E::Small>,
        {
            type Output = GenericBigNum<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, T, E: Encoding<'a, Big = RawType>> op_trait<T> for &GenericBigNum<'a, E>
        where
            T: Decode<'a, E::Small>,
        {
            type Output = GenericBigNum<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, T, E: Encoding<'a, Big = RawType>> [<op_trait Assign>]<T> for GenericBigNum<'a, E>
        where
            T: Decode<'a, E::Small>
        {
            fn [<op_fn _assign>](&mut self, rhs: T) {
                [<op_trait Op>]::call_update(self, rhs);
            }
        }
    }

    duplicate_iprims! {
        paste! {
            impl<'a, E: Encoding<'a, Big = RawType>> op_trait<GenericBigNum<'a, E>> for prim
            where
                E::Big: BigSigned,
            {
                type Output = GenericBigNum<'a, E>;

                #[inline(never)]
                fn op_fn(self, rhs: GenericBigNum<'a, E>) -> Self::Output {
                    [<op_trait Op>]::call(self, rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = RawType>> op_trait<GenericBigNum<'a, E>> for &prim
            where
                E::Big: BigSigned,
            {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: GenericBigNum<'a, E>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = RawType>> op_trait<&GenericBigNum<'a, E>> for prim
            where
                E::Big: BigSigned,
            {
                type Output = GenericBigNum<'a, E>;

                #[inline(never)]
                fn op_fn(self, rhs: &GenericBigNum<'a, E>) -> Self::Output {
                    [<op_trait Op>]::call(self, rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = RawType>> op_trait<&GenericBigNum<'a, E>> for &prim
            where
                E::Big: BigSigned,
            {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: &GenericBigNum<'a, E>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }
        }
    }

    duplicate_uprims! {
        paste! {
            impl<'a, E: Encoding<'a, Big = RawType>> op_trait<GenericBigNum<'a, E>> for prim {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: GenericBigNum<'a, E>) -> Self::Output {
                    [<op_trait Op>]::call(self, rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = RawType>> op_trait<GenericBigNum<'a, E>> for &prim {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: GenericBigNum<'a, E>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = RawType>> op_trait<&GenericBigNum<'a, E>> for prim {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: &GenericBigNum<'a, E>) -> Self::Output {
                    [<op_trait Op>]::call(self, rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = RawType>> op_trait<&GenericBigNum<'a, E>> for &prim {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: &GenericBigNum<'a, E>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }
        }
    }
}

duplicate_bit_ops! {
    paste! {
        impl<'a, T, E: Encoding<'a, Big = RawType>> op_trait<T> for GenericBigNum<'a, E>
        where
            T: Decode<'a, E::Small>,
        {
            type Output = GenericBigNum<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, T, E: Encoding<'a, Big = RawType>> op_trait<T> for &GenericBigNum<'a, E>
        where
            T: Decode<'a, E::Small>,
        {
            type Output = GenericBigNum<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, T, E: Encoding<'a, Big = RawType>> [<op_trait Assign>]<T> for GenericBigNum<'a, E>
        where
            T: Decode<'a, E::Small>
        {
            fn [<op_fn _assign>](&mut self, rhs: T) {
                [<op_trait Op>]::call_update(self, rhs);
            }
        }
    }
}

duplicate_shift_ops! {
    duplicate_prims! {
        paste! {
            impl<'a, E: Encoding<'a, Big = RawType>> op_trait<prim> for GenericBigNum<'a, E> {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: prim) -> Self::Output {
                    [<op_trait Op>]::[<call_ prim>](self, rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = RawType>> op_trait<&prim> for GenericBigNum<'a, E> {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: &prim) -> Self::Output {
                    self.op_fn(*rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = RawType>> op_trait<prim> for &GenericBigNum<'a, E> {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: prim) -> Self::Output {
                    [<op_trait Op>]::[<call_ prim>](self, rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = RawType>> op_trait<&prim> for &GenericBigNum<'a, E> {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: &prim) -> Self::Output {
                    self.op_fn(*rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = RawType>> [<op_trait Assign>]<prim> for GenericBigNum<'a, E> {
                fn [<op_fn _assign>](&mut self, rhs: prim) {
                    [<op_trait Op>]::[<call_update_big_ prim>](self, rhs);
                }
            }

            impl<'a, E: Encoding<'a, Big = RawType>> [<op_trait Assign>]<&prim> for GenericBigNum<'a, E> {
                fn [<op_fn _assign>](&mut self, rhs: &prim) {
                    self.[<op_fn _assign>](*rhs);
                }
            }
        }
    }
}

} // duplicate_generic_big_num!

// MARK: Pow Operator Implementations
// -----------------------------------------------------------------------------
duplicate_generic_big_num! {

impl<'a, E: Encoding<'a, Big = RawType>> Pow<Uint<'a, E::Unsigned>> for GenericBigNum<'a, E> {
    type Output = GenericBigNum<'a, E>;

    fn pow(self, rhs: Uint<'a, E::Unsigned>) -> Self::Output {
        PowOp::call(self, rhs)
    }
}

impl<'a, E: Encoding<'a, Big = RawType>> Pow<&Uint<'a, E::Unsigned>> for GenericBigNum<'a, E> {
    type Output = GenericBigNum<'a, E>;

    fn pow(self, rhs: &Uint<'a, E::Unsigned>) -> Self::Output {
        PowOp::call(self, rhs)
    }
}

impl<'a, E: Encoding<'a, Big = RawType>> Pow<Uint<'a, E::Unsigned>> for &GenericBigNum<'a, E> {
    type Output = GenericBigNum<'a, E>;

    fn pow(self, rhs: Uint<'a, E::Unsigned>) -> Self::Output {
        PowOp::call(self, rhs)
    }
}

impl<'a, E: Encoding<'a, Big = RawType>> Pow<&Uint<'a, E::Unsigned>> for &GenericBigNum<'a, E> {
    type Output = GenericBigNum<'a, E>;

    fn pow(self, rhs: &Uint<'a, E::Unsigned>) -> Self::Output {
        PowOp::call(self, rhs)
    }
}

duplicate_uprims! {
    paste! {
        impl<'a, E: Encoding<'a, Big = RawType>> Pow<prim> for GenericBigNum<'a, E> {
            type Output = GenericBigNum<'a, E>;

            fn pow(self, rhs: prim) -> Self::Output {
                PowOp::call(self, rhs)
            }
        }

        impl<'a, E: Encoding<'a, Big = RawType>> Pow<&prim> for GenericBigNum<'a, E> {
            type Output = GenericBigNum<'a, E>;

            fn pow(self, rhs: &prim) -> Self::Output {
                PowOp::call(self, rhs)
            }
        }

        impl<'a, E: Encoding<'a, Big = RawType>> Pow<prim> for &GenericBigNum<'a, E> {
            type Output = GenericBigNum<'a, E>;

            fn pow(self, rhs: prim) -> Self::Output {
                PowOp::call(self, rhs)
            }
        }

        impl<'a, E: Encoding<'a, Big = RawType>> Pow<&prim> for &GenericBigNum<'a, E> {
            type Output = GenericBigNum<'a, E>;

            fn pow(self, rhs: &prim) -> Self::Output {
                PowOp::call(self, rhs)
            }
        }
    }
}

} // duplicate_generic_big_num!

// MARK: Tests
#[cfg(test)]
mod test {
    use std::fmt::Display;

    use super::*;
    use crate::duplicate_arith_and_bit_ops;
    use crate::duplicate_generic_bignum_types;
    use crate::signed::Int;
    use num_bigint::BigInt;
    use num_traits::Zero;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    fn always(_lhs: &impl BigNumber, _rhs: &impl BigNumber) -> bool {
        true
    }

    fn nonzero_rhs(_lhs: &impl BigNumber, rhs: &impl BigNumber) -> bool {
        !rhs.is_zero()
    }

    fn can_subtract<T: BigNumber>(lhs: &T, rhs: &T) -> bool {
        T::is_signed() || lhs >= rhs
    }

    duplicate_generic_bignum_types! { mod bignum_tag {
        use super::*;

        macro_rules! test_bin_op {
            ($expected: ident, $lhs: ident, $rhs: ident, $op_trait: ident, $op_fn: ident, $op_test_pred: ident) => {
                paste! {
                    let big_lhs = &RawType::from($lhs.clone());
                    let big_rhs = &RawType::from($rhs.clone());

                    if !$op_test_pred(big_lhs, big_rhs) {
                        return TestResult::discard();
                    }
                    let $expected = $op_trait::$op_fn(big_lhs, big_rhs);
                    let actual1 = $op_trait::$op_fn($lhs.clone(), $rhs.clone()).into();
                    assert_eq!($expected, actual1);
                    let actual2 = $op_trait::$op_fn($lhs.clone(), &$rhs).into();
                    assert_eq!($expected, actual2);
                    let actual3 = $op_trait::$op_fn(&$lhs, $rhs.clone()).into();
                    assert_eq!($expected, actual3);
                    let actual4 = $op_trait::$op_fn(&$lhs, &$rhs).into();
                    assert_eq!($expected, actual4);
                }
            };
        }

        macro_rules! test_bin_op_with_assign {
            ($expected: ident, $lhs: ident, $rhs: ident, $op_trait: ident, $op_fn: ident, $op_test_pred: ident) => {
                paste! {
                    test_bin_op!($expected, $lhs, $rhs, $op_trait, $op_fn, $op_test_pred);
                    let mut actual5 = $lhs.clone();
                    [<$op_trait Assign>]::[<$op_fn _assign>](&mut actual5, $rhs.clone());
                    assert_eq!($expected, actual5.into());
                }
            };
        }

        macro_rules! test_bin_op_with_ref_assign {
            ($expected: ident, $lhs: ident, $rhs: ident, $op_trait: ident, $op_fn: ident, $op_test_pred: ident) => {
                paste! {
                    test_bin_op_with_assign!($expected, $lhs, $rhs, $op_trait, $op_fn, $op_test_pred);
                    let mut actual6 = $lhs.clone();
                    [<$op_trait Assign>]::[<$op_fn _assign>](&mut actual6, &$rhs);
                    assert_eq!($expected, actual6.into());
                }
            };
        }

        duplicate_arith_ops! {
            paste! {
                #[quickcheck]
                fn [<test_ op_fn>](lhs: EncodedType, rhs: EncodedType) -> TestResult {
                    test_bin_op_with_ref_assign!(expected,lhs, rhs, op_trait, op_fn, op_test_pred);
                    TestResult::passed()
                }
            }
        }
        duplicate_bit_ops! {
            paste! {
                #[quickcheck]
                fn [<test_ op_fn>](lhs: EncodedType, rhs: EncodedType) -> TestResult {
                    test_bin_op_with_assign!(expected, lhs, rhs, op_trait, op_fn, op_test_pred);
                    TestResult::passed()
                }
            }
        }

        duplicate_shift_ops! {
            duplicate_prims! {
                paste! {
                    #[quickcheck]
                    fn [<test_ op_fn _ prim _rhs>](lhs: EncodedType, rhs: u16) -> TestResult{
                        #[allow(irrefutable_let_patterns)]
                        if let Ok(rhs) = prim::try_from(rhs) {
                            let big_lhs = &RawType::from(lhs.clone());

                            #[allow(unused_comparisons)]
                            let nonnegative_rhs = rhs >= 0;
                            assert!(nonnegative_rhs, "shift amount must be non-negative");

                            let expected = op_trait::op_fn(big_lhs, rhs);
                            let actual1 = op_trait::op_fn(lhs.clone(), rhs).into();
                            assert_eq!(expected, actual1);
                            let actual2 = op_trait::op_fn(lhs.clone(), &rhs).into();
                            assert_eq!(expected, actual2);
                            let actual3 = op_trait::op_fn(&lhs, rhs).into();
                            assert_eq!(expected, actual3,);
                            let actual4 = op_trait::op_fn(&lhs, &rhs).into();
                            assert_eq!(expected, actual4);
                            let mut actual5 = lhs.clone();
                            [<op_trait Assign>]::[<op_fn _assign>](&mut actual5, rhs);
                            assert_eq!(expected, actual5.clone().into());
                            TestResult::passed()
                        } else {
                            TestResult::discard()
                        }
                    }
                }
            }
        }

        duplicate_arith_ops! {
            duplicate_uprims_and_iprims_if_signed! { signedness;
                paste! {
                    #[quickcheck]
                    fn [<test_ op_fn _ prim _lhs>](lhs: prim, rhs: EncodedType) -> TestResult{
                        test_bin_op!(expected, lhs, rhs, op_trait, op_fn, op_test_pred);
                        TestResult::passed()
                    }
                    #[quickcheck]
                    fn [<test_ op_fn _ prim _rhs>](lhs: EncodedType, rhs: prim) -> TestResult {
                        test_bin_op_with_assign!(expected, lhs, rhs, op_trait, op_fn, op_test_pred);
                        TestResult::passed()
                    }
                }
            }
        }

        duplicate_uprims! {
            paste! {
                // The #[quickcheck] macro gets confused here, so we have to call the inner function manually.
                #[test]
                fn [<test_pow_ prim>]() {
                    fn inner(lhs: EncodedType, rhs: u8) -> TestResult {
                        let rhs = rhs % 64; // limit the exponent to avoid long test times and potential OOM errors
                        #[allow(irrefutable_let_patterns)]
                        #[allow(clippy::unnecessary_fallible_conversions)]
                        if let Ok(rhs) = prim::try_from(rhs) {
                            let big_lhs = &EncodedType::from(lhs.clone());
                            let expected = Pow::pow(big_lhs, rhs);
                            let actual1 = EncodedType::from(Pow::pow(lhs.clone(), rhs));
                            let actual2 = EncodedType::from(Pow::pow(lhs, rhs));
                            assert_eq!(expected, actual1);
                            assert_eq!(expected, actual2);
                            TestResult::passed()
                        } else {
                            TestResult::discard()
                        }
                    }
                    quickcheck::quickcheck(inner as fn(_, _) -> _);
                }
            }
        }
    } }
}
