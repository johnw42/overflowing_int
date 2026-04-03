use crate::generic_bigint::encoding::{Decoded, EncodedBigNum, InspectEncoding};
use crate::generic_bigint::struct_def::GenericBigInt;
use crate::{
    duplicate_arith_ops, duplicate_bit_ops, duplicate_prims, duplicate_shift_ops, duplicate_uprims,
};
use num_bigint::BigInt;
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedSub, Pow, PrimInt};
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
    fn call<'a, 'b, L, R>(lhs: L, rhs: R) -> GenericBigInt<'e, E>
    where
        L: InspectEncoding<'a, E::Small, E::Big>,
        R: InspectEncoding<'b, E::Small, E::Big>,
    {
        match (lhs.decode(), rhs.decode()) {
            (Decoded::Small(lhs), Decoded::Small(rhs)) => {
                if let Ok(out) = Self::on_small(lhs, rhs) {
                    GenericBigInt::from_small(out)
                } else {
                    GenericBigInt::from_big(Self::on_big_small(Cow::Owned(E::Big::from(lhs)), rhs))
                }
            }
            (Decoded::Small(small_lhs), Decoded::Big(big_rhs)) => {
                GenericBigInt::from_big(Self::on_small_big(small_lhs, big_rhs))
            }
            (Decoded::Big(big_lhs), Decoded::Small(small_rhs)) => {
                GenericBigInt::from_big(Self::on_big_small(big_lhs, small_rhs))
            }
            (Decoded::Big(big_lhs), Decoded::Big(big_rhs)) => {
                GenericBigInt::from_big(Self::on_big(big_lhs, big_rhs))
            }
        }
    }

    /// Calls a version of the binary operator that updates a bigint argument in place.
    #[inline]
    fn call_update<'a, 'b, 'c, R>(lhs: &'a mut GenericBigInt<'e, E>, rhs: R)
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
                            Cow::Owned(E::Big::from(small_rhs)),
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
    fn call<'a, 'b, L, R>(lhs: L, rhs: R) -> GenericBigInt<'e, E>
    where
        L: InspectEncoding<'a, E::Small, E::Big>,
        R: InspectEncoding<'b, E::Small, E::Big>,
    {
        GenericBigInt::from_big(Self::on_big(lhs.into_big_cow(), rhs.into_big_cow()))
    }

    #[inline]
    fn call_update<'a, 'b, 'c, R>(lhs: &'a mut GenericBigInt<'e, E>, rhs: R)
    where
        R: InspectEncoding<'c, E::Small, E::Big>,
    {
        lhs.update_encoding(|encoding| match encoding {
            Decoded::Small(small_lhs) => {
                *encoding = Decoded::Big(Self::on_big(
                    Cow::Owned(E::Big::from(*small_lhs)),
                    rhs.big_cow(),
                ));
            }
            Decoded::Big(big_lhs) => {
                Self::update_big(big_lhs, rhs.big_cow());
            }
        });
    }
}
trait ShiftOp<'e, E: EncodedBigNum<'e>> {
    duplicate_prims! {
        paste! {
            fn [<on_big_ prim>](lhs: Cow<E::Big>, rhs: prim) -> E::Big;
            fn [<update_big_ prim>](lhs: &mut E::Big, rhs: prim);

            fn [<call_ prim>]<'a, L>(lhs: L, rhs: prim) -> GenericBigInt<'e, E>
            where
                L: InspectEncoding<'a, E::Small, E::Big>,
            {
                GenericBigInt::from_big(Self::[<on_big_ prim>](lhs.into_big_cow(), rhs))
            }

            #[inline]
            fn [<call_update_big_ prim>](lhs: &mut GenericBigInt<'e, E>, rhs: prim) {
                lhs.update_encoding(|encoding| match encoding {
                    Decoded::Small(small_lhs) => {
                        *encoding = Decoded::Big(Self::[<on_big_ prim>](
                            Cow::Owned(E::Big::from(*small_lhs)),
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
                    (Cow::Borrowed(lhs), Cow::Borrowed(rhs)) => op_trait::op_fn(lhs, rhs),
                    (Cow::Borrowed(lhs), Cow::Owned(rhs)) => op_trait::op_fn(lhs, rhs),
                    (Cow::Owned(lhs), Cow::Borrowed(rhs)) => op_trait::op_fn(lhs, rhs),
                    (Cow::Owned(lhs), Cow::Owned(rhs)) => op_trait::op_fn(lhs, rhs),
                }
            }

            fn on_small(lhs: E::Small, rhs: E::Small) -> Result<E::Small, ()> {
                lhs.[<checked_ op_fn>](rhs).ok_or(())
            }

            fn on_big_small(lhs: Cow<E::Big>, rhs: E::Small) -> E::Big {
                match lhs {
                    Cow::Borrowed(lhs) => op_trait::op_fn(lhs, rhs),
                    Cow::Owned(lhs) => op_trait::op_fn(lhs, rhs),
                }
            }

            fn on_small_big(lhs: E::Small, rhs: Cow<E::Big>) -> E::Big {
                match rhs {
                    Cow::Borrowed(rhs) => E::Small::[<op_fn _bigint_left_ref>](lhs, rhs),
                    Cow::Owned(rhs) => E::Small::[<op_fn _bigint_left>](lhs, rhs),
                }
            }

            fn update_big(lhs: &mut E::Big, rhs: Cow<E::Big>) {
                match rhs {
                    Cow::Borrowed(rhs) => [<op_trait Assign>]::[<op_fn _assign>](lhs, rhs),
                    Cow::Owned(rhs) => [<op_trait Assign>]::[<op_fn _assign>](lhs, rhs),
                }

            }

            fn update_small(lhs: &mut E::Big, rhs: E::Small) {
                [<op_trait Assign>]::[<op_fn _assign>](lhs, rhs);
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
                    (Cow::Borrowed(lhs), Cow::Borrowed(rhs)) => op_trait::op_fn(lhs, rhs),
                    (Cow::Borrowed(lhs), Cow::Owned(rhs)) => op_trait::op_fn(lhs, rhs),
                    (Cow::Owned(lhs), Cow::Borrowed(rhs)) => op_trait::op_fn(lhs, rhs),
                    (Cow::Owned(lhs), Cow::Owned(rhs)) => op_trait::op_fn(lhs, rhs),
                }
            }

            fn update_big(lhs: &mut E::Big, rhs: Cow<E::Big>)
            {
                match rhs {
                    Cow::Borrowed(rhs) => [<op_trait Assign>]::[<op_fn _assign>](lhs, rhs),
                    Cow::Owned(rhs) => [<op_trait Assign>]::[<op_fn _assign>](lhs, rhs),
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
                        Cow::Borrowed(lhs) => op_trait::op_fn(lhs, rhs),
                        Cow::Owned(lhs) => op_trait::op_fn(lhs, rhs),
                    }
                }

                fn [<update_big_ prim>](lhs: &mut E::Big, rhs: prim) {
                    [<op_trait Assign>]::[<op_fn _assign>](lhs, rhs);
                }
            }
        }
    }
}

// MARK: Operator Trait Implementations
// -----------------------------------------------------------------------------
duplicate_arith_ops! {
    paste! {
        impl<'a, T, E: EncodedBigNum<'a>> op_trait<T> for GenericBigInt<'a, E>
        where
            T: InspectEncoding<'a, E::Small, E::Big>,
        {
            type Output = GenericBigInt<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, T, E: EncodedBigNum<'a>> op_trait<T> for &GenericBigInt<'a, E>
        where
            T: InspectEncoding<'a, E::Small, E::Big>,
        {
            type Output = GenericBigInt<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, T, E: EncodedBigNum<'a>> [<op_trait Assign>]<T> for GenericBigInt<'a, E>
        where
            T: InspectEncoding<'a, E::Small, E::Big>
        {
            fn [<op_fn _assign>](&mut self, rhs: T) {
                [<op_trait Op>]::call_update(self, rhs);
            }
        }
    }

    crate::duplicate_prims! {
        paste! {
            impl<'a, E: EncodedBigNum<'a>> op_trait<GenericBigInt<'a, E>> for prim {
                type Output = GenericBigInt<'a, E>;

                #[inline(never)]
                fn op_fn(self, rhs: GenericBigInt<'a, E>) -> Self::Output {
                    [<op_trait Op>]::call(self, rhs)
                }
            }

            impl<'a, E: EncodedBigNum<'a>> op_trait<GenericBigInt<'a, E>> for &prim {
                type Output = GenericBigInt<'a, E>;

                fn op_fn(self, rhs: GenericBigInt<'a, E>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }

            impl<'a, E: EncodedBigNum<'a>> op_trait<&GenericBigInt<'a, E>> for prim {
                type Output = GenericBigInt<'a, E>;

                #[inline(never)]
                fn op_fn(self, rhs: &GenericBigInt<'a, E>) -> Self::Output {
                    [<op_trait Op>]::call(self, rhs)
                }
            }

            impl<'a, E: EncodedBigNum<'a>> op_trait<&GenericBigInt<'a, E>> for &prim {
                type Output = GenericBigInt<'a, E>;

                fn op_fn(self, rhs: &GenericBigInt<'a, E>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }
        }
    }
}

duplicate_bit_ops! {
    paste! {
        impl<'a, 'b, T, E: EncodedBigNum<'a>> op_trait<T> for GenericBigInt<'a, E>
        where
            T: InspectEncoding<'b, E::Small, E::Big>,
        {
            type Output = GenericBigInt<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, 'b, T, E: EncodedBigNum<'a>> op_trait<T> for &GenericBigInt<'a, E>
        where
            T: InspectEncoding<'b, E::Small, E::Big>,
        {
            type Output = GenericBigInt<'a, E>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [<op_trait Op>]::call(self, rhs)
            }
        }

        impl<'a, 'b, T, E: EncodedBigNum<'a>> [<op_trait Assign>]<T> for GenericBigInt<'a, E>
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
            impl<'a, E: EncodedBigNum<'a>> op_trait<prim> for GenericBigInt<'a, E> {
                type Output = GenericBigInt<'a, E>;

                fn op_fn(self, rhs: prim) -> Self::Output {
                    [<op_trait Op>]::[<call_ prim>](self, rhs)
                }
            }

            impl<'a, E: EncodedBigNum<'a>> op_trait<&prim> for GenericBigInt<'a, E> {
                type Output = GenericBigInt<'a, E>;

                fn op_fn(self, rhs: &prim) -> Self::Output {
                    self.op_fn(*rhs)
                }
            }

            impl<'a, E: EncodedBigNum<'a>> op_trait<prim> for &GenericBigInt<'a, E> {
                type Output = GenericBigInt<'a, E>;

                fn op_fn(self, rhs: prim) -> Self::Output {
                    [<op_trait Op>]::[<call_ prim>](self, rhs)
                }
            }

            impl<'a, E: EncodedBigNum<'a>> op_trait<&prim> for &GenericBigInt<'a, E> {
                type Output = GenericBigInt<'a, E>;

                fn op_fn(self, rhs: &prim) -> Self::Output {
                    self.op_fn(*rhs)
                }
            }

            impl<'a, E: EncodedBigNum<'a>> [<op_trait Assign>]<prim> for GenericBigInt<'a, E> {
                fn [<op_fn _assign>](&mut self, rhs: prim) {
                    [<op_trait Op>]::[<call_update_big_ prim>](self, rhs);
                }
            }

