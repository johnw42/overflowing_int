use crate::big_number::BigNumber;
use crate::generic_bignum::encoding::{Decoded, EncodedBigNum, InspectEncoding};
use crate::generic_bignum::struct_def::GenericBigNum;
use crate::small_num::SmallNumber as _;
use crate::{
    duplicate_arith_ops, duplicate_bit_ops, duplicate_prims, duplicate_shift_ops, duplicate_uprims,
};
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedSub, Pow};
use paste::paste;
use std::borrow::Cow;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

// MARK: Meta-Operator Trait Definitions
// -----------------------------------------------------------------------------
trait ArithOp<'e, E: EncodedBigNum<'e>> {
    fn on_big_small(lhs: Cow<E::Big>, rhs: E::Small) -> E::Big;
    fn on_small(lhs: E::Small, rhs: E::Small) -> Result<E::Small, ()>;
    fn on_small_big(lhs: E::Small, rhs: Cow<E::Big>) -> E::Big;
    fn on_big(lhs: Cow<E::Big>, rhs: Cow<E::Big>) -> E::Big;
    fn update_big(lhs: &mut E::Big, rhs: Cow<E::Big>);
    fn update_small(lhs: &mut E::Big, rhs: E::Small);

    /// Calls a version of the binary operator that returns a new number.
    #[inline]
    fn call<'a, 'b, L, R>(lhs: L, rhs: R) -> GenericBigNum<'e, E>
    where
        L: InspectEncoding<'a, E::Small, E::Big>,
        R: InspectEncoding<'b, E::Small, E::Big>,
    {
        match (lhs.decode(), rhs.decode()) {
            (Decoded::Small(lhs), Decoded::Small(rhs)) => {
                if let Ok(out) = Self::on_small(lhs, rhs) {
                    GenericBigNum::from_small(out)
                } else {
                    GenericBigNum::from_big(Self::on_big_small(Cow::Owned(lhs.to_big()), rhs))
                }
            }
            (Decoded::Small(small_lhs), Decoded::Big(big_rhs)) => {
                GenericBigNum::from_big(Self::on_small_big(small_lhs, big_rhs))
            }
            (Decoded::Big(big_lhs), Decoded::Small(small_rhs)) => {
                GenericBigNum::from_big(Self::on_big_small(big_lhs, small_rhs))
            }
            (Decoded::Big(big_lhs), Decoded::Big(big_rhs)) => {
                GenericBigNum::from_big(Self::on_big(big_lhs, big_rhs))
            }
        }
    }

    /// Calls a version of the binary operator that updates a bigint argument in place.
    #[inline]
    fn call_update<'a, 'c, R>(lhs: &'a mut GenericBigNum<'e, E>, rhs: R)
    where
        R: InspectEncoding<'c, E::Small, E::Big>,
    {
        lhs.update_encoding(|encoding| match encoding {
            Decoded::Small(small_lhs) => match rhs.decode() {
                Decoded::Small(small_rhs) => match Self::on_small(*small_lhs, small_rhs) {
                    Ok(out) => *encoding = Decoded::Small(out),
                    Err(()) => {
                        *encoding = Decoded::Big(Self::on_small_big(
                            *small_lhs,
                            Cow::Owned(small_rhs.to_big()),
                        ));
                    }
                },
                Decoded::Big(big_rhs) => {
                    *encoding = Decoded::Big(Self::on_small_big(*small_lhs, big_rhs));
                }
            },
            Decoded::Big(big_lhs) => match rhs.decode() {
                Decoded::Small(small_rhs) => {
                    Self::update_small(big_lhs, small_rhs);
                }
                Decoded::Big(big_rhs) => {
                    Self::update_big(big_lhs, big_rhs);
                }
            },
        });
    }
}

trait BitOp<'e, E: EncodedBigNum<'e>> {
    fn on_big(lhs: Cow<E::Big>, rhs: Cow<E::Big>) -> E::Big;
    fn update_big(lhs: &mut E::Big, rhs: Cow<E::Big>);

    #[inline]
    fn call<'a, 'b, L, R>(lhs: L, rhs: R) -> GenericBigNum<'e, E>
    where
        L: InspectEncoding<'a, E::Small, E::Big>,
        R: InspectEncoding<'b, E::Small, E::Big>,
    {
        GenericBigNum::from_big(Self::on_big(lhs.into_big_cow(), rhs.into_big_cow()))
    }

