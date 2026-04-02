use super::encoding::{Encoding, IntoBigIntCow, IntoEncoding};
use crate::cow_bigint::CowBigInt;
use crate::cow_bigint::small_num::SmallInt;
use crate::{
    duplicate_arith_ops, duplicate_bit_ops, duplicate_prims, duplicate_shift_ops, duplicate_uprims,
};
use duplicate::duplicate;
use num_bigint::BigInt;
use num_traits::Pow;
use paste::paste;
use std::borrow::Cow;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

// MARK: Meta-Operator Trait Definitions
// -----------------------------------------------------------------------------
trait ArithOp {
    fn on_big_small(lhs: Cow<BigInt>, rhs: SmallInt) -> BigInt;
    fn on_small(lhs: SmallInt, rhs: SmallInt) -> Result<SmallInt, ()>;
    fn on_small_big(lhs: SmallInt, rhs: Cow<BigInt>) -> BigInt;
    fn on_big(lhs: Cow<BigInt>, rhs: Cow<BigInt>) -> BigInt;
    fn update_big(lhs: &mut BigInt, rhs: Cow<BigInt>);
    fn update_small(lhs: &mut BigInt, rhs: SmallInt);

    /// Calls a version of the binary operator that returns a new number.
    #[inline]
    fn call<'a, 'b, L, R>(lhs: L, rhs: R) -> CowBigInt<'static>
    where
        L: IntoEncoding<'a, SmallInt, BigInt>,
        R: IntoEncoding<'b, SmallInt, BigInt>,
    {
        match (lhs.into_encoding(), rhs.into_encoding()) {
            (Encoding::Small(lhs), Encoding::Small(rhs)) => {
                if let Ok(out) = Self::on_small(lhs, rhs) {
                    out.into()
                } else {
                    Self::on_big_small(Cow::Owned(BigInt::from(lhs)), rhs).into()
                }
            }
            (Encoding::Small(small_lhs), Encoding::Big(big_rhs)) => {
                Self::on_small_big(small_lhs, big_rhs).into()
            }
            (Encoding::Big(big_lhs), Encoding::Small(small_rhs)) => {
                Self::on_big_small(big_lhs, small_rhs).into()
            }
            (Encoding::Big(big_lhs), Encoding::Big(big_rhs)) => {
                Self::on_big(big_lhs, big_rhs).into()
            }
        }
    }

    /// Calls a version of the binary operator that updates a bigint argument in place.
    #[inline]
    fn call_update<'a, 'b, 'c, R>(lhs: &'a mut CowBigInt<'b>, rhs: R)
    where
        R: IntoEncoding<'c, SmallInt, BigInt>,
    {
        lhs.update_encoding(|encoding| match encoding {
            Encoding::Small(small_lhs) => match rhs.into_encoding() {
                Encoding::Small(small_rhs) => match Self::on_small(*small_lhs, small_rhs) {
                    Ok(out) => *encoding = Encoding::Small(out),
                    Err(()) => {
                        *encoding = Encoding::Big(Cow::Owned(Self::on_small_big(
                            *small_lhs,
                            Cow::Owned(BigInt::from(small_rhs)),
                        )));
                    }
                },
                Encoding::Big(big_rhs) => {
                    *encoding = Encoding::Big(Cow::Owned(Self::on_small_big(*small_lhs, big_rhs)));
                }
            },
            Encoding::Big(big_lhs) => match rhs.into_encoding() {
                Encoding::Small(small_rhs) => {
                    Self::update_small(big_lhs.to_mut(), small_rhs);
                }
                Encoding::Big(big_rhs) => {
                    Self::update_big(big_lhs.to_mut(), big_rhs);
                }
            },
        });
    }
}

trait BitOp {
    fn on_big(lhs: Cow<BigInt>, rhs: Cow<BigInt>) -> BigInt;
    fn update_big(lhs: &mut BigInt, rhs: Cow<BigInt>);

