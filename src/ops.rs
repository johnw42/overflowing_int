use crate::cbigint::CBigInt;
use crate::checked;
use crate::decoded::Decoded;
use crate::to_cow::{ToCow, ToDecodedCow};
use crate::Digit;
use num_bigint::BigInt;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

struct BinaryOp {
    digits: fn(Digit, Digit) -> Option<Digit>,
    owned: fn(BigInt, BigInt) -> BigInt,
    owned_borrowed: fn(BigInt, &BigInt) -> BigInt,
    borrowed_owned: for<'a> fn(&'a BigInt, BigInt) -> BigInt,
    borrowed: for<'a> fn(&'a BigInt, &'a BigInt) -> BigInt,
    update_owned: for<'a> fn(&'a mut BigInt, BigInt),
    update_borrowed: fn(&mut BigInt, &BigInt),
}

impl BinaryOp {
    fn call<'a, L, R>(&self, lhs: L, rhs: R) -> CBigInt
    where
        L: ToDecodedCow<'a>,
        R: ToDecodedCow<'a>,
    {
        use Cow::*;
        let lhs = lhs.to_decoded_cow();
        let rhs = rhs.to_decoded_cow();

        if let (&Decoded::Digit(lhs), &Decoded::Digit(rhs)) = (&lhs, &rhs) {
            if let Some(out) = (self.digits)(lhs, rhs) {
                return out.into();
            }
        }

        match (lhs.to_cow(), rhs.to_cow()) {
            (Owned(lhs), Owned(rhs)) => (self.owned)(lhs, rhs),
            (Owned(lhs), Borrowed(rhs)) => (self.owned_borrowed)(lhs, rhs),
            (Borrowed(lhs), Owned(rhs)) => (self.borrowed_owned)(lhs, rhs),
            (Borrowed(lhs), Borrowed(rhs)) => (self.borrowed)(lhs, rhs),
        }
        .into()
    }

    fn call_update<'a, R>(&self, lhs: &mut CBigInt, rhs: R)
    where
        R: ToDecodedCow<'a>,
    {
        use Cow::*;
        let rhs = rhs.to_decoded_cow();

        if let (Decoded::Digit(lhs), &Decoded::Digit(rhs)) = (&mut lhs.decode_mut(), &rhs) {
            if let Some(out) = (self.digits)(*lhs, rhs) {
                *lhs = out;
                return;
            }
        }

        let target = lhs;
        let lhs = std::mem::take(target);
        *target = match (lhs.decode(), rhs.to_cow()) {
            (Decoded::Digit(digit), Owned(rhs)) => (self.owned)(BigInt::from(digit), rhs),
            (Decoded::Digit(digit), Borrowed(rhs)) => {
                (self.owned_borrowed)(BigInt::from(digit), rhs)
            }
            (Decoded::Big(mut big), Owned(rhs)) => {
                (self.update_owned)(&mut big, rhs);
                big
            }
            (Decoded::Big(mut big), Borrowed(rhs)) => {
                (self.update_borrowed)(&mut big, rhs);
                big
            }
        }
        .into();
    }

