use crate::Digit;
use crate::cbigint::CBigInt;
use crate::checked;
use crate::encoding::Encoded;
use crate::to_cow::{ToCow, ToDecodedCow};
use duplicate::duplicate;
use num_bigint::BigInt;
use paste::paste;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

trait BinaryOp {
    fn on_digits(lhs: Digit, rhs: Digit) -> Option<Digit>;
    fn on_owned(lhs: BigInt, rhs: BigInt) -> BigInt;
    fn on_owned_borrowed(lhs: BigInt, rhs: &BigInt) -> BigInt;
    fn on_borrowed_owned(lhs: &BigInt, rhs: BigInt) -> BigInt;
    fn on_borrowed(lhs: &BigInt, rhs: &BigInt) -> BigInt;
    fn update_owned(lhs: &mut BigInt, rhs: BigInt);
    fn update_borrowed(lhs: &mut BigInt, rhs: &BigInt);

    /// Calls a version of the binary operator that returns a new number.
    fn call<'a, L, R>(lhs: L, rhs: R) -> CBigInt
    where
        L: ToDecodedCow<'a>,
        R: ToDecodedCow<'a>,
    {
        use Cow::*;
        let lhs = lhs.to_decoded_cow();
        let rhs = rhs.to_decoded_cow();

        if let (&Encoded::Digit(lhs), &Encoded::Digit(rhs)) = (&lhs, &rhs)
            && let Some(out) = Self::on_digits(lhs, rhs)
        {
            return out.into();
        }

        match (lhs.to_cow(), rhs.to_cow()) {
            (Owned(lhs), Owned(rhs)) => Self::on_owned(lhs, rhs),
            (Owned(lhs), Borrowed(rhs)) => Self::on_owned_borrowed(lhs, rhs),
            (Borrowed(lhs), Owned(rhs)) => Self::on_borrowed_owned(lhs, rhs),
            (Borrowed(lhs), Borrowed(rhs)) => Self::on_borrowed(lhs, rhs),
        }
        .into()
    }

    /// Calls a version of the binary operator that updates a bigint argument in place.
    fn call_update<'a, R>(lhs: &mut CBigInt, rhs: R)
    where
        R: ToDecodedCow<'a>,
    {
        use Cow::*;
        let rhs = rhs.to_decoded_cow();

        if let (&Encoded::Digit(lhs_digit), &Encoded::Digit(rhs)) = (&lhs.0, &rhs)
            && let Some(out) = Self::on_digits(lhs_digit, rhs)
        {
            *lhs = out.into();
            return;
        }

        let target = lhs;
        let lhs = std::mem::take(target);
        *target = match (lhs.0, rhs.to_cow()) {
            (Encoded::Digit(digit), Owned(rhs)) => Self::on_owned(BigInt::from(digit), rhs),
            (Encoded::Digit(digit), Borrowed(rhs)) => {
                Self::on_owned_borrowed(BigInt::from(digit), rhs)
            }
            (Encoded::Big(mut big), Owned(rhs)) => {
                Self::update_owned(&mut big, rhs);
                big
            }
            (Encoded::Big(mut big), Borrowed(rhs)) => {
                Self::update_borrowed(&mut big, rhs);
                big
            }
        }
        .into();
    }