    #[inline]
    fn call<'a, 'b, L, R>(lhs: L, rhs: R) -> CowBigInt<'static>
    where
        L: IntoBigIntCow<'a>,
        R: IntoBigIntCow<'b>,
    {
        Self::on_big(lhs.into_bigint_cow(), rhs.into_bigint_cow()).into()
    }

    #[inline]
    fn call_update<'a, 'b, 'c, R>(lhs: &'a mut CowBigInt<'b>, rhs: R)
    where
        R: IntoBigIntCow<'c>,
    {
        lhs.update_encoding(|encoding| match encoding {
            Encoding::Small(small_lhs) => {
                *encoding = Encoding::Big(Cow::Owned(Self::on_big(
                    Cow::Owned(BigInt::from(*small_lhs)),
                    rhs.into_bigint_cow(),
                )));
            }
            Encoding::Big(big_lhs) => {
                Self::update_big(big_lhs.to_mut(), rhs.into_bigint_cow());
            }
        });
    }
}
trait ShiftOp {
    duplicate_prims! {
        paste! {
            fn [<on_big_ prim>](lhs: Cow<BigInt>, rhs: prim) -> BigInt;
            fn [<update_big_ prim>](lhs: &mut BigInt, rhs: prim);

            fn [<call_ prim>]<'a, L>(lhs: L, rhs: prim) -> CowBigInt<'static>
            where
                L: IntoBigIntCow<'a>,
            {
                Self::[<on_big_ prim>](lhs.into_bigint_cow(), rhs).into()
            }

            #[inline]
            fn [<call_update_big_ prim>](lhs: &mut CowBigInt<'_>, rhs: prim) {
                lhs.update_encoding(|encoding| match encoding {
                    Encoding::Small(small_lhs) => {
                        *encoding = Encoding::Big(Cow::Owned(Self::[<on_big_ prim>](
                            Cow::Owned(BigInt::from(*small_lhs)),
                            rhs,
                        )));
                    }
                    Encoding::Big(big_lhs) => {
                        Self::[<update_big_ prim>](big_lhs.to_mut(), rhs);
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
        struct [< op_trait Op >];

        impl ArithOp for [< op_trait Op >] {

            fn on_big(lhs: Cow<BigInt>, rhs: Cow<BigInt>) -> BigInt {
                match (lhs, rhs) {
                    (Cow::Borrowed(lhs), Cow::Borrowed(rhs)) => op_trait::op_fn(lhs, rhs),
                    (Cow::Borrowed(lhs), Cow::Owned(rhs)) => op_trait::op_fn(lhs, rhs),
                    (Cow::Owned(lhs), Cow::Borrowed(rhs)) => op_trait::op_fn(lhs, rhs),
                    (Cow::Owned(lhs), Cow::Owned(rhs)) => op_trait::op_fn(lhs, rhs),
                }
            }

            fn on_small(lhs: SmallInt, rhs: SmallInt) -> Result<SmallInt, ()> {
                lhs.[<checked_ op_fn>](rhs).ok_or(())
            }

            fn on_big_small(lhs: Cow<BigInt>, rhs: SmallInt) -> BigInt {
                match lhs {
                    Cow::Borrowed(lhs) => op_trait::op_fn(lhs, rhs),
                    Cow::Owned(lhs) => op_trait::op_fn(lhs, rhs),
                }
            }

            fn on_small_big(lhs: SmallInt, rhs: Cow<BigInt>) -> BigInt {
                match rhs {
                    Cow::Borrowed(rhs) => op_trait::op_fn(lhs, rhs),
                    Cow::Owned(rhs) => op_trait::op_fn(lhs, rhs),
                }
            }

            fn update_big(lhs: &mut BigInt, rhs: Cow<BigInt>) {
                match rhs {
                    Cow::Borrowed(rhs) => [<op_trait Assign>]::[<op_fn _assign>](lhs, rhs),
                    Cow::Owned(rhs) => [<op_trait Assign>]::[<op_fn _assign>](lhs, rhs),
                }

            }

            fn update_small(lhs: &mut BigInt, rhs: SmallInt) {
                [<op_trait Assign>]::[<op_fn _assign>](lhs, rhs);
            }
        }
    }
}

duplicate_bit_ops! {
    paste! {
        struct [< op_trait Op >];

        impl BitOp for [< op_trait Op >] {

            fn on_big(lhs: Cow<BigInt>, rhs: Cow<BigInt>) -> BigInt
            {
                match (lhs, rhs) {
                    (Cow::Borrowed(lhs), Cow::Borrowed(rhs)) => op_trait::op_fn(lhs, rhs),
                    (Cow::Borrowed(lhs), Cow::Owned(rhs)) => op_trait::op_fn(lhs, rhs),
                    (Cow::Owned(lhs), Cow::Borrowed(rhs)) => op_trait::op_fn(lhs, rhs),
                    (Cow::Owned(lhs), Cow::Owned(rhs)) => op_trait::op_fn(lhs, rhs),
                }
            }

            fn update_big(lhs: &mut BigInt, rhs: Cow<BigInt>)
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

    impl ShiftOp for paste! { [<op_trait Op>] } {
        duplicate_prims! {
            paste! {
                fn [<on_big_ prim>](lhs: Cow<BigInt>, rhs: prim) -> BigInt {
                    match lhs {
                        Cow::Borrowed(lhs) => op_trait::op_fn(lhs, rhs),
                        Cow::Owned(lhs) => op_trait::op_fn(lhs, rhs),
                    }
                }

                fn [<update_big_ prim>](lhs: &mut BigInt, rhs: prim) {
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
        impl<'a, T> op_trait<T> for CowBigInt<'a>
        where
            T: IntoEncoding<'a, SmallInt, BigInt>,
        {
            type Output = CowBigInt<'a>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [< op_trait Op >]::call(self, rhs)
            }
        }

        impl<'a, T> op_trait<T> for &CowBigInt<'a>
        where
            T: IntoEncoding<'a, SmallInt, BigInt>,
        {
            type Output = CowBigInt<'a>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [< op_trait Op >]::call(self, rhs)
            }
        }

        impl<'a, T> [< op_trait Assign >]<T> for CowBigInt<'a>
        where
            T: IntoEncoding<'a, SmallInt, BigInt>
        {
            fn [< op_fn _assign >](&mut self, rhs: T) {
                [< op_trait Op >]::call_update(self, rhs);
            }
        }
    }

    crate::duplicate_prims! {
        paste! {
            impl<'a> op_trait<CowBigInt<'a>> for prim {
                type Output = CowBigInt<'a>;
                #[inline(never)]
                fn op_fn(self, rhs: CowBigInt<'a>) -> Self::Output {
                    [< op_trait Op >]::call(self, rhs)
                }
            }

            impl<'a> op_trait<CowBigInt<'a>> for &prim {
                type Output = CowBigInt<'a>;
                fn op_fn(self, rhs: CowBigInt<'a>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }

            impl<'a> op_trait<&CowBigInt<'a>> for prim {
                type Output = CowBigInt<'a>;
                #[inline(never)]
                fn op_fn(self, rhs: &CowBigInt<'a>) -> Self::Output {
                    [< op_trait Op >]::call(self, rhs)
                }
            }

            impl<'a> op_trait<&CowBigInt<'a>> for &prim {
                type Output = CowBigInt<'a>;
                fn op_fn(self, rhs: &CowBigInt<'a>) -> Self::Output {
                    (*self).op_fn(rhs)
                }
            }
        }
    }
}

duplicate_bit_ops! {
    paste! {
        impl<'a, 'b, T> op_trait<T> for CowBigInt<'a>
        where
            T: IntoBigIntCow<'b>,
        {
            type Output = CowBigInt<'a>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [< op_trait Op >]::call(self, rhs)
            }
        }

        impl<'a, 'b, T> op_trait<T> for &CowBigInt<'a>
        where
            T: IntoBigIntCow<'b>,
        {
            type Output = CowBigInt<'a>;

            fn op_fn(self, rhs: T) -> Self::Output {
                [< op_trait Op >]::call(self, rhs)
            }
        }

        impl<'a, 'b, T> [< op_trait Assign >]<T> for CowBigInt<'a>
        where
            T: IntoBigIntCow<'b>
        {
            fn [< op_fn _assign >](&mut self, rhs: T) {
                [< op_trait Op >]::call_update(self, rhs);
            }
        }
    }
}

duplicate_shift_ops! {
    duplicate_prims! {
        paste! {
            impl<'a> op_trait<prim> for CowBigInt<'a> {
                type Output = CowBigInt<'a>;
                fn op_fn(self, rhs: prim) -> Self::Output {
                    [< op_trait Op >]::[<call_ prim>](self, rhs)
                }
            }

            impl<'a> op_trait<&prim> for CowBigInt<'a> {
                type Output = CowBigInt<'a>;
                fn op_fn(self, rhs: &prim) -> Self::Output {
                    self.op_fn(*rhs)
                }
            }

            impl<'a> op_trait<prim> for &CowBigInt<'a> {
                type Output = CowBigInt<'a>;
                fn op_fn(self, rhs: prim) -> Self::Output {
                    [< op_trait Op >]::[<call_ prim>](self, rhs)
                }
            }

            impl<'a> op_trait<&prim> for &CowBigInt<'a> {
                type Output = CowBigInt<'a>;
                fn op_fn(self, rhs: &prim) -> Self::Output {
                    self.op_fn(*rhs)
                }
            }

            impl<'a> [< op_trait Assign >]<prim> for CowBigInt<'a> {
                fn [< op_fn _assign >](&mut self, rhs: prim) {
                    [< op_trait Op >]::[<call_update_big_ prim>](self, rhs);
                }
            }

            impl<'a> [< op_trait Assign >]<&prim> for CowBigInt<'a> {
                fn [< op_fn _assign >](&mut self, rhs: &prim) {
                    self.[< op_fn _assign >](*rhs);
                }
            }
        }
    }
}

// MARK: Pow Operator Implementations
// -----------------------------------------------------------------------------
duplicate_uprims! {
    impl<'a> Pow<prim> for CowBigInt<'a> {
        type Output = CowBigInt<'a>;

        fn pow(self, rhs: prim) -> Self::Output {
            BigInt::from(self).pow(rhs).into()
        }
    }

    impl<'a> Pow<&prim> for CowBigInt<'a> {
        type Output = CowBigInt<'a>;

        fn pow(self, rhs: &prim) -> Self::Output {
            BigInt::from(self).pow(rhs).into()
        }
    }

    impl<'a> Pow<prim> for &CowBigInt<'a> {
        type Output = CowBigInt<'a>;

        fn pow(self, rhs: prim) -> Self::Output {
            BigInt::from(self).pow(rhs).into()
        }
    }

    impl<'a> Pow<&prim> for &CowBigInt<'a> {
        type Output = CowBigInt<'a>;

        fn pow(self, rhs: &prim) -> Self::Output {
            BigInt::from(self).pow(rhs).into()
        }
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Display;

    use super::*;
    use crate::duplicate_arith_and_bit_ops;
    use num_traits::{Pow, Zero};
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    fn always(_lhs: &BigInt, _rhs: &BigInt) -> bool {
        true
    }

    fn nonzero_rhs(_lhs: &BigInt, rhs: &BigInt) -> bool {
        !rhs.is_zero()
    }

    struct ShiftOpsForType<R> {
        cbigint_op1: fn(CowBigInt<'static>, R) -> CowBigInt<'static>,
        cbigint_op2: fn(CowBigInt<'static>, &R) -> CowBigInt<'static>,
        cbigint_op3: fn(&CowBigInt<'static>, R) -> CowBigInt<'static>,
        cbigint_op4: fn(&CowBigInt<'static>, &R) -> CowBigInt<'static>,
        op_assign1: fn(&mut CowBigInt<'static>, R),
        op_assign2: fn(&mut CowBigInt<'static>, &R),
        bigint_op: fn(&BigInt, R) -> BigInt,
    }

    struct BinOpsForTypes<L, R> {
        predicate: fn(&BigInt, &BigInt) -> bool,
        cbigint_op1: fn(L, R) -> CowBigInt<'static>,
        cbigint_op2: fn(L, &R) -> CowBigInt<'static>,
        cbigint_op3: fn(&L, R) -> CowBigInt<'static>,
        cbigint_op4: fn(&L, &R) -> CowBigInt<'static>,
        op_assign1: fn(&mut CowBigInt<'static>, R),
        op_assign2: fn(&mut CowBigInt<'static>, &R),
        bigint_op: fn(&BigInt, &BigInt) -> BigInt,
    }

    fn test_shift_op<R>(ops: ShiftOpsForType<R>, lhs: CowBigInt<'static>, rhs: u16) -> TestResult
    where
        R: TryFrom<u16> + Copy + Ord + Zero + Display,
    {
        let big_lhs = &BigInt::from(lhs.clone());
        let lhs = CowBigInt::from(big_lhs.clone());
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

    fn test_bin_op<L, R>(ops: BinOpsForTypes<L, R>, lhs: L, rhs: R) -> TestResult
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

    duplicate_arith_and_bit_ops! {
        paste! {
            #[quickcheck]
            fn [< test_ op_fn >](lhs: CowBigInt<'static>, rhs: CowBigInt<'static>) -> TestResult{
                test_bin_op::<CowBigInt, CowBigInt>(BinOpsForTypes {
                    predicate: op_test_pred,
                    cbigint_op1: |x, y| op_trait::op_fn(x, y),
                    cbigint_op2: |x, y| op_trait::op_fn(x, y),
                    cbigint_op3: |x, y| op_trait::op_fn(x, y),
                    cbigint_op4: |x, y| op_trait::op_fn(x, y),
                    op_assign1: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                    op_assign2: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                    bigint_op: |x, y| op_trait::op_fn(x, y),
                }, lhs, rhs)
            }
        }
    }

    duplicate_shift_ops! {
         duplicate_prims! {
            paste! {
                #[quickcheck]
                fn [< test_ op_fn _ prim _rhs >](lhs: CowBigInt<'static>, rhs: u16) -> TestResult{
                    test_shift_op::<prim>(ShiftOpsForType {
                        cbigint_op1: |x, y| op_trait::op_fn(x, y),
                        cbigint_op2: |x, y| op_trait::op_fn(x, y),
                        cbigint_op3: |x, y| op_trait::op_fn(x, y),
                        cbigint_op4: |x, y| op_trait::op_fn(x, y),
                        op_assign1: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                        op_assign2: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
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
                fn [< test_ op_fn _ prim _lhs >](lhs: prim, rhs: CowBigInt<'static>) -> TestResult{
                    test_bin_op::<prim, CowBigInt>(BinOpsForTypes {
                        predicate: op_test_pred,
                        cbigint_op1: |x, y| op_trait::op_fn(x, y),
                        cbigint_op2: |x, y| op_trait::op_fn(x, y),
                        cbigint_op3: |x, y| op_trait::op_fn(x, y),
                        cbigint_op4: |x, y| op_trait::op_fn(x, y),
                        op_assign1: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                        op_assign2: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                        bigint_op: |x, y| op_trait::op_fn(x, y),
                    }, lhs, rhs)
                }
                #[quickcheck]
                fn [< test_ op_fn _ prim _rhs >](lhs: CowBigInt<'static>, rhs: prim) -> TestResult {
                    test_bin_op::<CowBigInt, prim>(BinOpsForTypes {
                        predicate: op_test_pred,
                        cbigint_op1: |x, y| op_trait::op_fn(x, y),
                        cbigint_op2: |x, y| op_trait::op_fn(x, y),
                        cbigint_op3: |x, y| op_trait::op_fn(x, y),
                        cbigint_op4: |x, y| op_trait::op_fn(x, y),
                        op_assign1: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                        op_assign2: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                        bigint_op: |x, y| op_trait::op_fn(x, y),
                    }, lhs, rhs)
                }
            }
        }
    }

    duplicate_prims! {
        paste! {
            #[quickcheck]
            fn [< test_pow_ prim >](lhs: CowBigInt<'static>, rhs: u32) {
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
