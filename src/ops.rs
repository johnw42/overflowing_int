use crate::SmallInt;
use crate::cbigint::CBigInt;
use crate::encoding::{Encoding, ToCow, ToEncodingCow};
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
    fn call<'a, L, R>(lhs: L, rhs: R) -> CBigInt
    where
        L: ToEncodingCow<'a, BigInt>,
        R: ToEncodingCow<'a, BigInt>,
    {
        match (lhs.to_encoding_cow(), rhs.to_encoding_cow()) {
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
    fn call_update<'a, R>(lhs: &mut CBigInt, rhs: R)
    where
        R: ToEncodingCow<'a, BigInt>,
    {
        lhs.0.update_encoding(|encoding| match encoding {
            Encoding::Small(small_lhs) => match rhs.to_encoding_cow() {
                Encoding::Small(small_rhs) => match Self::on_small(*small_lhs, small_rhs) {
                    Ok(out) => *encoding = Encoding::Small(out),
                    Err(()) => {
                        *encoding = Encoding::Big(Self::on_small_big(
                            *small_lhs,
                            Cow::Owned(BigInt::from(small_rhs)),
                        ));
                    }
                },
                Encoding::Big(big_rhs) => {
                    *encoding = Encoding::Big(Self::on_small_big(*small_lhs, big_rhs));
                }
            },
            Encoding::Big(big_lhs) => match rhs.to_encoding_cow() {
                Encoding::Small(small_rhs) => {
                    Self::update_small(big_lhs, small_rhs);
                }
                Encoding::Big(big_rhs) => {
                    Self::update_big(big_lhs, big_rhs);
                }
            },
        });
    }
}
trait BitOp {
    fn on_big(lhs: Cow<BigInt>, rhs: Cow<BigInt>) -> BigInt;
    fn update_big(lhs: &mut BigInt, rhs: Cow<BigInt>);

    #[inline]
    fn call<'a, L, R>(lhs: L, rhs: R) -> CBigInt
    where
        L: ToCow<'a, BigInt>,
        R: ToCow<'a, BigInt>,
    {
        Self::on_big(lhs.to_cow(), rhs.to_cow()).into()
    }

    #[inline]
    fn call_update<'a, R>(lhs: &mut CBigInt, rhs: R)
    where
        R: ToCow<'a, BigInt>,
    {
        lhs.0.update_encoding(|encoding| match encoding {
            Encoding::Small(small_lhs) => {
                *encoding = Encoding::Big(Self::on_big(
                    Cow::Owned(BigInt::from(*small_lhs)),
                    rhs.to_cow(),
                ));
            }
            Encoding::Big(big_lhs) => {
                Self::update_big(big_lhs, rhs.to_cow());
            }
        });
    }
}
trait ShiftOp {
    duplicate_prims! {
        paste! {
            fn [<on_big_ prim>](lhs: Cow<BigInt>, rhs: prim) -> BigInt;
            fn [<update_big_ prim>](lhs: &mut BigInt, rhs: prim);

            fn [<call_ prim>]<'a, L>(lhs: L, rhs: prim) -> CBigInt
            where
                L: ToCow<'a, BigInt>,
            {
                Self::[<on_big_ prim>](lhs.to_cow(), rhs).into()
            }

            #[inline]
            fn [<call_update_big_ prim>](lhs: &mut CBigInt, rhs: prim) {
                lhs.0.update_encoding(|encoding| match encoding {
                    Encoding::Small(small_lhs) => {
                        *encoding = Encoding::Big(Self::[<on_big_ prim>](
                            Cow::Owned(BigInt::from(*small_lhs)),
                            rhs,
                        ));
                    }
                    Encoding::Big(big_lhs) => {
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
        impl<'a, T> op_trait<T> for CBigInt
        where
            T: ToEncodingCow<'a, BigInt>,
        {
            type Output = CBigInt;

            fn op_fn(self, rhs: T) -> Self::Output {
                [< op_trait Op >]::call(self, rhs)
            }
        }

        impl<'a, T> op_trait<T> for &'a CBigInt
        where
            T: ToEncodingCow<'a, BigInt>,
        {
            type Output = CBigInt;

            fn op_fn(self, rhs: T) -> Self::Output {
                [< op_trait Op >]::call(self, rhs)
            }
        }

        impl<'a, T> [< op_trait Assign >]<T> for CBigInt
        where
            T: ToEncodingCow<'a, BigInt>
        {
            fn [< op_fn _assign >](&mut self, rhs: T) {
                [< op_trait Op >]::call_update(self, rhs);
            }
        }
    }

    crate::duplicate_prims! {
        paste! {
            impl op_trait<CBigInt> for prim {
                type Output = CBigInt;
                #[inline(never)]
                fn op_fn(self, rhs: CBigInt) -> CBigInt {
                    [< op_trait Op >]::call(self, rhs)
                }
            }

            impl op_trait<CBigInt> for &prim {
                type Output = CBigInt;
                fn op_fn(self, rhs: CBigInt) -> CBigInt {
                    (*self).op_fn(rhs)
                }
            }

            impl<'a> op_trait<&'a CBigInt> for prim {
                type Output = CBigInt;
                #[inline(never)]
                fn op_fn(self, rhs: &'a CBigInt) -> CBigInt {
                    [< op_trait Op >]::call(self, rhs)
                }
            }

            impl<'b> op_trait<&'b CBigInt> for &prim {
                type Output = CBigInt;
                fn op_fn(self, rhs: &'b CBigInt) -> CBigInt {
                    (*self).op_fn(rhs)
                }
            }
        }
    }
}

duplicate_bit_ops! {
    paste! {
        impl<'a, T> op_trait<T> for CBigInt
        where
            T: ToCow<'a, BigInt>,
        {
            type Output = CBigInt;

            fn op_fn(self, rhs: T) -> Self::Output {
                [< op_trait Op >]::call(self, rhs)
            }
        }

        impl<'a, T> op_trait<T> for &'a CBigInt
        where
            T: ToCow<'a, BigInt>,
        {
            type Output = CBigInt;

            fn op_fn(self, rhs: T) -> Self::Output {
                [< op_trait Op >]::call(self, rhs)
            }
        }

        impl<'a, T> [< op_trait Assign >]<T> for CBigInt
        where
            T: ToCow<'a, BigInt>
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
            impl op_trait<prim> for CBigInt {
                type Output = CBigInt;
                fn op_fn(self, rhs: prim) -> CBigInt {
                    [< op_trait Op >]::[<call_ prim>](self, rhs)
                }
            }

            impl<'a> op_trait<&'a prim> for CBigInt {
                type Output = CBigInt;
                fn op_fn(self, rhs: &'a prim) -> CBigInt {
                    self.op_fn(*rhs)
                }
            }

            impl op_trait<prim> for &CBigInt {
                type Output = CBigInt;
                fn op_fn(self, rhs: prim) -> CBigInt {
                    [< op_trait Op >]::[<call_ prim>](self, rhs)
                }
            }

            impl op_trait<&prim> for &CBigInt {
                type Output = CBigInt;
                fn op_fn(self, rhs: &prim) -> CBigInt {
                    self.op_fn(*rhs)
                }
            }

            impl [< op_trait Assign >]<prim> for CBigInt {
                fn [< op_fn _assign >](&mut self, rhs: prim) {
                    [< op_trait Op >]::[<call_update_big_ prim>](self, rhs);
                }
            }

            impl [< op_trait Assign >]<&prim> for CBigInt {
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
    impl Pow<prim> for CBigInt {
        type Output = CBigInt;

        fn pow(self, rhs: prim) -> Self::Output {
            BigInt::from(self).pow(rhs).into()
        }
    }

    impl Pow<&prim> for CBigInt {
        type Output = CBigInt;

        fn pow(self, rhs: &prim) -> Self::Output {
            BigInt::from(self).pow(rhs).into()
        }
    }

    impl Pow<prim> for &CBigInt {
        type Output = CBigInt;

        fn pow(self, rhs: prim) -> Self::Output {
            BigInt::from(self).pow(rhs).into()
        }
    }

    impl Pow<&prim> for &CBigInt {
        type Output = CBigInt;

        fn pow(self, rhs: &prim) -> Self::Output {
            BigInt::from(self).pow(rhs).into()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::duplicate_arith_and_bit_ops;
    use num_traits::{Pow, Zero};

    fn always(_lhs: &BigInt, _rhs: &BigInt) -> bool {
        true
    }

    fn nonzero_rhs(_lhs: &BigInt, rhs: &BigInt) -> bool {
        !rhs.is_zero()
    }

    fn make_range() -> Vec<BigInt> {
        let mut small_range: Vec<SmallInt> = vec![SmallInt::MIN, SmallInt::MAX, -SmallInt::MAX];
        small_range.extend(-10..=10);
        let mut range: Vec<BigInt> = small_range.into_iter().map(From::from).collect();
        let huge: BigInt = <BigInt as From<i128>>::from(i128::MAX).pow(2u32);
        range.push(huge.clone());
        range.push(-huge);
        range
    }

    struct ShiftOpsForType<R> {
        cbigint_op1: fn(CBigInt, R) -> CBigInt,
        cbigint_op2: fn(CBigInt, &R) -> CBigInt,
        cbigint_op3: fn(&CBigInt, R) -> CBigInt,
        cbigint_op4: fn(&CBigInt, &R) -> CBigInt,
        op_assign1: fn(&mut CBigInt, R),
        op_assign2: fn(&mut CBigInt, &R),
        bigint_op: fn(&BigInt, R) -> BigInt,
    }

    fn test_shift_op<R>(ops: ShiftOpsForType<R>)
    where
        R: TryFrom<u32>,
        R: Copy + Ord + Zero,
    {
        let range = make_range();

        for big_lhs in &range {
            for big_rhs in (0..128).chain((150..500).step_by(10)) {
                let lhs = CBigInt::from(big_lhs.clone());
                if let Ok(rhs) = R::try_from(big_rhs) {
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
                    let label = format!("failed with inputs {}, {}", big_lhs, big_rhs);
                    assert_eq!(expected, actual1, "{}", label);
                    assert_eq!(expected, actual2, "{}", label);
                    assert_eq!(expected, actual3, "{}", label);
                    assert_eq!(expected, actual4, "{}", label);
                    assert_eq!(expected, BigInt::from(actual5), "{}", label);
                    assert_eq!(expected, BigInt::from(actual6), "{}", label);
                }
            }
        }
    }

    struct BinOpsForTypes<L, R> {
        predicate: fn(&BigInt, &BigInt) -> bool,
        cbigint_op1: fn(L, R) -> CBigInt,
        cbigint_op2: fn(L, &R) -> CBigInt,
        cbigint_op3: fn(&L, R) -> CBigInt,
        cbigint_op4: fn(&L, &R) -> CBigInt,
        op_assign1: fn(&mut CBigInt, R),
        op_assign2: fn(&mut CBigInt, &R),
        bigint_op: fn(&BigInt, &BigInt) -> BigInt,
    }

    fn test_bin_op<L, R>(ops: BinOpsForTypes<L, R>)
    where
        L: TryFrom<BigInt> + Clone,
        R: TryFrom<BigInt> + Clone,
    {
        let range = make_range();

        for big_lhs in &range {
            for big_rhs in &range {
                if (ops.predicate)(big_lhs, big_rhs)
                    && let (Ok(lhs), Ok(rhs)) =
                        (L::try_from(big_lhs.clone()), R::try_from(big_rhs.clone()))
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
                }
            }
        }
    }

    duplicate_arith_and_bit_ops! {
        paste! {
            #[test]
            fn [< test_ op_fn >]() {
                test_bin_op::<CBigInt, CBigInt>(BinOpsForTypes {
                    predicate: op_test_pred,
                    cbigint_op1: |x, y| op_trait::op_fn(x, y),
                    cbigint_op2: |x, y| op_trait::op_fn(x, y),
                    cbigint_op3: |x, y| op_trait::op_fn(x, y),
                    cbigint_op4: |x, y| op_trait::op_fn(x, y),
                    op_assign1: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                    op_assign2: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                    bigint_op: |x, y| op_trait::op_fn(x, y),
                });
            }
        }
    }
    duplicate_shift_ops! {
         duplicate_prims! {
            paste! {
                #[test]
                fn [< test_ op_fn _ prim _rhs >]() {
                    test_shift_op::<prim>(ShiftOpsForType {
                        cbigint_op1: |x, y| op_trait::op_fn(x, y),
                        cbigint_op2: |x, y| op_trait::op_fn(x, y),
                        cbigint_op3: |x, y| op_trait::op_fn(x, y),
                        cbigint_op4: |x, y| op_trait::op_fn(x, y),
                        op_assign1: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                        op_assign2: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                        bigint_op: |x, y| op_trait::op_fn(x, y),
                    });
                }
            }
        }
    }
    duplicate_arith_ops! {
         duplicate_prims! {
            paste! {
                #[test]
                fn [< test_ op_fn _ prim _lhs >]() {
                    test_bin_op::<prim, CBigInt>(BinOpsForTypes {
                        predicate: op_test_pred,
                        cbigint_op1: |x, y| op_trait::op_fn(x, y),
                        cbigint_op2: |x, y| op_trait::op_fn(x, y),
                        cbigint_op3: |x, y| op_trait::op_fn(x, y),
                        cbigint_op4: |x, y| op_trait::op_fn(x, y),
                        op_assign1: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                        op_assign2: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                        bigint_op: |x, y| op_trait::op_fn(x, y),
                });
                }
                #[test]
                fn [< test_ op_fn _ prim _rhs >]() {
                    test_bin_op::<CBigInt, prim>(BinOpsForTypes {
                        predicate: op_test_pred,
                        cbigint_op1: |x, y| op_trait::op_fn(x, y),
                        cbigint_op2: |x, y| op_trait::op_fn(x, y),
                        cbigint_op3: |x, y| op_trait::op_fn(x, y),
                        cbigint_op4: |x, y| op_trait::op_fn(x, y),
                        op_assign1: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                        op_assign2: |x, y| [< op_trait Assign >]::[< op_fn _assign >](x, y),
                        bigint_op: |x, y| op_trait::op_fn(x, y),
                    });
                }
            }
        }
    }
}