    #[inline]
    fn call_update<'a, 'c, R>(lhs: &'a mut GenericBigNum<'e, E>, rhs: R)
    where
        R: InspectEncoding<'c, E::Small, E::Big>,
    {
        lhs.update_encoding(|encoding| match encoding {
            Decoded::Small(small_lhs) => {
                *encoding = Decoded::Big(Self::on_big(
                    Cow::Owned(small_lhs.to_big()),
                    rhs.into_big_cow(),
                ));
            }
            Decoded::Big(big_lhs) => {
                Self::update_big(big_lhs, rhs.into_big_cow());
            }
        });
    }
}
trait ShiftOp<'e, E: EncodedBigNum<'e>> {
    duplicate_prims! {
        paste! {
            fn [<on_big_ prim>](lhs: Cow<E::Big>, rhs: prim) -> E::Big;
            fn [<update_big_ prim>](lhs: &mut E::Big, rhs: prim);

            fn [<call_ prim>]<'a, L>(lhs: L, rhs: prim) -> GenericBigNum<'e, E>
            where
                L: InspectEncoding<'a, E::Small, E::Big>,
            {
                GenericBigNum::from_big(Self::[<on_big_ prim>](lhs.into_big_cow(), rhs))
            }

            #[inline]
            fn [<call_update_big_ prim>](lhs: &mut GenericBigNum<'e, E>, rhs: prim) {
                lhs.update_encoding(|encoding| match encoding {
                    Decoded::Small(small_lhs) => {
                        *encoding = Decoded::Big(Self::[<on_big_ prim>](
                            Cow::Owned(small_lhs.to_big()),
                            rhs,
                        ));
                    }
                    Decoded::Big(big_lhs) => {
                        Self::[<update_big_ prim>](big_lhs, rhs);
                    }
                });
            }
        }
    }
}

// MARK: Meta-Operator Trait Implementations
// -----------------------------------------------------------------------------
duplicate_arith_ops! {
    paste! {
        struct [<op_trait Op>];

        impl<'e, E: EncodedBigNum<'e>> ArithOp<'e, E> for [<op_trait Op>] {

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

        impl<'e, E: EncodedBigNum<'e>> BitOp<'e, E> for [<op_trait Op>] {

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

    impl<'e, E: EncodedBigNum<'e>> ShiftOp<'e, E> for paste! { [<op_trait Op>] } {
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

// MARK: Operator Trait Implementations
// -----------------------------------------------------------------------------
duplicate_arith_ops! {
    paste! {
        impl<'a, T, E: EncodedBigNum<'a>> op_trait<T> for GenericBigNum<'a, E>
        where
            T: InspectEncoding<'a, E::Small, E::Big>,
        {
            type Output = GenericBigNum<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, T, E: EncodedBigNum<'a>> op_trait<T> for &GenericBigNum<'a, E>
        where
            T: InspectEncoding<'a, E::Small, E::Big>,
        {
            type Output = GenericBigNum<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, T, E: EncodedBigNum<'a>> [<op_trait Assign>]<T> for GenericBigNum<'a, E>
        where
            T: InspectEncoding<'a, E::Small, E::Big>
        {
            fn [<op_fn _assign>](&mut self, rhs: T) {
                [<op_trait Op>]::call_update(self, rhs);
            }
        }
    }

    crate::duplicate_uprims! {
        paste! {
            impl<'a, E: EncodedBigNum<'a>> op_trait<GenericBigNum<'a, E>> for prim {
                type Output = GenericBigNum<'a, E>;

                #[inline(never)]
                fn op_fn(self, rhs: GenericBigNum<'a, E>) -> Self::Output {
                    [<op_trait Op>]::call(self, rhs)
                }
            }

            impl<'a, E: EncodedBigNum<'a>> op_trait<GenericBigNum<'a, E>> for &prim {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: GenericBigNum<'a, E>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }

            impl<'a, E: EncodedBigNum<'a>> op_trait<&GenericBigNum<'a, E>> for prim {
                type Output = GenericBigNum<'a, E>;

                #[inline(never)]
                fn op_fn(self, rhs: &GenericBigNum<'a, E>) -> Self::Output {
                    [<op_trait Op>]::call(self, rhs)
                }
            }

            impl<'a, E: EncodedBigNum<'a>> op_trait<&GenericBigNum<'a, E>> for &prim {
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
        impl<'a, 'b, T, E: EncodedBigNum<'a>> op_trait<T> for GenericBigNum<'a, E>
        where
            T: InspectEncoding<'b, E::Small, E::Big>,
        {
            type Output = GenericBigNum<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, 'b, T, E: EncodedBigNum<'a>> op_trait<T> for &GenericBigNum<'a, E>
        where
            T: InspectEncoding<'b, E::Small, E::Big>,
        {
            type Output = GenericBigNum<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, 'b, T, E: EncodedBigNum<'a>> [<op_trait Assign>]<T> for GenericBigNum<'a, E>
        where
            T: InspectEncoding<'b, E::Small, E::Big>
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
            impl<'a, E: EncodedBigNum<'a>> op_trait<prim> for GenericBigNum<'a, E> {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: prim) -> Self::Output {
                    [<op_trait Op>]::[<call_ prim>](self, rhs)
                }
            }

            impl<'a, E: EncodedBigNum<'a>> op_trait<&prim> for GenericBigNum<'a, E> {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: &prim) -> Self::Output {
                    self.op_fn(*rhs)
                }
            }

            impl<'a, E: EncodedBigNum<'a>> op_trait<prim> for &GenericBigNum<'a, E> {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: prim) -> Self::Output {
                    [<op_trait Op>]::[<call_ prim>](self, rhs)
                }
            }

            impl<'a, E: EncodedBigNum<'a>> op_trait<&prim> for &GenericBigNum<'a, E> {
                type Output = GenericBigNum<'a, E>;

                fn op_fn(self, rhs: &prim) -> Self::Output {
                    self.op_fn(*rhs)
                }
            }

            impl<'a, E: EncodedBigNum<'a>> [<op_trait Assign>]<prim> for GenericBigNum<'a, E> {
                fn [<op_fn _assign>](&mut self, rhs: prim) {
                    [<op_trait Op>]::[<call_update_big_ prim>](self, rhs);
                }
            }

            impl<'a, E: EncodedBigNum<'a>> [<op_trait Assign>]<&prim> for GenericBigNum<'a, E> {
                fn [<op_fn _assign>](&mut self, rhs: &prim) {
                    self.[<op_fn _assign>](*rhs);
                }
            }
        }
    }
}