    fn call_update_prim<'a, R>(
        &self,
        lhs: &mut CBigInt,
        rhs: R,
        big_op: fn(BigInt, R) -> BigInt,
        big_assign_op: for<'b> fn(&'b mut BigInt, R),
    ) where
        R: Copy,
        Digit: TryFrom<R>,
    {
        if let Decoded::Digit(lhs) = &mut lhs.decode_mut() {
            if let Ok(rhs) = Digit::try_from(rhs) {
                if let Some(out) = (self.digits)(*lhs, rhs) {
                    *lhs = out;
                    return;
                }
            }
        }

        let target = lhs;
        let lhs = std::mem::take(target);
        *target = match lhs.decode() {
            Decoded::Digit(digit) => big_op(BigInt::from(digit), rhs),
            Decoded::Big(mut big) => {
                big_assign_op(&mut big, rhs);
                big
            }
        }
        .into();
    }

    fn call_prim_lhs<'a, L, R>(
        &self,
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
            Decoded::Digit(rhs) => {
                if let Ok(lhs) = Digit::try_from(lhs) {
                    if let Some(out) = (self.digits)(lhs, rhs) {
                        return out.into();
                    }
                }
                big_op(lhs, BigInt::from(rhs)).into()
            }
            Decoded::Big(big) => match big {
                Cow::Owned(big) => big_op(lhs, big),
                Cow::Borrowed(big) => big_ref_op(lhs, big),
            }
            .into(),
        }
    }

    fn call_prim_rhs<'a, L, R>(
        &self,
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
            Decoded::Digit(lhs) => {
                if let Ok(rhs) = Digit::try_from(rhs) {
                    if let Some(out) = (self.digits)(lhs, rhs) {
                        return out.into();
                    }
                }
                big_op(BigInt::from(lhs), rhs).into()
            }
            Decoded::Big(big) => match big {
                Cow::Owned(big) => big_op(big, rhs),
                Cow::Borrowed(big) => big_ref_op(big, rhs),
            }
            .into(),
        }
    }
}

struct ShiftOp(fn(Digit, u32) -> Option<Digit>);

impl ShiftOp {
    // Very similar to BinaryOp::call_prim_rhs.
    fn call<'a, L, R>(
        &self,
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
            Decoded::Digit(lhs) => {
                if let Ok(rhs) = u32::try_from(rhs) {
                    if let Some(out) = (self.0)(lhs, rhs) {
                        return out.into();
                    }
                }
                big_op(BigInt::from(lhs), rhs).into()
            }
            Decoded::Big(big) => match big {
                Cow::Owned(big) => big_op(big, rhs),
                Cow::Borrowed(big) => big_ref_op(big, rhs),
            }
            .into(),
        }
    }
}

macro_rules! bigint_op {
    [arith_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        pub(super) const $op: BinaryOp = BinaryOp {
            digits: |lhs, rhs| {
                if let Some(out) = checked::$op(lhs, rhs) {
                    Some(out)
                } else {
                    None
                }
            },
            owned: |lhs: BigInt, rhs: BigInt| $trait::$op(lhs, rhs),
            owned_borrowed: |lhs: BigInt, rhs: &BigInt| $trait::$op(lhs, rhs),
            borrowed_owned: |lhs: &BigInt, rhs: BigInt| $trait::$op(lhs, rhs),
            borrowed: |lhs: &BigInt, rhs: &BigInt| $trait::$op(lhs, rhs),
            update_owned: $assign_trait::$assign_op,
            update_borrowed: |lhs: &mut BigInt, rhs: &BigInt| $assign_trait::$assign_op(lhs, rhs),
        };
    };
    [bit_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        bigint_op![arith_op, [$trait, $op, $assign_trait, $assign_op]];
    };
    [shift_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        pub(super) const $op: ShiftOp = ShiftOp(|lhs: Digit, rhs: u32| checked::$op(lhs, rhs));
    };
}

#[allow(non_upper_case_globals)]
mod bigint_ops {
    use super::*;

    with_ops!(bigint_op, []);
}

macro_rules! bigint_op_traits {
    [arith_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        impl<'a, T> $trait<T> for CBigInt
        where
            T: ToDecodedCow<'a>,
        {
            type Output = CBigInt;
            fn $op(self, rhs: T) -> Self::Output {
                bigint_ops::$op.call(self, rhs)
            }
        }
        impl<'a, T> $trait<T> for &'a CBigInt
        where
            T: ToDecodedCow<'a>,
        {
            type Output = CBigInt;
            fn $op(self, rhs: T) -> Self::Output {
                bigint_ops::$op.call(self, rhs)
            }
        }
        impl<'a, T> $assign_trait<T> for CBigInt
        where
            T: ToDecodedCow<'a>
        {
            fn $assign_op(&mut self, rhs: T) {
                bigint_ops::$op.call_update(self, rhs);
            }
        }
    };
    [bit_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        // Bit ops work the same as other ops.
        bigint_op_traits!(arith_op, [$trait, $op, $assign_trait, $assign_op]);
    };
    [shift_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        // Can't shift a BigInt by a BigInt.
    };
}