    /// Calls a version of the binary operator that updates a primitive in place.
    fn call_update_prim<R>(
        lhs: &mut CBigInt,
        rhs: R,
        big_op: fn(BigInt, R) -> BigInt,
        big_assign_op: for<'b> fn(&'b mut BigInt, R),
    ) where
        R: Copy,
        Digit: TryFrom<R>,
    {
        if let Encoded::Digit(lhs_digit) = lhs.0
            && let Ok(rhs) = Digit::try_from(rhs)
            && let Some(out) = Self::on_digits(lhs_digit, rhs)
        {
            *lhs = out.into();
            return;
        }

        let target = lhs;
        let lhs = std::mem::take(target);
        *target = match lhs.0 {
            Encoded::Digit(digit) => big_op(BigInt::from(digit), rhs),
            Encoded::Big(mut big) => {
                big_assign_op(&mut big, rhs);
                big
            }
        }
        .into();
    }

    fn call_prim_lhs<'a, L, R>(
        lhs: L,
        rhs: R,
        big_op: fn(L, BigInt) -> BigInt,
        big_ref_op: fn(L, &'a BigInt) -> BigInt,
    ) -> CBigInt
    where
        R: ToDecodedCow<'a>,
        L: Copy,
        Digit: TryFrom<L>,
    {
        match rhs.to_decoded_cow() {
            Encoded::Digit(rhs) => {
                if let Ok(lhs) = Digit::try_from(lhs)
                    && let Some(out) = Self::on_digits(lhs, rhs)
                {
                    return out.into();
                }
                big_op(lhs, BigInt::from(rhs)).into()
            }
            Encoded::Big(big) => match big {
                Cow::Owned(big) => big_op(lhs, big),
                Cow::Borrowed(big) => big_ref_op(lhs, big),
            }
            .into(),
        }
    }

    fn call_prim_rhs<'a, L, R>(
        lhs: L,
        rhs: R,
        big_op: fn(BigInt, R) -> BigInt,
        big_ref_op: fn(&'a BigInt, R) -> BigInt,
    ) -> CBigInt
    where
        L: ToDecodedCow<'a>,
        R: Copy,
        Digit: TryFrom<R>,
    {
        match lhs.to_decoded_cow() {
            Encoded::Digit(lhs) => {
                if let Ok(rhs) = Digit::try_from(rhs)
                    && let Some(out) = Self::on_digits(lhs, rhs)
                {
                    return out.into();
                }
                big_op(BigInt::from(lhs), rhs).into()
            }
            Encoded::Big(big) => match big {
                Cow::Owned(big) => big_op(big, rhs),
                Cow::Borrowed(big) => big_ref_op(big, rhs),
            }
            .into(),
        }
    }
}

trait ShiftOp {
    fn on_digit(lhs: Digit, rhs: u32) -> Option<Digit>;

    // Very similar to BinaryOp::call_prim_rhs.
    fn call<'a, L, R>(
        lhs: L,
        rhs: R,
        big_op: fn(BigInt, R) -> BigInt,
        big_ref_op: fn(&'a BigInt, R) -> BigInt,
    ) -> CBigInt
    where
        L: ToDecodedCow<'a>,
        R: Copy,
        u32: TryFrom<R>,
    {
        match lhs.to_decoded_cow() {
            Encoded::Digit(lhs) => {
                if let Ok(rhs) = u32::try_from(rhs)
                    && let Some(out) = Self::on_digit(lhs, rhs)
                {
                    return out.into();
                }
                big_op(BigInt::from(lhs), rhs).into()
            }
            Encoded::Big(big) => match big {
                Cow::Owned(big) => big_op(big, rhs),
                Cow::Borrowed(big) => big_ref_op(big, rhs),
            }
            .into(),
        }
    }
}

macro_rules! duplicate_arith_ops {
    ($($body:tt)*) => {
        duplicate! {
            [
                op_type op_trait op_fn op_test_pred;
                [arith] [Add] [add] [always];
                [arith] [Sub] [sub] [always];
                [arith] [Mul] [mul] [always];
                [arith] [Div] [div] [nonzero_rhs];
                [arith] [Rem] [rem] [nonzero_rhs];
            ]
            $($body)*
        }
    }
}

macro_rules! duplicate_shift_ops {
    ($($body:tt)*) => {
        duplicate! {
            [
                op_type op_trait op_fn op_test_pred;
                [shift] [Shl] [shl] [always];
                [shift] [Shr] [shr] [always];
            ]
            $($body)*
        }
    }
}