            impl<'a, E: EncodedBigNum<'a>> [<op_trait Assign>]<&prim> for GenericBigInt<'a, E> {
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
    impl<'a, E: EncodedBigNum<'a>> Pow<prim> for GenericBigInt<'a, E> {
        type Output = GenericBigInt<'a, E>;

        fn pow(self, rhs: prim) -> Self::Output {
            BigInt::from(self).pow(rhs).into()
        }
    }

    impl<'a, E: EncodedBigNum<'a>> Pow<&prim> for GenericBigInt<'a, E> {
        type Output = GenericBigInt<'a, E>;

        fn pow(self, rhs: &prim) -> Self::Output {
            BigInt::from(self).pow(rhs).into()
        }
    }

    impl<'a, E: EncodedBigNum<'a>> Pow<prim> for &GenericBigInt<'a, E> {
        type Output = GenericBigInt<'a, E>;

        fn pow(self, rhs: prim) -> Self::Output {
            BigInt::from(self).pow(rhs).into()
        }
    }

    impl<'a, E: EncodedBigNum<'a>> Pow<&prim> for &GenericBigInt<'a, E> {
        type Output = GenericBigInt<'a, E>;

        fn pow(self, rhs: &prim) -> Self::Output {
            BigInt::from(self).pow(rhs).into()
        }
    }
}

