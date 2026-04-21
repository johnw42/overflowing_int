#![allow(unused_imports)]
use crate::big_number::{BigNumber, BigSigned};
use crate::encoding::{Decode, Decoded, Encoding, EncodingMut};
use crate::signed::Int;
use crate::small_num::SmallNumber as _;
use crate::unsigned::Uint;
use crate::{
    duplicate_arith_ops, duplicate_bit_ops, duplicate_generic_bignum, duplicate_iprims,
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
trait ArithOp<'enc, E>
where
    E: Encoding<'enc>,
{
    fn on_big_small(lhs: Cow<E::Big>, rhs: E::Small) -> E::Big;
    fn on_small(lhs: E::Small, rhs: E::Small) -> Option<E::Small>;
    fn on_small_big(lhs: E::Small, rhs: Cow<E::Big>) -> E::Big;
    fn on_big(lhs: Cow<E::Big>, rhs: Cow<E::Big>) -> E::Big;
    fn update_big(lhs: &mut E::Big, rhs: Cow<E::Big>);
    fn update_small(lhs: &mut E::Big, rhs: E::Small);

    /// Calls a version of the binary operator that returns a new number.
    fn call<'lhs, 'rhs, L, R>(lhs: L, rhs: R) -> E
    where
        L: Decode<'lhs, E::Small>,
        R: Decode<'rhs, E::Small>,
    {
        match (lhs.decode(), rhs.decode()) {
            (Decoded::Small(lhs), Decoded::Small(rhs)) => match Self::on_small(lhs, rhs) {
                Some(out) => E::from_small(out),
                None => E::from_big(Self::on_big_small(Cow::Owned(lhs.to_big()), rhs)),
            },
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
    fn call_update<'rhs, R>(lhs: &mut E, rhs: R)
    where
        E: EncodingMut<'enc>,
        R: Decode<'rhs, E::Small>,
    {
        lhs.update_encoding(|encoding| match encoding {
            Decoded::Small(small_lhs) => match rhs.decode() {
                Decoded::Small(small_rhs) => match Self::on_small(*small_lhs, small_rhs) {
                    Some(out) => *encoding = Decoded::Small(out),
                    None => {
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

trait BitOp<'enc, E>
where
    E: Encoding<'enc>,
{
    fn on_big<'lhs, 'rhs>(lhs: Cow<'lhs, E::Big>, rhs: Cow<'rhs, E::Big>) -> E::Big;
    fn update_big(lhs: &mut E::Big, rhs: Cow<E::Big>);

    fn call<'lhs, 'rhs, L, R>(lhs: L, rhs: R) -> E
    where
        L: Decode<'lhs, E::Small>,
        R: Decode<'rhs, E::Small>,
    {
        E::from_big(Self::on_big(lhs.big_cow(), rhs.big_cow()))
    }

    fn call_update<'rhs, R>(lhs: &mut E, rhs: R)
    where
        E: EncodingMut<'enc>,
        R: Decode<'rhs, E::Small>,
    {
        lhs.update_encoding(|encoding| match encoding {
            Decoded::Small(small_lhs) => {
                *encoding = Decoded::Big(Cow::Owned(Self::on_big(
                    Cow::Owned(small_lhs.to_big()),
                    rhs.big_cow(),
                )));
            }
            Decoded::Big(big_lhs) => {
                Self::update_big(big_lhs.to_mut(), rhs.big_cow());
            }
        });
    }
}
trait ShiftOp<'enc, E>
where
    E: Encoding<'enc>,
{
    duplicate_prims! {
        paste! {
            fn [<on_big_ prim>](lhs: Cow<E::Big>, rhs: prim) -> E::Big;
            fn [<update_big_ prim>](lhs: &mut E::Big, rhs: prim);

            fn [<call_ prim>]<'a, L>(lhs: L, rhs: prim) -> E
            where
                L: Decode<'a, E::Small>,
            {
                E::from_big(Self::[<on_big_ prim>](lhs.big_cow(), rhs))
            }

            #[inline]
            fn [<call_update_big_ prim>](lhs: &mut E, rhs: prim)
            where
                E: EncodingMut<'enc>,
            {
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
trait PowOpTrait<'enc, E>
where
    E: Encoding<'enc>,
{
    fn on_big_small(lhs: Cow<E::Big>, rhs: <E::Unsigned as Encoding<'enc>>::Small) -> E::Big;
    fn on_small(lhs: E::Small, rhs: <E::Unsigned as Encoding<'enc>>::Small)
    -> Result<E::Small, ()>;
    fn on_big(lhs: Cow<E::Big>, rhs: Cow<<E::Unsigned as Encoding<'enc>>::Big>) -> E::Big;

    /// Calls a version of the binary operator that returns a new number.
    fn call<'lhs, 'rhs, L, R>(lhs: L, rhs: R) -> E
    where
        L: Decode<'lhs, E::Small>,
        R: Decode<'rhs, <E::Unsigned as Encoding<'enc>>::Small>,
    {
        match (lhs.decode(), rhs.decode()) {
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
        }
    }
}

// MARK: Meta-Operator Trait Implementations
// -----------------------------------------------------------------------------
duplicate_arith_ops! {
    paste! {
        struct [<OpTrait Op>];

        impl<'e, E> ArithOp<'e, E> for [<OpTrait Op>]
        where
            E: Encoding<'e>,
        {

            fn on_big(lhs: Cow<E::Big>, rhs: Cow<E::Big>) -> E::Big {
                match (lhs, rhs) {
                    (Cow::Borrowed(lhs), Cow::Borrowed(rhs)) => E::Big::[<op_fn _ref_self_and_ref_self>](lhs, rhs),
                    (Cow::Borrowed(lhs), Cow::Owned(rhs)) => E::Big::[<op_fn _ref_self_and_self>](lhs, rhs),
                    (Cow::Owned(lhs), Cow::Borrowed(rhs)) => E::Big::[<op_fn _self_and_ref_self>](lhs, rhs),
                    (Cow::Owned(lhs), Cow::Owned(rhs)) => E::Big::[<op_fn _self_and_self>](lhs, rhs),
                }
            }

            fn on_small(lhs: E::Small, rhs: E::Small) -> Option<E::Small> {
                lhs.[<checked_ op_fn>](&rhs)
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
        struct [<OpTrait Op>];

        impl<'e, E> BitOp<'e, E> for [<OpTrait Op>]
        where
            E: Encoding<'e>,
        {

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
    paste! { struct [<OpTrait Op>]; }

    impl<'e, E> ShiftOp<'e, E> for paste! { [<OpTrait Op>] }
    where
        E: Encoding<'e>,
    {
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

impl<'e, E> PowOpTrait<'e, E> for PowOp
where
    E: Encoding<'e>,
{
    fn on_big(lhs: Cow<E::Big>, rhs: Cow<<E::Unsigned as Encoding<'e>>::Big>) -> E::Big {
        big_pow(lhs, rhs.as_ref())
    }

    fn on_small(lhs: E::Small, rhs: <E::Unsigned as Encoding<'e>>::Small) -> Result<E::Small, ()> {
        if let Some(rhs) = rhs.to_u32()
            && let Some(result) = lhs.try_pow(rhs)
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
duplicate_generic_bignum! {

duplicate_arith_ops! {
    paste! {
        impl<'enc, 'rhs, T, E> OpTrait<T> for EncodedType<E>
        where
            E: Encoding<'enc, Big = ImplType>,
            T: Decode<'rhs, E::Small>,
        {
            type Output = EncodedType<E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                EncodedType([<OpTrait Op>]::call(self, rhs))
            }
        }

        impl<'enc, 'lhs, 'rhs, T, E> OpTrait<T> for &'lhs EncodedType<E>
        where
            E: Encoding<'enc, Big = ImplType>,
            T: Decode<'rhs, E::Small>,
        {
            type Output = EncodedType<E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                EncodedType([<OpTrait Op>]::call(self, rhs))
            }
        }

        impl<'enc, 'rhs, T, E> [<OpTrait  Assign>]<T> for EncodedType<E>
        where
            E: EncodingMut<'enc, Big = ImplType>,
            T: Decode<'rhs, E::Small>,
        {
            fn [<op_fn _assign>](&mut self, rhs: T) {
                [<OpTrait Op>]::call_update(&mut self.0, rhs);
            }
        }
    }

    duplicate_iprims! {
        paste! {
            impl<'enc, E> OpTrait<EncodedType<E>> for prim
            where
                E: Encoding<'enc, Big = ImplType>,
                E::Big: BigSigned,
            {
                type Output = EncodedType<E>;

                #[inline(never)]
                fn op_fn(self, rhs: EncodedType<E>) -> Self::Output {
                    EncodedType([<OpTrait Op>]::call(self, rhs))
                }
            }

            impl<'enc, 'lhs, E> OpTrait<EncodedType<E>> for &'lhs prim
            where
                E: Encoding<'enc, Big = ImplType>,
                E::Big: BigSigned,
            {
                type Output = EncodedType<E>;

                fn op_fn(self, rhs: EncodedType<E>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }

            impl<'enc, 'rhs, E> OpTrait<&'rhs EncodedType<E>> for prim
            where
                E: Encoding<'enc, Big = ImplType>,
                E::Big: BigSigned,
            {
                type Output = EncodedType<E>;

                #[inline(never)]
                fn op_fn(self, rhs: &'rhs EncodedType<E>) -> Self::Output {
                    EncodedType([<OpTrait Op>]::call(self, rhs))
                }
            }

            impl<'enc, 'rhs, E> OpTrait<&'rhs EncodedType<E>> for &prim
            where
                E: Encoding<'enc, Big = ImplType>,
                E::Big: BigSigned,
            {
                type Output = EncodedType<E>;

                fn op_fn(self, rhs: &'rhs EncodedType<E>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }
        }
    }

    duplicate_uprims! {
        paste! {
            impl<'enc, E> OpTrait<EncodedType<E>> for prim
            where
                E: Encoding<'enc, Big = ImplType>,
            {
                type Output = EncodedType<E>;

                fn op_fn(self, rhs: EncodedType<E>) -> Self::Output {
                    EncodedType([<OpTrait Op>]::call(self, rhs))
                }
            }

            impl<'enc, E> OpTrait<EncodedType<E>> for &prim
            where
                E: Encoding<'enc, Big = ImplType>,
            {
                type Output = EncodedType<E>;

                fn op_fn(self, rhs: EncodedType<E>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }

            impl<'enc, 'rhs, E> OpTrait<&'rhs EncodedType<E>> for prim
            where
                E: Encoding<'enc, Big = ImplType>,
            {
                type Output = EncodedType<E>;

                fn op_fn(self, rhs: &'rhs EncodedType<E>) -> Self::Output {
                    EncodedType([<OpTrait Op>]::call(self, rhs))
                }
            }

            impl<'enc, 'rhs, E> OpTrait<&'rhs EncodedType<E>> for &prim
            where
                E: Encoding<'enc, Big = ImplType>,
            {
                type Output = EncodedType<E>;

                fn op_fn(self, rhs: &'rhs EncodedType<E>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }
        }
    }
}

duplicate_bit_ops! {
    paste! {
        impl<'enc, 'rhs, R, E> OpTrait<R> for EncodedType<E>
        where
            E: Encoding<'enc, Big = ImplType>,
            R: Decode<'rhs, E::Small>,
        {
            type Output = EncodedType<E>;

            fn op_fn(self, rhs: R) -> Self::Output {
                EncodedType([<OpTrait Op>]::call(self, rhs))
            }
        }

        impl<'enc, 'lhs, 'rhs, R, E> OpTrait<R> for &'lhs EncodedType<E>
        where
            E: Encoding<'enc, Big = ImplType>,
            R: Decode<'rhs, E::Small>,
        {
            type Output = EncodedType<E>;

            fn op_fn(self, rhs: R) -> Self::Output {
                EncodedType([<OpTrait Op>]::call(self, rhs))
            }
        }

        impl<'enc, 'rhs, R, E> [<OpTrait  Assign>]<R> for EncodedType<E>
        where
            E: EncodingMut<'enc, Big = ImplType>,
            R: Decode<'rhs, E::Small>
        {
            fn [<op_fn _assign>](&mut self, rhs: R) {
                [<OpTrait Op>]::call_update(&mut self.0, rhs);
            }
        }
    }
}

duplicate_shift_ops! {
    duplicate_prims! {
        paste! {
            impl<'enc, E> OpTrait<prim> for EncodedType<E>
            where
                E: Encoding<'enc, Big = ImplType>,
            {
                type Output = EncodedType<E>;

                fn op_fn(self, rhs: prim) -> Self::Output {
                    EncodedType([<OpTrait Op>]::[<call_ prim>](self.0, rhs))
                }
            }

            impl<'enc, E> OpTrait<&prim> for EncodedType<E>
            where
                E: Encoding<'enc, Big = ImplType>,
            {
                type Output = EncodedType<E>;

                fn op_fn(self, rhs: &prim) -> Self::Output {
                    self.op_fn(*rhs)
                }
            }

            impl<'enc, 'lhs, E> OpTrait<prim> for &'lhs EncodedType<E>
            where
                E: Encoding<'enc, Big = ImplType>,
            {
                type Output = EncodedType<E>;

                fn op_fn(self, rhs: prim) -> Self::Output {
                    EncodedType([<OpTrait Op>]::[<call_ prim>](self, rhs))
                }
            }

            impl<'enc, 'lhs, E> OpTrait<&prim> for &'lhs EncodedType<E>
            where
                E: Encoding<'enc, Big = ImplType>,
            {
                type Output = EncodedType<E>;

                fn op_fn(self, rhs: &prim) -> Self::Output {
                    self.op_fn(*rhs)
                }
            }

            impl<'enc, E> [<OpTrait  Assign>]<prim> for EncodedType<E>
            where
                E: EncodingMut<'enc, Big = ImplType>,
            {
                fn [<op_fn _assign>](&mut self, rhs: prim) {
                    [<OpTrait Op>]::[<call_update_big_ prim>](&mut self.0, rhs);
                }
            }

            impl<'enc, E> [<OpTrait  Assign>]<&prim> for EncodedType<E>
            where
                E: EncodingMut<'enc, Big = ImplType>,
            {
                fn [<op_fn _assign>](&mut self, rhs: &prim) {
                    self.[<op_fn _assign>](*rhs);
                }
            }
        }
    }
}

} // duplicate_generic_bignum!

// MARK: Pow Operator Implementations
// -----------------------------------------------------------------------------
duplicate_generic_bignum! {

impl<'enc, E> Pow<Uint<E::Unsigned>> for EncodedType<E>
where
    E: Encoding<'enc, Big = ImplType>,
{
    type Output = EncodedType<E>;

    fn pow(self, rhs: Uint<E::Unsigned>) -> Self::Output {
        EncodedType(PowOp::call(self, rhs))
    }
}

impl<'enc, 'rhs, E> Pow<&'rhs Uint<E::Unsigned>> for EncodedType<E>
where
    E: Encoding<'enc, Big = ImplType>,
{
    type Output = EncodedType<E>;

    fn pow(self, rhs: &'rhs Uint<E::Unsigned>) -> Self::Output {
        EncodedType(PowOp::call(self, rhs))
    }
}

impl<'enc, 'lhs, E> Pow<Uint<E::Unsigned>> for &'lhs EncodedType<E>
where
    E: Encoding<'enc, Big = ImplType>,
{
    type Output = EncodedType<E>;

    fn pow(self, rhs: Uint<E::Unsigned>) -> Self::Output {
        EncodedType(PowOp::call(self, rhs))
    }
}

impl<'enc,'lhs, 'rhs, E> Pow<&'rhs Uint<E::Unsigned>> for &'lhs EncodedType<E>
where
    E: Encoding<'enc, Big = ImplType>,
{
    type Output = EncodedType<E>;

    fn pow(self, rhs: &'rhs Uint<E::Unsigned>) -> Self::Output {
        EncodedType(PowOp::call(self, rhs))
    }
}

duplicate_uprims! {
    paste! {
        impl<'enc, E> Pow<prim> for EncodedType<E>
        where
            E: Encoding<'enc, Big = ImplType>,
        {
            type Output = EncodedType<E>;

            fn pow(self, rhs: prim) -> Self::Output {
                EncodedType(PowOp::call(self, rhs))
            }
        }

        impl<'enc, E> Pow<&prim> for EncodedType<E>
        where
            E: Encoding<'enc, Big = ImplType>,
        {
            type Output = EncodedType<E>;

            fn pow(self, rhs: &prim) -> Self::Output {
                EncodedType(PowOp::call(self, rhs))
            }
        }

        impl<'enc, E> Pow<prim> for &EncodedType<E>
        where
            E: Encoding<'enc, Big = ImplType>,
        {
            type Output = EncodedType<E>;

            fn pow(self, rhs: prim) -> Self::Output {
                EncodedType(PowOp::call(self, rhs))
            }
        }

        impl<'enc, E> Pow<&prim> for &EncodedType<E>
        where
            E: Encoding<'enc, Big = ImplType>,
        {
            type Output = EncodedType<E>;

            fn pow(self, rhs: &prim) -> Self::Output {
                EncodedType(PowOp::call(self, rhs))
            }
        }
    }
}

} // duplicate_generic_bignum!

// MARK: Tests
#[cfg(test)]
mod test {
    use std::fmt::Display;

    use super::*;
    use crate::duplicate_arith_and_bit_ops;
    use crate::duplicate_encoded_types;
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

    duplicate_encoded_types! { mod encoding_tag {
        use super::*;
        use std::borrow::Borrow as _;

        macro_rules! test_bin_op {
            ($expected: ident, $lhs: ident, $rhs: ident, $OpTrait: ident, $op_fn: ident, $op_test_pred: ident) => {
                paste! {
                    let big_lhs = &ImplType::from($lhs.clone());
                    let big_rhs = &ImplType::from($rhs.clone());

                    if !$op_test_pred(big_lhs, big_rhs) {
                        return TestResult::discard();
                    }
                    let $expected = $OpTrait::$op_fn(big_lhs, big_rhs);
                    let actual = $OpTrait::$op_fn($lhs.clone(), $rhs.clone()).into();
                    assert_eq!($expected, actual);
                    let actual = $OpTrait::$op_fn($lhs.clone(), &$rhs).into();
                    assert_eq!($expected, actual);
                    let actual = $OpTrait::$op_fn(&$lhs, $rhs.clone()).into();
                    assert_eq!($expected, actual);
                    let actual = $OpTrait::$op_fn(&$lhs, &$rhs).into();
                    assert_eq!($expected, actual);
                    let actual = $OpTrait::$op_fn($lhs.clone(), $rhs.borrow()).into();
                    assert_eq!($expected, actual);
                    let actual = $OpTrait::$op_fn($lhs.borrow(), $rhs.clone()).into();
                    assert_eq!($expected, actual);
                    let actual = $OpTrait::$op_fn($lhs.borrow(), $rhs.borrow()).into();
                    assert_eq!($expected, actual);
                }
            };
        }

        macro_rules! test_bin_op_with_assign {
            ($expected: ident, $lhs: ident, $rhs: ident, $OpTrait: ident, $op_fn: ident, $op_test_pred: ident) => {
                paste! {
                    test_bin_op!($expected, $lhs, $rhs, $OpTrait, $op_fn, $op_test_pred);
                    let mut actual = $lhs.clone();
                    [<$OpTrait  Assign>]::[<$op_fn _assign>](&mut actual, $rhs.clone());
                    assert_eq!($expected, actual.into());
                }
            };
        }

        macro_rules! test_bin_op_with_ref_assign {
            ($expected: ident, $lhs: ident, $rhs: ident, $OpTrait: ident, $op_fn: ident, $op_test_pred: ident) => {
                paste! {
                    test_bin_op_with_assign!($expected, $lhs, $rhs, $OpTrait, $op_fn, $op_test_pred);
                    let mut actual = $lhs.clone();
                    [<$OpTrait  Assign>]::[<$op_fn _assign>](&mut actual, &$rhs);
                    assert_eq!($expected, actual.into());
                    let mut actual = $lhs.clone();
                    [<$OpTrait  Assign>]::[<$op_fn _assign>](&mut actual, $rhs.borrow());
                    assert_eq!($expected, actual.into());
                }
            };
        }

        duplicate_arith_ops! {
            paste! {
                #[quickcheck]
                fn [<test_ op_fn>](lhs: EncodedType, rhs: EncodedType) -> TestResult {
                    test_bin_op_with_ref_assign!(expected,lhs, rhs, OpTrait, op_fn, op_test_pred);
                    TestResult::passed()
                }
            }
        }
        duplicate_bit_ops! {
            paste! {
                #[quickcheck]
                fn [<test_ op_fn>](lhs: EncodedType, rhs: EncodedType) -> TestResult {
                    test_bin_op_with_assign!(expected, lhs, rhs, OpTrait, op_fn, op_test_pred);
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
                            let big_lhs = &ImplType::from(lhs.clone());

                            #[allow(unused_comparisons)]
                            let nonnegative_rhs = rhs >= 0;
                            assert!(nonnegative_rhs, "shift amount must be non-negative");

                            let expected = OpTrait::op_fn(big_lhs, rhs);
                            let actual = OpTrait::op_fn(lhs.clone(), rhs).into();
                            assert_eq!(expected, actual);
                            let actual = OpTrait::op_fn(lhs.clone(), &rhs).into();
                            assert_eq!(expected, actual);
                            let actual = OpTrait::op_fn(&lhs, rhs).into();
                            assert_eq!(expected, actual,);
                            let actual = OpTrait::op_fn(&lhs, &rhs).into();
                            assert_eq!(expected, actual);
                            let mut actual = lhs.clone();
                            [<OpTrait  Assign>]::[<op_fn _assign>](&mut actual, rhs);
                            assert_eq!(expected, actual.clone().into());
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
                        test_bin_op!(expected, lhs, rhs, OpTrait, op_fn, op_test_pred);
                        TestResult::passed()
                    }
                    #[quickcheck]
                    fn [<test_ op_fn _ prim _rhs>](lhs: EncodedType, rhs: prim) -> TestResult {
                        test_bin_op_with_assign!(expected, lhs, rhs, OpTrait, op_fn, op_test_pred);
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
                        let rhs = rhs % 16; // limit the exponent to avoid long test times and potential OOM errors
                        #[allow(irrefutable_let_patterns)]
                        #[allow(clippy::unnecessary_fallible_conversions)]
                        if let Ok(rhs) = prim::try_from(rhs) {
                            let big_lhs = &EncodedType::from(lhs.clone());
                            let expected = Pow::pow(big_lhs, rhs);
                            let actual = EncodedType::from(Pow::pow(lhs.clone(), rhs));
                            assert_eq!(expected, actual);
                            let actual = EncodedType::from(Pow::pow(lhs, rhs));
                            assert_eq!(expected, actual);
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