macro_rules! prim_op_traits {
    [
        [int $(, $_1:tt)*], [$prim:ident, $to_prim:ident],
        arith_op, [
            $trait:ident,
            $op:ident,
            $assign_trait:ident,
            $assign_op:ident
        ]
    ] => {
        impl $trait<$prim> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: $prim) -> CBigInt {
                bigint_ops::$op.call_prim_rhs(self, rhs, $trait::$op, |x: &BigInt, y| x.$op(y))
            }
        }

        impl<'a> $trait<&'a $prim> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: &'a $prim) -> CBigInt {
                self.$op(*rhs)
            }
        }

        impl<'a> $trait<$prim> for &'a CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: $prim) -> CBigInt {
                bigint_ops::$op.call_prim_rhs(self, rhs, $trait::$op, |x: &BigInt, y| x.$op(y))
            }
        }

        impl<'a, 'b> $trait<&'a $prim> for &'b CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: &'a $prim) -> CBigInt {
                self.$op(*rhs)
            }
        }

        impl $trait<CBigInt> for $prim {
            type Output = CBigInt;
            fn $op(self, rhs: CBigInt) -> CBigInt {
                bigint_ops::$op.call_prim_lhs(self, rhs, $trait::$op, |x, y: &BigInt| x.$op(y))
            }
        }

        impl<'a> $trait<CBigInt> for &'a $prim {
            type Output = CBigInt;
            fn $op(self, rhs: CBigInt) -> CBigInt {
                (*self).$op(rhs)
            }
        }

        impl<'a> $trait<&'a CBigInt> for $prim {
            type Output = CBigInt;
            fn $op(self, rhs: &'a CBigInt) -> CBigInt {
                bigint_ops::$op.call_prim_lhs(self, rhs, $trait::$op, |x, y: &BigInt| x.$op(y))
            }
        }

        impl<'a, 'b> $trait<&'b CBigInt> for &'a $prim {
            type Output = CBigInt;
            fn $op(self, rhs: &'b CBigInt) -> CBigInt {
                (*self).$op(rhs)
            }
        }

        impl $assign_trait<$prim> for CBigInt {
            fn $assign_op(&mut self, rhs: $prim) {
                bigint_ops::$op.call_update_prim(self, rhs, BigInt::$op, BigInt::$assign_op);
            }
        }

        impl<'a> $assign_trait<&'a $prim> for CBigInt {
            fn $assign_op(&mut self, rhs: &'a $prim) {
                self.$assign_op(*rhs);
            }
        }
    };
    [
        [int $(, $_1:tt)*], [$prim:ident, $to_prim:ident],
        shift_op, [
            $trait:ident,
            $op:ident,
            $assign_trait:ident,
            $assign_op:ident
        ]
    ] => {
        impl $trait<$prim> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: $prim) -> CBigInt {
                bigint_ops::$op.call(self, rhs, BigInt::$op, |x: &BigInt, y| x.$op(y))
            }
        }

        impl<'a> $trait<&'a $prim> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: &'a $prim) -> CBigInt {
                self.$op(*rhs)
            }
        }

        impl<'a> $trait<$prim> for &'a CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: $prim) -> CBigInt {
                bigint_ops::$op.call(self, rhs, BigInt::$op, |x: &BigInt, y| x.$op(y))
            }
        }

        impl<'a, 'b> $trait<&'a $prim> for &'b CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: &'a $prim) -> CBigInt {
                self.$op(*rhs)
            }
        }

        impl $assign_trait<$prim> for CBigInt {
            fn $assign_op(&mut self, rhs: $prim) {
                match self.decode_mut() {
                    Decoded::Digit(_) => *self = self.clone().$op(rhs),
                    Decoded::Big(big) => big.$assign_op(rhs),
                }
            }
        }

        impl<'a> $assign_trait<&'a $prim> for CBigInt {
            fn $assign_op(&mut self, rhs: &'a $prim) {
                self.$assign_op(*rhs);
            }
        }
    };
    [
        [int $(, $int_attr:tt)*], [$prim:ident, $to_prim:ident],
        bit_op, [
            $trait:ident,
            $op:ident,
            $assign_trait:ident,
            $assign_op:ident
        ]
    ] => {
        // No bitwise operations on primitives.
    };
    [
        [float $(, $_1:tt)*], [$prim:ident, $to_prim:ident],
        $op:tt, $op_attrs:tt
    ] => {
        // For operations on float primitives.
    };
}

