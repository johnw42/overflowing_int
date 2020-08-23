use crate::cbigint::CBigInt;
use crate::checked;
use crate::decoded::Decoded;
use crate::to_cow::{ToCow, ToDecodedCow};
use crate::Digit;
use expand_macro::expand;
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

        if let (Decoded::Digit(lhs_digit), &Decoded::Digit(rhs)) = (lhs.decode_mut(), &rhs) {
            if let Some(out) = (self.digits)(lhs_digit, rhs) {
                *lhs = out.into();
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
        if let Decoded::Digit(lhs_digit) = lhs.decode_mut() {
            if let Ok(rhs) = Digit::try_from(rhs) {
                if let Some(out) = (self.digits)(lhs_digit, rhs) {
                    *lhs = out.into();
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

expand! {
    let $ops = [
        [arith Add add]
        [arith Sub sub]
        [arith Mul mul]
        [arith Div div]
        [arith Rem rem]
        [shift Shl shl]
        [shift Shr shr]
        [bit BitAnd bitand]
        [bit BitOr bitor]
        [bit BitXor bitxor]
    ]
    =>

    #[allow(non_upper_case_globals)]
    mod bigint_ops {
        use super::*;

        expand! {
            for [$op_type $trait $op] in $ops
            let $assign_trait = ${$trait Assign}
            let $assign_op = ${$op _assign}
            =>
            expand! {
                if let arith | bit = $op_type
                =>
                pub(super) const $op: BinaryOp = BinaryOp {
                    digits: |lhs, rhs| {
                        if let Some(out) = checked::$op(lhs, rhs) {
                            Some(out)
                        } else {
                            None
                        }
                    },
                    owned: |lhs, rhs| $trait::$op(lhs, rhs),
                    owned_borrowed: |lhs, rhs| $trait::$op(lhs, rhs),
                    borrowed_owned: |lhs, rhs| $trait::$op(lhs, rhs),
                    borrowed: |lhs, rhs| $trait::$op(lhs, rhs),
                    update_owned: |lhs, rhs| $assign_trait::$assign_op(lhs, rhs),
                    update_borrowed: |lhs, rhs| $assign_trait::$assign_op(lhs, rhs),
                };
            }
            expand! {
                if let shift = $op_type
                =>
                pub(super) const $op: ShiftOp = ShiftOp(|lhs: Digit, rhs: u32| checked::$op(lhs, rhs));
            }
        }
    }

    expand! {
        for [$op_type $trait $op] in $ops
        let $assign_trait = ${$trait Assign}
        let $assign_op = ${$op _assign}
        =>
        expand! {
            if let arith | bit = $op_type
            =>
            expand! {
                =>
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
            }
        }
        expand! {
            for [$prim_type $prim] in [
                [signed i8]
                [signed i16]
                [signed i32]
                [signed i64]
                [signed i128]
                [signed isize]
                [unsigned u8]
                [unsigned u16]
                [unsigned u32]
                [unsigned u64]
                [unsigned u128]
                [unsigned usize]
            ]
            =>
            expand! {
                if let arith = $op_type
                =>
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
            }
            expand! {
                if let shift = $op_type
                =>
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
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use expand_macro::expand;
    use num_traits::Zero;

    fn always(_lhs: &BigInt, _rhs: &BigInt) -> bool {
        true
    }

    fn nonzero_rhs(_lhs: &BigInt, rhs: &BigInt) -> bool {
        !rhs.is_zero()
    }

    fn make_range() -> Vec<BigInt> {
        let mut small_range: Vec<Digit> = vec![Digit::MIN, Digit::MAX, -Digit::MAX];
        small_range.extend((-10..=10).into_iter());
        let mut range: Vec<BigInt> = small_range.into_iter().map(From::from).collect();
        let huge: BigInt = <BigInt as From<i128>>::from(i128::MAX).pow(2);
        range.push(huge.clone());
        range.push(-huge);
        range
    }

    fn test_shift_op<R>(
        cbigint_op1: fn(CBigInt, R) -> CBigInt,
        cbigint_op2: fn(CBigInt, &R) -> CBigInt,
        cbigint_op3: fn(&CBigInt, R) -> CBigInt,
        cbigint_op4: fn(&CBigInt, &R) -> CBigInt,
        op_assign1: fn(&mut CBigInt, R),
        op_assign2: fn(&mut CBigInt, &R),
        bigint_op: fn(&BigInt, R) -> BigInt,
    ) where
        R: TryFrom<u32>,
        R: Copy,
    {
        let range = make_range();

        for big_lhs in &range {
            for big_rhs in (0..128).chain((150..500).step_by(10)) {
                let lhs = CBigInt::from(big_lhs.clone());
                if let Ok(rhs) = R::try_from(big_rhs) {
                    let expected = bigint_op(big_lhs, rhs);
                    let actual1 = BigInt::from(cbigint_op1(lhs.clone(), rhs));
                    let actual2 = BigInt::from(cbigint_op2(lhs.clone(), &rhs));
                    let actual3 = BigInt::from(cbigint_op3(&lhs, rhs));
                    let actual4 = BigInt::from(cbigint_op4(&lhs, &rhs));
                    let mut actual5 = lhs.clone();
                    op_assign1(&mut actual5, rhs);
                    let mut actual6 = lhs.clone();
                    op_assign2(&mut actual6, &rhs);
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

    fn test_bin_op<L, R>(
        predicate: fn(&BigInt, &BigInt) -> bool,
        cbigint_op1: fn(L, R) -> CBigInt,
        cbigint_op2: fn(L, &R) -> CBigInt,
        cbigint_op3: fn(&L, R) -> CBigInt,
        cbigint_op4: fn(&L, &R) -> CBigInt,
        op_assign1: fn(&mut CBigInt, R),
        op_assign2: fn(&mut CBigInt, &R),
        bigint_op: fn(&BigInt, &BigInt) -> BigInt,
    ) where
        L: TryFrom<BigInt> + Clone,
        R: TryFrom<BigInt> + Clone,
    {
        let range = make_range();

        for big_lhs in &range {
            for big_rhs in &range {
                if predicate(big_lhs, big_rhs) {
                    if let (Ok(lhs), Ok(rhs)) =
                        (L::try_from(big_lhs.clone()), R::try_from(big_rhs.clone()))
                    {
                        let expected = bigint_op(big_lhs, big_rhs);
                        let actual1 = BigInt::from(cbigint_op1(lhs.clone(), rhs.clone()));
                        let actual2 = BigInt::from(cbigint_op2(lhs.clone(), &rhs));
                        let actual3 = BigInt::from(cbigint_op3(&lhs, rhs.clone()));
                        let actual4 = BigInt::from(cbigint_op4(&lhs, &rhs));
                        let mut actual5 = big_lhs.clone().into();
                        op_assign1(&mut actual5, rhs.clone());
                        let mut actual6 = big_lhs.clone().into();
                        op_assign2(&mut actual6, &rhs);
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
    }

    expand! {
        for [$trait, $op, $pred] in [
            [Add, add, always]
            [Sub, sub, always]
            [Mul, mul, always]
            [Div, div, nonzero_rhs]
            [Rem, rem, nonzero_rhs]
            [BitAnd, bitand, always]
            [BitOr,  bitor,  always]
            [BitXor, bitxor, always]
        ] =>
        #[test]
        fn ${test_ $op}() {
            test_bin_op::<CBigInt, CBigInt>(
                $pred,
                |x, y| $trait::$op(x, y),
                |x, y| $trait::$op(x, y),
                |x, y| $trait::$op(x, y),
                |x, y| $trait::$op(x, y),
                |x, y| ${$trait Assign}::${$op _assign}(x, y),
                |x, y| ${$trait Assign}::${$op _assign}(x, y),
                |x, y| $trait::$op(x, y),
            );
        }
    }
    expand! {
        for $other_type in [
            i8 i16 i32 i64 i128 isize
            u8 u16 u32 u64 u128 usize
        ] =>
        expand! {
            for [$trait, $op] in [
                [Shl, shl]
                [Shr, shr]
            ] =>
            #[test]
            fn ${test_ $op _ $other_type _rhs}() {
                test_shift_op::<$other_type>(
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| ${$trait Assign}::${$op _assign}(x, y),
                    |x, y| ${$trait Assign}::${$op _assign}(x, y),
                    |x, y| $trait::$op(x, y),
                );
            }
        }
        expand! {
            for [$trait, $op, $pred] in [
                [Add, add, always]
                [Sub, sub, always]
                [Mul, mul, always]
                [Div, div, nonzero_rhs]
                [Rem, rem, nonzero_rhs]
            ] =>
            #[test]
            fn ${test_ $op _ $other_type _lhs}() {
                test_bin_op::<$other_type, CBigInt>(
                    $pred,
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| ${$trait Assign}::${$op _assign}(x, y),
                    |x, y| ${$trait Assign}::${$op _assign}(x, y),
                    |x, y| $trait::$op(x, y),
                );
            }
            #[test]
            fn ${test_ $op _ $other_type _rhs}() {
                test_bin_op::<CBigInt, $other_type>(
                    $pred,
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| $trait::$op(x, y),
                    |x, y| ${$trait Assign}::${$op _assign}(x, y),
                    |x, y| ${$trait Assign}::${$op _assign}(x, y),
                    |x, y| $trait::$op(x, y),
                );
            }
        }
    }
}