macro_rules! duplicate_bit_ops {
    ($($body:tt)*) => {
        duplicate! {
            [
                op_type op_trait op_fn op_test_pred;
                [bit] [BitAnd] [bitand] [always];
                [bit] [BitOr] [bitor] [always];
                [bit] [BitXor] [bitxor] [always]
            ]
            $($body)*
        }
    }
}

macro_rules! duplicate_arith_and_bit_ops {
    ($($body:tt)*) => {
        duplicate_arith_ops! { $($body)* }
        duplicate_bit_ops! { $($body)* }
    }
}

// macro_rules! duplicate_ops {
//     ($($body:tt)*) => {
//         duplicate_arith_ops! { $($body)* }
//         duplicate_shift_ops! { $($body)* }
//         duplicate_bit_ops! { $($body)* }
//     }
// }

macro_rules! duplicate_prims {
    ($($body:tt)*) => {
        duplicate! {
            [
                prim;
                [i8];
                [i16];
                [i32];
                [i64];
                [i128];
                [isize];
                [u8];
                [u16];
                [u32];
                [u64];
                [u128];
                [usize];
            ]
            $($body)*
        }
    }
}

duplicate_arith_and_bit_ops! {
    paste! {
        struct [< op_trait Op >];

        impl BinaryOp for [< op_trait Op >] {
            fn on_digits(lhs: Digit, rhs: Digit) -> Option<Digit> {
                checked::op_fn(lhs, rhs)
            }

            fn on_owned(lhs: BigInt, rhs: BigInt) -> BigInt {
                op_trait::op_fn(lhs, rhs)
            }

            fn on_owned_borrowed(lhs: BigInt, rhs: &BigInt) -> BigInt {
                op_trait::op_fn(lhs, rhs)
            }

            fn on_borrowed_owned(lhs: &BigInt, rhs: BigInt) -> BigInt {
                op_trait::op_fn(lhs, rhs)
            }

            fn on_borrowed(lhs: &BigInt, rhs: &BigInt) -> BigInt {
                op_trait::op_fn(lhs, rhs)
            }

            fn update_owned(lhs: &mut BigInt, rhs: BigInt) {
                [<op_trait Assign>]::[< op_fn _assign >](lhs, rhs)
            }

            fn update_borrowed(lhs: &mut BigInt, rhs: &BigInt) {
                [<op_trait Assign>]::[< op_fn _assign >](lhs, rhs)
            }
        }
    }
}

duplicate_shift_ops! {
    paste! {
        struct [< op_trait Op >];

        impl ShiftOp for [< op_trait Op >] {
            fn on_digit(lhs: Digit, rhs: u32) -> Option<Digit> { checked::op_fn(lhs, rhs) }
        }
    }
}

macro_rules! impl_arith_or_bit_ops {
    ($op_fn: ident, $op_trait:ident) => {
        paste! {
            impl<'a, T> $op_trait<T> for CBigInt
            where
                T: ToDecodedCow<'a>,
            {
                type Output = CBigInt;

                fn $op_fn(self, rhs: T) -> Self::Output {
                    [< $op_trait Op >]::call(self, rhs)
                }
            }

            impl<'a, T> $op_trait<T> for &'a CBigInt
            where
                T: ToDecodedCow<'a>,
            {
                type Output = CBigInt;

                fn $op_fn(self, rhs: T) -> Self::Output {
                    [< $op_trait Op >]::call(self, rhs)
                }
            }

            impl<'a, T> [< $op_trait Assign >]<T> for CBigInt
            where
                T: ToDecodedCow<'a>
            {
                fn [< $op_fn _assign >](&mut self, rhs: T) {
                    [< $op_trait Op >]::call_update(self, rhs);
                }
            }
        }
    };
}