macro_rules! prim_op_traits_for_prim {
    [$($arg:tt),*] => {
        with_ops!(prim_op_traits, [$($arg),*]);
    };
}

with_prims!(prim_op_traits_for_prim, []);
with_ops!(bigint_op_traits, []);

#[cfg(test)]
mod test {
    use super::*;
    use foreach_macro::for_each;
    use num_traits::Zero;

    fn always(_lhs: &BigInt, _rhs: &BigInt) -> bool {
        true
    }

    fn nonzero_rhs(_lhs: &BigInt, rhs: &BigInt) -> bool {
        !rhs.is_zero()
    }

    fn test_bin_op<L, R>(
        predicate: fn(&BigInt, &BigInt) -> bool,
        cbigint_op1: fn(L, R) -> CBigInt,
        cbigint_op2: fn(L, &R) -> CBigInt,
        cbigint_op3: fn(&L, R) -> CBigInt,
        cbigint_op4: fn(&L, &R) -> CBigInt,
        bigint_op: fn(&BigInt, &BigInt) -> BigInt,
    ) where
        L: TryFrom<BigInt> + Clone,
        R: TryFrom<BigInt> + Clone,
    {
        let mut small_range: Vec<i128> = vec![Digit::MIN, Digit::MAX, -Digit::MAX];
        small_range.extend((-10..=10).into_iter());
        let mut range: Vec<BigInt> = small_range.into_iter().map(From::from).collect();
        let huge: BigInt = <BigInt as From<i128>>::from(i128::MAX).pow(2);
        range.push(huge.clone());
        range.push(-huge);

        for big_lhs in &range {
            for big_rhs in &range {
                if predicate(big_lhs, big_rhs) {
                    if let (Ok(ref lhs), Ok(ref rhs)) =
                        (L::try_from(big_lhs.clone()), R::try_from(big_rhs.clone()))
                    {
                        let expected = bigint_op(big_lhs, big_rhs);
                        let actual1 =
                            BigInt::from(cbigint_op1(L::from(lhs.clone()), R::from(rhs.clone())));
                        let actual2 =
                            BigInt::from(cbigint_op2(L::from(lhs.clone()), &R::from(rhs.clone())));
                        let actual3 =
                            BigInt::from(cbigint_op3(&L::from(lhs.clone()), R::from(rhs.clone())));
                        let actual4 =
                            BigInt::from(cbigint_op4(&L::from(lhs.clone()), &R::from(rhs.clone())));
                        assert_eq!(
                            expected, actual1,
                            "failed: f({}, {}) == {} (got {})",
                            big_lhs, big_rhs, expected, actual1
                        );
                        assert_eq!(
                            expected, actual2,
                            "failed: f({}, {}) == {} (got {})",
                            big_lhs, big_rhs, expected, actual2
                        );
                        assert_eq!(
                            expected, actual3,
                            "failed: f({}, {}) == {} (got {})",
                            big_lhs, big_rhs, expected, actual3
                        );
                        assert_eq!(
                            expected, actual4,
                            "failed: f({}, {}) == {} (got {})",
                            big_lhs, big_rhs, expected, actual4
                        );
                    }
                }
            }
        }
    }