// MARK: Pow Operator Implementations
// -----------------------------------------------------------------------------
duplicate_uprims! {
    paste! {
        impl<'a, E: EncodedBigNum<'a>> Pow<prim> for GenericBigNum<'a, E> {
            type Output = GenericBigNum<'a, E>;

            fn pow(self, rhs: prim) -> Self::Output {
                GenericBigNum::from_big(E::Big::[<pow_self_and_ref_ prim>](self.big_cow().into_owned(), &rhs))
            }
        }

        impl<'a, E: EncodedBigNum<'a>> Pow<&prim> for GenericBigNum<'a, E> {
            type Output = GenericBigNum<'a, E>;

            fn pow(self, rhs: &prim) -> Self::Output {
                GenericBigNum::from_big(E::Big::[<pow_self_and_ref_ prim>](self.big_cow().into_owned(), rhs))
            }
        }
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use std::borrow::Borrow;
    use std::fmt::Display;

    use super::*;
    use crate::duplicate_arith_and_bit_ops;
    use crate::duplicate_generic_bigint_types;
    use num_bigint::BigInt;
    use num_traits::{Pow, Zero};
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    fn always(_lhs: &impl BigNumber, _rhs: &impl BigNumber) -> bool {
        true
    }

    fn nonzero_rhs(_lhs: &impl BigNumber, rhs: &impl BigNumber) -> bool {
        !rhs.is_zero()
    }

    struct ShiftOpsForType<R, E: EncodedBigNum<'static>> {
        cbigint_op1: fn(GenericBigNum<'static, E>, R) -> GenericBigNum<'static, E>,
        cbigint_op2: fn(GenericBigNum<'static, E>, &R) -> GenericBigNum<'static, E>,
        cbigint_op3: fn(&GenericBigNum<'static, E>, R) -> GenericBigNum<'static, E>,
        cbigint_op4: fn(&GenericBigNum<'static, E>, &R) -> GenericBigNum<'static, E>,
        op_assign1: fn(&mut GenericBigNum<'static, E>, R),
        op_assign2: fn(&mut GenericBigNum<'static, E>, &R),
        bigint_op: fn(&E::Big, R) -> E::Big,
    }

    struct BinOpsForTypes<L, R, E: EncodedBigNum<'static>> {
        predicate: fn(&E::Big, &E::Big) -> bool,
        cbigint_op1: fn(L, R) -> GenericBigNum<'static, E>,
        cbigint_op2: fn(L, &R) -> GenericBigNum<'static, E>,
        cbigint_op3: fn(&L, R) -> GenericBigNum<'static, E>,
        cbigint_op4: fn(&L, &R) -> GenericBigNum<'static, E>,
        op_assign1: fn(&mut GenericBigNum<'static, E>, R),
        op_assign2: fn(&mut GenericBigNum<'static, E>, &R),
        bigint_op: fn(&E::Big, &E::Big) -> E::Big,
    }

    fn test_shift_op<R, E: EncodedBigNum<'static>>(
        ops: ShiftOpsForType<R, E>,
        lhs: GenericBigNum<'static, E>,
        rhs: u16,
    ) -> TestResult
    where
        R: TryFrom<u16> + Copy + Ord + Zero + Display,
    {
        let big_lhs = &lhs.clone().into_big();
        let lhs = GenericBigNum::from_big(big_lhs.clone());
        if let Ok(rhs) = R::try_from(rhs) {
            assert!(rhs >= R::zero(), "shift amount must be non-negative");
            let expected = (ops.bigint_op)(big_lhs, rhs);
            let actual1 = (ops.cbigint_op1)(lhs.clone(), rhs).into_big();
            let actual2 = (ops.cbigint_op2)(lhs.clone(), &rhs).into_big();
            let actual3 = (ops.cbigint_op3)(&lhs, rhs).into_big();
            let actual4 = (ops.cbigint_op4)(&lhs, &rhs).into_big();
            let mut actual5 = lhs.clone();
            (ops.op_assign1)(&mut actual5, rhs);
            let mut actual6 = lhs.clone();
            (ops.op_assign2)(&mut actual6, &rhs);
            let label = format!("failed with inputs {}, {}", big_lhs, rhs);
            assert_eq!(expected, actual1, "{}", label);
            assert_eq!(expected, actual2, "{}", label);
            assert_eq!(expected, actual3, "{}", label);
            assert_eq!(expected, actual4, "{}", label);
            assert_eq!(expected, actual5.into_big(), "{}", label);
            assert_eq!(expected, actual6.into_big(), "{}", label);
            TestResult::passed()
        } else {
            TestResult::discard()
        }
    }

    fn test_bin_op<L, R, E: EncodedBigNum<'static>>(
        ops: BinOpsForTypes<L, R, E>,
        lhs: L,
        rhs: R,
    ) -> TestResult
    where
        L: TryFrom<E::Big> + Clone,
        R: TryFrom<E::Big> + Clone,
        E::Big: From<L>,
        E::Big: From<R>,
    {
        let big_lhs = &E::Big::from(lhs.clone());
        let big_rhs = &E::Big::from(rhs.clone());

        if (ops.predicate)(big_lhs, big_rhs)
            && let (Ok(lhs), Ok(rhs)) = (L::try_from(big_lhs.clone()), R::try_from(big_rhs.clone()))
        {
            let expected = (ops.bigint_op)(big_lhs, big_rhs);
            let actual1 = (ops.cbigint_op1)(lhs.clone(), rhs.clone()).into_big();
            let actual2 = (ops.cbigint_op2)(lhs.clone(), &rhs).into_big();
            let actual3 = (ops.cbigint_op3)(&lhs, rhs.clone()).into_big();
            let actual4 = (ops.cbigint_op4)(&lhs, &rhs).into_big();
            let mut actual5 = GenericBigNum::from_big(E::Big::from(big_lhs.clone()));
            (ops.op_assign1)(&mut actual5, rhs.clone());
            let mut actual6 = GenericBigNum::from_big(E::Big::from(big_lhs.clone()));
            (ops.op_assign2)(&mut actual6, &rhs);
            let label = format!("failed with inputs {}, {}", big_lhs, big_rhs);
            assert_eq!(expected, actual1, "{}", label);
            assert_eq!(expected, actual2, "{}", label);
            assert_eq!(expected, actual3, "{}", label);
            assert_eq!(expected, actual4, "{}", label);
            assert_eq!(expected, actual5.into_big(), "{}", label);
            assert_eq!(expected, actual6.into_big(), "{}", label);
            TestResult::passed()
        } else {
            TestResult::discard()
        }
    }

    // TODO: Implement this macro
    duplicate_generic_bigint_types! {
        duplicate_arith_and_bit_ops! {
            paste! {
                #[quickcheck]
                fn [<test_ op_fn _ bigint_tag>](lhs: bigint_type, rhs: bigint_type) -> TestResult{
                    test_bin_op::<bigint_type, bigint_type>(BinOpsForTypes {
                        predicate: op_test_pred,
                        cbigint_op1: |x, y| op_trait::op_fn(x, y),
                        cbigint_op2: |x, y| op_trait::op_fn(x, y),
                        cbigint_op3: |x, y| op_trait::op_fn(x, y),
                        cbigint_op4: |x, y| op_trait::op_fn(x, y),
                        op_assign1: |x, y| [<op_trait Assign>]::[<op_fn _assign>](x, y),
                        op_assign2: |x, y| [<op_trait Assign>]::[<op_fn _assign>](x, y),
                        bigint_op: |x, y| op_trait::op_fn(x, y),
                    }, lhs, rhs)
                }
            }
        }

        duplicate_shift_ops! {
            duplicate_prims! {
                paste! {
                    #[quickcheck]
                    fn [<test_ op_fn _ prim _rhs_ bigint_tag>](lhs: bigint_type, rhs: u16) -> TestResult{
                        test_shift_op::<prim>(ShiftOpsForType {
                            cbigint_op1: |x, y| op_trait::op_fn(x, y),
                            cbigint_op2: |x, y| op_trait::op_fn(x, y),
                            cbigint_op3: |x, y| op_trait::op_fn(x, y),
                            cbigint_op4: |x, y| op_trait::op_fn(x, y),
                            op_assign1: |x, y| [<op_trait Assign>]::[<op_fn _assign>](x, y),
                            op_assign2: |x, y| [<op_trait Assign>]::[<op_fn _assign>](x, y),
                            bigint_op: |x, y| op_trait::op_fn(x, y),
                        }, lhs, rhs)
                    }
                }
            }
        }

        duplicate_arith_ops! {
            duplicate_prims! {
                paste! {
                    #[quickcheck]
                    fn [<test_ op_fn _ prim _lhs_ bigint_tag>](lhs: prim, rhs: bigint_type) -> TestResult{
                        test_bin_op::<prim, bigint_type>(BinOpsForTypes {
                            predicate: op_test_pred,
                            cbigint_op1: |x, y| op_trait::op_fn(x, y),
                            cbigint_op2: |x, y| op_trait::op_fn(x, y),
                            cbigint_op3: |x, y| op_trait::op_fn(x, y),
                            cbigint_op4: |x, y| op_trait::op_fn(x, y),
                            op_assign1: |x, y| [<op_trait Assign>]::[<op_fn _assign>](x, y),
                            op_assign2: |x, y| [<op_trait Assign>]::[<op_fn _assign>](x, y),
                            bigint_op: |x, y| op_trait::op_fn(x, y),
                        }, lhs, rhs)
                    }
                    #[quickcheck]
                    fn [<test_ op_fn _ prim _rhs_ bigint_tag>](lhs: bigint_type, rhs: prim) -> TestResult {
                        test_bin_op::<bigint_type, prim>(BinOpsForTypes {
                            predicate: op_test_pred,
                            cbigint_op1: |x, y| op_trait::op_fn(x, y),
                            cbigint_op2: |x, y| op_trait::op_fn(x, y),
                            cbigint_op3: |x, y| op_trait::op_fn(x, y),
                            cbigint_op4: |x, y| op_trait::op_fn(x, y),
                            op_assign1: |x, y| [<op_trait Assign>]::[<op_fn _assign>](x, y),
                            op_assign2: |x, y| [<op_trait Assign>]::[<op_fn _assign>](x, y),
                            bigint_op: |x, y| op_trait::op_fn(x, y),
                        }, lhs, rhs)
                    }
                }
            }
        }

        duplicate_prims! {
            paste! {
                #[quickcheck]
                fn [<test_pow_ prim _ bigint_tag>](lhs: bigint_type, rhs: u32) {
                    let rhs = rhs % 64; // limit the exponent to avoid long test times
                    let big_lhs = &BigInt::from(lhs.clone());
                    let expected = big_lhs.pow(rhs);
                    let actual1 = BigInt::from(lhs.clone().pow(rhs));
                    let actual2 = BigInt::from(lhs.pow(rhs));
                    let label = format!("failed with inputs {}, {}", big_lhs, rhs);
                    assert_eq!(expected, actual1, "{}", label);
                    assert_eq!(expected, actual2, "{}", label);
                }
            }
        }
    }
}
