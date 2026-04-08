use crate::big_number::{BigNumber, BigSigned};
use crate::generic_bignum::encoding::{Decode, Decoded, Encoding};
use crate::generic_bignum::signed::GenericSignedBigNum;
use crate::generic_bignum::unsigned::GenericUnsignedBigNum;
use crate::small_num::SmallNumber as _;
use crate::{
    duplicate_arith_ops, duplicate_bit_ops, duplicate_generic_big_num, duplicate_iprims,
    duplicate_prims, duplicate_shift_ops, duplicate_uprims,
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
    #[inline]
    fn call<'a, 'b, L, R>(lhs: L, rhs: R) -> E
    where
        L: Decode<'a, E::Small>,
        R: Decode<'b, E::Small>,
    {
        match (lhs.decode(), rhs.decode()) {
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
        }
    }

    /// Calls a version of the binary operator that updates a bigint argument in place.
    #[inline]
    fn call_update<'a, 'c, R>(lhs: &'a mut E, rhs: R)
    where
        R: Decode<'c, E::Small>,
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

    #[inline]
    fn call<'a, 'b, L, R>(lhs: L, rhs: R) -> E
    where
        L: Decode<'a, E::Small>,
        R: Decode<'b, E::Small>,
    {
        E::from_big(lhs.with_big_cows(&rhs, |lhs, rhs| Self::on_big(lhs, rhs)))
    }

    #[inline]
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
    #[inline]
    fn call<'a, 'b, L, R>(lhs: L, rhs: R) -> E
    where
        L: Decode<'a, E::Small>,
        R: Decode<'b, <E::Unsigned as Encoding<'e>>::Small>,
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
        impl<'a, T, E: Encoding<'a, Big = BigNumType>> op_trait<T> for GenericBigNum<'a, E>
        where
            T: Decode<'a, E::Small>,
        {
            type Output = GenericBigNum<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, T, E: Encoding<'a, Big = BigNumType>> op_trait<T> for &GenericBigNum<'a, E>
        where
            T: Decode<'a, E::Small>,
        {
            type Output = GenericBigNum<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, T, E: Encoding<'a, Big = BigNumType>> [<op_trait Assign>]<T> for GenericBigNum<'a, E>
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
            impl<'a, E: Encoding<'a, Big = BigNumType>> op_trait<GenericBigNum<'a, E>> for prim
            where
                E::Big: BigSigned,
            {
                type Output = GenericBigNum<'a, E>;

                #[inline(never)]
                fn op_fn(self, rhs: GenericBigNum<'a, E>) -> Self::Output {
                    [<op_trait Op>]::call(self, rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = BigNumType>> op_trait<GenericBigNum<'a, E>> for &prim
            where
                E::Big: BigSigned,
            {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: GenericBigNum<'a, E>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = BigNumType>> op_trait<&GenericBigNum<'a, E>> for prim
            where
                E::Big: BigSigned,
            {
                type Output = GenericBigNum<'a, E>;

                #[inline(never)]
                fn op_fn(self, rhs: &GenericBigNum<'a, E>) -> Self::Output {
                    [<op_trait Op>]::call(self, rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = BigNumType>> op_trait<&GenericBigNum<'a, E>> for &prim
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
            impl<'a, E: Encoding<'a, Big = BigNumType>> op_trait<GenericBigNum<'a, E>> for prim {
                type Output = GenericBigNum<'a, E>;

                #[inline(never)]
                fn op_fn(self, rhs: GenericBigNum<'a, E>) -> Self::Output {
                    [<op_trait Op>]::call(self, rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = BigNumType>> op_trait<GenericBigNum<'a, E>> for &prim {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: GenericBigNum<'a, E>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = BigNumType>> op_trait<&GenericBigNum<'a, E>> for prim {
                type Output = GenericBigNum<'a, E>;

                #[inline(never)]
                fn op_fn(self, rhs: &GenericBigNum<'a, E>) -> Self::Output {
                    [<op_trait Op>]::call(self, rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = BigNumType>> op_trait<&GenericBigNum<'a, E>> for &prim {
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
        impl<'a, 'b, T, E: Encoding<'a, Big = BigNumType>> op_trait<T> for GenericBigNum<'a, E>
        where
            T: Decode<'b, E::Small>,
        {
            type Output = GenericBigNum<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, 'b, T, E: Encoding<'a, Big = BigNumType>> op_trait<T> for &GenericBigNum<'a, E>
        where
            T: Decode<'b, E::Small>,
        {
            type Output = GenericBigNum<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, 'b, T, E: Encoding<'a, Big = BigNumType>> [<op_trait Assign>]<T> for GenericBigNum<'a, E>
        where
            T: Decode<'b, E::Small>
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
            impl<'a, E: Encoding<'a, Big = BigNumType>> op_trait<prim> for GenericBigNum<'a, E> {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: prim) -> Self::Output {
                    [<op_trait Op>]::[<call_ prim>](self, rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = BigNumType>> op_trait<&prim> for GenericBigNum<'a, E> {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: &prim) -> Self::Output {
                    self.op_fn(*rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = BigNumType>> op_trait<prim> for &GenericBigNum<'a, E> {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: prim) -> Self::Output {
                    [<op_trait Op>]::[<call_ prim>](self, rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = BigNumType>> op_trait<&prim> for &GenericBigNum<'a, E> {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: &prim) -> Self::Output {
                    self.op_fn(*rhs)
                }
            }

            impl<'a, E: Encoding<'a, Big = BigNumType>> [<op_trait Assign>]<prim> for GenericBigNum<'a, E> {
                fn [<op_fn _assign>](&mut self, rhs: prim) {
                    [<op_trait Op>]::[<call_update_big_ prim>](self, rhs);
                }
            }

            impl<'a, E: Encoding<'a, Big = BigNumType>> [<op_trait Assign>]<&prim> for GenericBigNum<'a, E> {
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

impl<'a, E: Encoding<'a, Big = BigNumType>> Pow<GenericUnsignedBigNum<'a, E::Unsigned>> for GenericBigNum<'a, E> {
    type Output = GenericBigNum<'a, E>;

    fn pow(self, rhs: GenericUnsignedBigNum<'a, E::Unsigned>) -> Self::Output {
        PowOp::call(self, rhs)
    }
}

impl<'a, E: Encoding<'a, Big = BigNumType>> Pow<&GenericUnsignedBigNum<'a, E::Unsigned>> for GenericBigNum<'a, E> {
    type Output = GenericBigNum<'a, E>;

    fn pow(self, rhs: &GenericUnsignedBigNum<'a, E::Unsigned>) -> Self::Output {
        PowOp::call(self, rhs)
    }
}

impl<'a, E: Encoding<'a, Big = BigNumType>> Pow<GenericUnsignedBigNum<'a, E::Unsigned>> for &GenericBigNum<'a, E> {
    type Output = GenericBigNum<'a, E>;

    fn pow(self, rhs: GenericUnsignedBigNum<'a, E::Unsigned>) -> Self::Output {
        PowOp::call(self, rhs)
    }
}

impl<'a, E: Encoding<'a, Big = BigNumType>> Pow<&GenericUnsignedBigNum<'a, E::Unsigned>> for &GenericBigNum<'a, E> {
    type Output = GenericBigNum<'a, E>;

    fn pow(self, rhs: &GenericUnsignedBigNum<'a, E::Unsigned>) -> Self::Output {
        PowOp::call(self, rhs)
    }
}

duplicate_uprims! {
    paste! {
        impl<'a, E: Encoding<'a, Big = BigNumType>> Pow<prim> for GenericBigNum<'a, E> {
            type Output = GenericBigNum<'a, E>;

            fn pow(self, rhs: prim) -> Self::Output {
                PowOp::call(self, rhs)
            }
        }

        impl<'a, E: Encoding<'a, Big = BigNumType>> Pow<&prim> for GenericBigNum<'a, E> {
            type Output = GenericBigNum<'a, E>;

            fn pow(self, rhs: &prim) -> Self::Output {
                PowOp::call(self, rhs)
            }
        }

        impl<'a, E: Encoding<'a, Big = BigNumType>> Pow<prim> for &GenericBigNum<'a, E> {
            type Output = GenericBigNum<'a, E>;

            fn pow(self, rhs: prim) -> Self::Output {
                PowOp::call(self, rhs)
            }
        }

        impl<'a, E: Encoding<'a, Big = BigNumType>> Pow<&prim> for &GenericBigNum<'a, E> {
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
    use crate::duplicate_prims_with_signedness;
    use crate::generic_bignum::signed::GenericSignedBigNum;
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

    duplicate_generic_big_num! { mod signedness {
        use super::*;

        pub struct ShiftOpsForType<R, E: Encoding<'static, Big = BigNumType>> {
            pub cbigint_op1: fn(GenericBigNum<'static, E>, R) -> GenericBigNum<'static, E>,
            pub cbigint_op2: fn(GenericBigNum<'static, E>, &R) -> GenericBigNum<'static, E>,
            pub cbigint_op3: fn(&GenericBigNum<'static, E>, R) -> GenericBigNum<'static, E>,
            pub cbigint_op4: fn(&GenericBigNum<'static, E>, &R) -> GenericBigNum<'static, E>,
            pub op_assign1: fn(&mut GenericBigNum<'static, E>, R),
            pub bigint_op: fn(&E::Big, R) -> E::Big,
        }

        pub struct BinOpsForTypes<L, R, E: Encoding<'static, Big = BigNumType>> {
            pub predicate: fn(&E::Big, &E::Big) -> bool,
            pub cbigint_op1: fn(L, R) -> GenericBigNum<'static, E>,
            pub cbigint_op2: fn(L, &R) -> GenericBigNum<'static, E>,
            pub cbigint_op3: fn(&L, R) -> GenericBigNum<'static, E>,
            pub cbigint_op4: fn(&L, &R) -> GenericBigNum<'static, E>,
            pub op_assign1: fn(&mut GenericBigNum<'static, E>, R),
            pub op_assign2: Option<fn(&mut GenericBigNum<'static, E>, &R)>,
            pub bigint_op: fn(&E::Big, &E::Big) -> E::Big,
        }

        pub fn test_shift_op<R, E: Encoding<'static, Big = BigNumType>>(
            ops: ShiftOpsForType<R, E>,
            lhs: GenericBigNum<'static, E>,
            rhs: R,
        ) -> TestResult
        where
            R: Copy + Ord + Zero + Display,
        {
            let big_lhs = &E::Big::from(lhs.clone());

            assert!(rhs >= R::zero(), "shift amount must be non-negative");
            let expected = (ops.bigint_op)(big_lhs, rhs);
            let actual1 = (ops.cbigint_op1)(lhs.clone(), rhs).into();
            assert_eq!(expected, actual1, "failed with inputs {}, {}", big_lhs, rhs);
            let actual2 = (ops.cbigint_op2)(lhs.clone(), &rhs).into();
            assert_eq!(expected, actual2, "failed with inputs {}, {}", big_lhs, rhs);
            let actual3 = (ops.cbigint_op3)(&lhs, rhs).into();
            assert_eq!(expected, actual3, "failed with inputs {}, {}", big_lhs, rhs,);
            let actual4 = (ops.cbigint_op4)(&lhs, &rhs).into();
            assert_eq!(expected, actual4, "failed with inputs {}, {}", big_lhs, rhs);
            let mut actual5 = big_lhs.clone().into();
            (ops.op_assign1)(&mut actual5, rhs);
            assert_eq!(
                expected,
                actual5.clone().into(),
                "failed with inputs {}, {}",
                big_lhs,
                rhs
            );
            TestResult::passed()
        }

        pub fn test_bin_op<L, R, E: Encoding<'static, Big = BigNumType>>(
            ops: BinOpsForTypes<L, R, E>,
            lhs: L,
            rhs: R,
        ) -> TestResult
        where
            L: Clone,
            R: Clone,
            E::Big: From<L>,
            E::Big: From<R>,
        {
            let big_lhs = &E::Big::from(lhs.clone());
            let big_rhs = &E::Big::from(rhs.clone());

            if (ops.predicate)(big_lhs, big_rhs) {
                let expected = (ops.bigint_op)(big_lhs, big_rhs);
                let actual1 = (ops.cbigint_op1)(lhs.clone(), rhs.clone()).into();
                assert_eq!(
                    expected, actual1,
                    "failed with inputs {}, {}",
                    big_lhs, big_rhs
                );
                let actual2 = (ops.cbigint_op2)(lhs.clone(), &rhs).into();
                assert_eq!(
                    expected, actual2,
                    "failed with inputs {}, {}",
                    big_lhs, big_rhs
                );
                let actual3 = (ops.cbigint_op3)(&lhs, rhs.clone()).into();
                assert_eq!(
                    expected, actual3,
                    "failed with inputs {}, {}",
                    big_lhs, big_rhs
                );
                let actual4 = (ops.cbigint_op4)(&lhs, &rhs).into();
                assert_eq!(
                    expected, actual4,
                    "failed with inputs {}, {}",
                    big_lhs, big_rhs
                );
                let mut actual5 = big_lhs.clone().into();
                (ops.op_assign1)(&mut actual5, rhs.clone());
                assert_eq!(
                    expected,
                    actual5.clone().into(),
                    "failed with inputs {}, {}",
                    big_lhs,
                    big_rhs
                );
                if let Some(op_assign) = ops.op_assign2 {
                    let mut actual6 = big_lhs.clone().into();
                    op_assign(&mut actual6, &rhs);
                    assert_eq!(
                        expected,
                        actual6.clone().into(),
                        "failed with inputs {}, {}",
                        big_lhs,
                        big_rhs
                    );
                }
                TestResult::passed()
            } else {
                TestResult::discard()
            }
        }
    } }

    duplicate_generic_bignum_types! { mod bignum_tag {
        use super::*;
        use signedness::*;

        duplicate_arith_and_bit_ops! {
            paste! {
                #[quickcheck]
                fn [<test_ op_fn>](lhs: bignum_type, rhs: bignum_type) -> TestResult {
                    test_bin_op(BinOpsForTypes {
                        predicate: op_test_pred,
                        cbigint_op1: |x, y| op_trait::op_fn(x, y),
                        cbigint_op2: |x, y| op_trait::op_fn(x, y),
                        cbigint_op3: |x, y| op_trait::op_fn(x, y),
                        cbigint_op4: |x, y| op_trait::op_fn(x, y),
                        op_assign1: |x, y| [<op_trait Assign>]::[<op_fn _assign>](x, y),
                        op_assign2: Some(|x, y| [<op_trait Assign>]::[<op_fn _assign>](x, y)),
                        bigint_op: |x, y| op_trait::op_fn(x, y),
                    }, lhs, rhs)
                }
            }
        }

        duplicate_shift_ops! {
            duplicate_prims! {
                paste! {
                    #[quickcheck]
                    fn [<test_ op_fn _ prim _rhs>](lhs: bignum_type, rhs: u16) -> TestResult{
                                #[allow(irrefutable_let_patterns)]
                        if let Ok(rhs) = prim::try_from(rhs) {
                            test_shift_op(ShiftOpsForType {
                                cbigint_op1: |x, y| op_trait::op_fn(x, y),
                                cbigint_op2: |x, y| op_trait::op_fn(x, y),
                                cbigint_op3: |x, y| op_trait::op_fn(x, y),
                                cbigint_op4: |x, y| op_trait::op_fn(x, y),
                                op_assign1: |x, y| [<op_trait Assign>]::[<op_fn _assign>](x, y),
                                bigint_op: |x, y| op_trait::op_fn(x, y),
                            }, lhs, rhs)
                        } else {
                            TestResult::discard()
                        }
                    }
                }
            }
        }

        duplicate_arith_ops! {
            duplicate_prims_with_signedness! { signedness;
                paste! {
                    #[quickcheck]
                    fn [<test_ op_fn _ prim _lhs>](lhs: prim, rhs: bignum_type) -> TestResult{
                        test_bin_op(BinOpsForTypes {
                            predicate: op_test_pred,
                            cbigint_op1: |x: prim, y: bignum_type| op_trait::op_fn(x, y),
                            cbigint_op2: |x, y| op_trait::op_fn(x, y),
                            cbigint_op3: |x, y| op_trait::op_fn(x, y),
                            cbigint_op4: |x, y| op_trait::op_fn(x, y),
                            op_assign1: |x, y| [<op_trait Assign>]::[<op_fn _assign>](x, y),
                            op_assign2: Some(|x, y| [<op_trait Assign>]::[<op_fn _assign>](x, y)),
                            bigint_op: |x, y| op_trait::op_fn(x, y),
                        }, lhs, rhs)
                    }
                    #[quickcheck]
                    fn [<test_ op_fn _ prim _rhs>](lhs: bignum_type, rhs: prim) -> TestResult {
                        test_bin_op(BinOpsForTypes {
                            predicate: op_test_pred,
                            cbigint_op1: |x: bignum_type, y: prim| op_trait::op_fn(x, y),
                            cbigint_op2: |x, y| op_trait::op_fn(x, y),
                            cbigint_op3: |x, y| op_trait::op_fn(x, y),
                            cbigint_op4: |x, y| op_trait::op_fn(x, y),
                            op_assign1: |x, y| [<op_trait Assign>]::[<op_fn _assign>](x, y),
                            op_assign2: None,
                            bigint_op: |x, y| op_trait::op_fn(x, y),
                        }, lhs, rhs)
                    }
                }
            }
        }

        // TODO
        // duplicate_uprims! {
        //     paste! {
        //         #[quickcheck]
        //         fn [<test_pow_ prim>](lhs: bignum_type, rhs: u8) -> TestResult {
        //             let rhs = rhs % 64; // limit the exponent to avoid long test times and potential OOM errors
        //             #[allow(irrefutable_let_patterns)]
        //             #[allow(clippy::unnecessary_fallible_conversions)]
        //             if let Ok(rhs) = prim::try_from(rhs) {
        //                 let big_lhs = &bignum_type::from(lhs.clone());
        //                 let expected = Pow::pow(big_lhs, rhs);
        //                 let actual1 = bignum_type::from(Pow::pow(lhs.clone(), rhs));
        //                 let actual2 = bignum_type::from(Pow::pow(lhs, rhs));
        //                 let label = format!("failed with inputs {}, {}", big_lhs, rhs);
        //                 assert_eq!(expected, actual1, "{}", label);
        //                 assert_eq!(expected, actual2, "{}", label);
        //                 TestResult::passed()
        //             } else {
        //                 TestResult::discard()
        //             }
        //         }
        //     }
        // }
    } }
}