duplicate_arith_ops! {
    impl_arith_or_bit_ops!(op_fn, op_trait);
    duplicate_prims! {
        paste! {
            impl op_trait<prim> for CBigInt {
                type Output = CBigInt;
                #[inline(never)]
                fn op_fn(self, rhs: prim) -> CBigInt {
                    [< op_trait Op >]::call_prim_rhs(self, rhs, op_trait::op_fn, |x: &BigInt, y| x.op_fn(y))
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
                #[inline(never)]
                fn op_fn(self, rhs: prim) -> CBigInt {
                    [< op_trait Op >]::call_prim_rhs(self, rhs, op_trait::op_fn, |x: &BigInt, y| x.op_fn(y))
                }
            }

            impl<'a> op_trait<&'a prim> for &CBigInt {
                type Output = CBigInt;
                fn op_fn(self, rhs: &'a prim) -> CBigInt {
                    self.op_fn(*rhs)
                }
            }

            impl op_trait<CBigInt> for prim {
                type Output = CBigInt;
                #[inline(never)]
                fn op_fn(self, rhs: CBigInt) -> CBigInt {
                    [< op_trait Op >]::call_prim_lhs(self, rhs, op_trait::op_fn, |x, y: &BigInt| x.op_fn(y))
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
                    [< op_trait Op >]::call_prim_lhs(self, rhs, op_trait::op_fn, |x, y: &BigInt| x.op_fn(y))
                }
            }

            impl<'b> op_trait<&'b CBigInt> for &prim {
                type Output = CBigInt;
                fn op_fn(self, rhs: &'b CBigInt) -> CBigInt {
                    (*self).op_fn(rhs)
                }
            }
            impl [< op_trait Assign >]<prim> for CBigInt {
                #[inline(never)]
                fn [< op_fn _assign >](&mut self, rhs: prim) {
                    [< op_trait Op >]::call_update_prim(self, rhs, BigInt::op_fn, BigInt::[< op_fn _assign >]);
                }
            }

            impl<'a> [< op_trait Assign >]<&'a prim> for CBigInt {
                fn [< op_fn _assign >](&mut self, rhs: &'a prim) {
                    self.[< op_fn _assign >](*rhs);
                }
            }
        }
    }
}

duplicate_bit_ops! {
    impl_arith_or_bit_ops!(op_fn, op_trait);
}

duplicate_shift_ops! {
    duplicate_prims! {
        paste! {
            impl op_trait<prim> for CBigInt {
                type Output = CBigInt;
                fn op_fn(self, rhs: prim) -> CBigInt {
                    [< op_trait Op >]::call(self, rhs, BigInt::op_fn, |x: &BigInt, y| x.op_fn(y))
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
                    [< op_trait Op >]::call(self, rhs, BigInt::op_fn, |x: &BigInt, y| x.op_fn(y))
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
                    match &mut self.0 {
                        Encoded::Digit(_) => *self = self.clone().op_fn(rhs),
                        Encoded::Big(big) => big.[< op_fn _assign >](rhs),
                    }
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

#[cfg(test)]
mod test {
    use super::*;
    use num_traits::Zero;

    fn always(_lhs: &BigInt, _rhs: &BigInt) -> bool {
        true
    }

    fn nonzero_rhs(_lhs: &BigInt, rhs: &BigInt) -> bool {
        !rhs.is_zero()
    }

    fn make_range() -> Vec<BigInt> {
        let mut small_range: Vec<Digit> = vec![Digit::MIN, Digit::MAX, -Digit::MAX];
        small_range.extend(-10..=10);
        let mut range: Vec<BigInt> = small_range.into_iter().map(From::from).collect();
        let huge: BigInt = <BigInt as From<i128>>::from(i128::MAX).pow(2);
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
        R: Copy,
    {
        let range = make_range();

        for big_lhs in &range {
            for big_rhs in (0..128).chain((150..500).step_by(10)) {
                let lhs = CBigInt::from(big_lhs.clone());
                if let Ok(rhs) = R::try_from(big_rhs) {
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

    duplicate_ops! {
        match_ident! {
            op_type;
            arith | bit => {
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
            _ => {}
        }
        duplicate_prims! {
            paste! {
                match_ident! {
                    op_type;
                    shift => {
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
                    arith => {
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
                    _ => {}
                }
            }
        }
    }
}