    for_each!([$trait, $op, $pred] in [
        [Add, add, always]
        [Sub, sub, always]
        [Mul, mul, always]
        [Div, div, nonzero_rhs]
        [Rem, rem, nonzero_rhs]
        [BitAnd, bitand, always]
        [BitOr,  bitor,  always]
        [BitXor, bitxor, always]
    ] {
        #[test]
        fn $(test_ $op)() {
            test_bin_op::<CBigInt, CBigInt>(
                $pred,
                |x, y| $trait::$op(x, y),
                |x, y| $trait::$op(x, y),
                |x, y| $trait::$op(x, y),
                |x, y| $trait::$op(x, y),
                |x, y| $trait::$op(x, y),
            );
        }
    });
    for_each!([$trait, $op, $pred] in [
        [Add, add, always]
        [Sub, sub, always]
        [Mul, mul, always]
        [Div, div, nonzero_rhs]
        [Rem, rem, nonzero_rhs]
    ] {
        for_each!($other_type in [
            i8 i16 i32 i64 i128 isize
            u8 u16 u32 u64 u128 usize
        ] {
            #[test]
            fn $(test_ $op _ $other_type _lhs)() {
                test_bin_op::<$other_type, CBigInt>(
                    $pred,
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                );
            }
            #[test]
            fn $(test_ $op _ $other_type _rhs)() {
                test_bin_op::<CBigInt, $other_type>(
                    $pred,
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                );
            }
        });
    });
    //
    // #[test]
    // fn test_add() {
    //     test_bin_op(
    //         always,
    //         |x, y| Add::add(x, y),
    //         |x, y| Add::add(x, y),
    //         |x, y| Add::add(x, y),
    //         |x, y| Add::add(x, y),
    //         |x, y| Add::add(x, y),
    //     );
    // }
    //
    // #[test]
    // fn test_sub() {
    //     test_bin_op(
    //         always,
    //         |x, y| Sub::sub(x, y),
    //         |x, y| Sub::sub(x, y),
    //         |x, y| Sub::sub(x, y),
    //         |x, y| Sub::sub(x, y),
    //         |x, y| Sub::sub(x, y),
    //     );
    // }
    //
    // #[test]
    // fn test_mul() {
    //     test_bin_op(
    //         always,
    //         |x, y| Mul::mul(x, y),
    //         |x, y| Mul::mul(x, y),
    //         |x, y| Mul::mul(x, y),
    //         |x, y| Mul::mul(x, y),
    //         |x, y| Mul::mul(x, y),
    //     );
    // }
    //
    // #[test]
    // fn test_div() {
    //     test_bin_op(
    //         nonzero_rhs,
    //         |x, y| Div::div(x, y),
    //         |x, y| Div::div(x, y),
    //         |x, y| Div::div(x, y),
    //         |x, y| Div::div(x, y),
    //         |x, y| Div::div(x, y),
    //     );
    // }
    //
    // #[test]
    // fn test_rem() {
    //     test_bin_op(
    //         nonzero_rhs,
    //         |x, y| Rem::rem(x, y),
    //         |x, y| Rem::rem(x, y),
    //         |x, y| Rem::rem(x, y),
    //         |x, y| Rem::rem(x, y),
    //         |x, y| Rem::rem(x, y),
    //     );
    // }
    //
    // #[test]
    // fn test_bitand() {
    //     test_bin_op(
    //         always,
    //         |x, y| BitAnd::bitand(x, y),
    //         |x, y| BitAnd::bitand(x, y),
    //         |x, y| BitAnd::bitand(x, y),
    //         |x, y| BitAnd::bitand(x, y),
    //         |x, y| BitAnd::bitand(x, y),
    //     );
    // }
    //
    // #[test]
    // fn test_bitor() {
    //     test_bin_op(
    //         always,
    //         |x, y| BitOr::bitor(x, y),
    //         |x, y| BitOr::bitor(x, y),
    //         |x, y| BitOr::bitor(x, y),
    //         |x, y| BitOr::bitor(x, y),
    //         |x, y| BitOr::bitor(x, y),
    //     );
    // }
    //
    // #[test]
    // fn test_bitxor() {
    //     test_bin_op(
    //         always,
    //         |x, y| BitXor::bitxor(x, y),
    //         |x, y| BitXor::bitxor(x, y),
    //         |x, y| BitXor::bitxor(x, y),
    //         |x, y| BitXor::bitxor(x, y),
    //         |x, y| BitXor::bitxor(x, y),
    //     );
    // }
}