// MARK: Testts
#[cfg(test)]
mod test {
    use std::fmt::Display;

    use super::*;
    use crate::duplicate_arith_and_bit_ops;
    use crate::duplicate_generic_bigint_types;
    use num_traits::{Pow, Zero};
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    fn always(_lhs: &BigInt, _rhs: &BigInt) -> bool {
        true
    }

    fn nonzero_rhs(_lhs: &BigInt, rhs: &BigInt) -> bool {
        !rhs.is_zero()
    }

    struct ShiftOpsForType<R, E: EncodedBigNum<'static>> {
        cbigint_op1: fn(GenericBigInt<'static, E>, R) -> GenericBigInt<'static, E>,
        cbigint_op2: fn(GenericBigInt<'static, E>, &R) -> GenericBigInt<'static, E>,
        cbigint_op3: fn(&GenericBigInt<'static, E>, R) -> GenericBigInt<'static, E>,
        cbigint_op4: fn(&GenericBigInt<'static, E>, &R) -> GenericBigInt<'static, E>,
        op_assign1: fn(&mut GenericBigInt<'static, E>, R),
        op_assign2: fn(&mut GenericBigInt<'static, E>, &R),
        bigint_op: fn(&BigInt, R) -> BigInt,
    }

    struct BinOpsForTypes<L, R, E: EncodedBigNum<'static>> {
        predicate: fn(&BigInt, &BigInt) -> bool,
        cbigint_op1: fn(L, R) -> GenericBigInt<'static, E>,
        cbigint_op2: fn(L, &R) -> GenericBigInt<'static, E>,
        cbigint_op3: fn(&L, R) -> GenericBigInt<'static, E>,
        cbigint_op4: fn(&L, &R) -> GenericBigInt<'static, E>,
        op_assign1: fn(&mut GenericBigInt<'static, E>, R),
        op_assign2: fn(&mut GenericBigInt<'static, E>, &R),
        bigint_op: fn(&BigInt, &BigInt) -> BigInt,
    }

    fn test_shift_op<R, E: EncodedBigNum<'static>>(
        ops: ShiftOpsForType<R, E>,
        lhs: GenericBigInt<'static, E>,
        rhs: u16,
    ) -> TestResult
    where
        R: TryFrom<u16> + Copy + Ord + Zero + Display,
    {
        let big_lhs = &BigInt::from(lhs.clone());
        let lhs = GenericBigInt::from(big_lhs.clone());
        if let Ok(rhs) = R::try_from(rhs) {
            assert!(rhs >= R::zero(), "shift amount must be non-negative");
            let expected = (ops.bigint_op)(big_lhs, rhs);
            let actual1 = BigInt::from((ops.cbigint_op1)(lhs.clone(), rhs));
            let actual2 = BigInt::from((ops.cbigint_op2)(lhs.clone(), &rhs));
            let actual3 = BigInt::from((ops.cbigint_op3)(&lhs, rhs));
            let actual4 = BigInt::from((ops.cbigint_op4)(&lhs, &rhs));
            let mut actual5 = lhs.clone();
            (ops.op_assign1)(&mut actual5, rhs);
            let mut actual6 = lhs.clone();
            (ops.op_assign2)(&mut actual6, &rhs);
            let label = format!("failed with inputs {}, {}", big_lhs, rhs);
            assert_eq!(expected, actual1, "{}", label);
            assert_eq!(expected, actual2, "{}", label);
            assert_eq!(expected, actual3, "{}", label);
            assert_eq!(expected, actual4, "{}", label);
            assert_eq!(expected, BigInt::from(actual5), "{}", label);
            assert_eq!(expected, BigInt::from(actual6), "{}", label);
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
        L: TryFrom<BigInt> + Clone,
        R: TryFrom<BigInt> + Clone,
        BigInt: From<L>,
        BigInt: From<R>,
    {
        let big_lhs = &BigInt::from(lhs.clone());
        let big_rhs = &BigInt::from(rhs.clone());

        if (ops.predicate)(big_lhs, big_rhs)
            && let (Ok(lhs), Ok(rhs)) = (L::try_from(big_lhs.clone()), R::try_from(big_rhs.clone()))
        {
            let expected = (ops.bigint_op)(big_lhs, big_rhs);
            let actual1 = BigInt::from((ops.cbigint_op1)(lhs.clone(), rhs.clone()));
            let actual2 = BigInt::from((ops.cbigint_op2)(lhs.clone(), &rhs));
            let actual3 = BigInt::from((ops.cbigint_op3)(&lhs, rhs.clone()));
            let actual4 = BigInt::from((ops.cbigint_op4)(&lhs, &rhs));
            let mut actual5 = big_lhs.clone().into();
            (ops.op_assign1)(&mut actual5, rhs.clone());
            let mut actual6 = big_lhs.clone().into();
            (ops.op_assign2)(&mut actual6, &rhs);
            let label = format!("failed with inputs {}, {}", big_lhs, big_rhs);
            assert_eq!(expected, actual1, "{}", label);
            assert_eq!(expected, actual2, "{}", label);
            assert_eq!(expected, actual3, "{}", label);
            assert_eq!(expected, actual4, "{}", label);
            assert_eq!(expected, BigInt::from(actual5), "{}", label);
            assert_eq!(expected, BigInt::from(actual6), "{}", label);
            TestResult::passed()
        } else {
            TestResult::discard()
        }
    }

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
