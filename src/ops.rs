use crate::cbigint::CBigInt;
use crate::decoded::{Decoded, DecodedMut};
use crate::overflowing::Overflowing;
use crate::to_cow::{ToCow, ToDecodedCow};
use crate::Digit;
use num_bigint::BigInt;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, DerefMut, Div,
    DivAssign, Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
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

        match (lhs.decode_mut(), rhs.to_cow()) {
            (Decoded::Digit(_), Owned(rhs)) => {
                *lhs = (self.owned)(BigInt::from(std::mem::take(lhs)), rhs).into();
            }
            (Decoded::Digit(_), Borrowed(rhs)) => {
                *lhs = (self.owned_borrowed)(BigInt::from(std::mem::take(lhs)), rhs).into();
            }
            (Decoded::Big(big), Owned(rhs)) => {
                (self.update_owned)(big, rhs);
                *lhs = std::mem::take(big).into()
            }
            (Decoded::Big(big), Borrowed(rhs)) => {
                (self.update_borrowed)(big, rhs);
                *lhs = std::mem::take(big).into()
            }
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

    // fn call_prim_lhs<L>(&self, lhs: L, rhs: &CBigInt) -> Option<CBigInt>
    // where
    //     Digit: TryFrom<L>,
    // {
    //     if let Some(rhs) = rhs.to_digit() {
    //         if let Ok(lhs) = Digit::try_from(lhs) {
    //             if let Some(out) = (self.digits)(lhs, rhs) {
    //                 return Some(out.into());
    //             }
    //         }
    //     }
    //     None
    // }
}

struct ShiftOp(fn(Digit, u32) -> (Digit, bool));

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
                    if let (out, false) = (self.0)(lhs, rhs) {
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
                if let (out, false) = Overflowing::$op(lhs, rhs) {
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
        pub(super) const $op: BinaryOp = BinaryOp {
            digits: |lhs, rhs| Some($trait::$op(lhs, rhs)),
            owned: |lhs: BigInt, rhs: BigInt| $trait::$op(lhs, rhs),
            owned_borrowed: |lhs: BigInt, rhs: &BigInt| $trait::$op(lhs, rhs),
            borrowed_owned: |lhs: &BigInt, rhs: BigInt| $trait::$op(lhs, rhs),
            borrowed: |lhs: &BigInt, rhs: &BigInt| $trait::$op(lhs, rhs),
            update_owned: $assign_trait::$assign_op,
            update_borrowed: |lhs: &mut BigInt, rhs: &BigInt| $assign_trait::$assign_op(lhs, rhs),
        };
    };
    [shift_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        pub(super) const $op: ShiftOp = ShiftOp(|lhs: Digit, rhs: u32| Overflowing::$op(lhs, rhs));
    };
}

#[allow(non_upper_case_globals)]
mod bigint_ops {
    use super::*;

    with_ops!(bigint_op, []);
}

macro_rules! op_traits {
    [arith_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        impl $trait<CBigInt> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: CBigInt) -> Self::Output {
                bigint_ops::$op.call(self, rhs)
            }
        }
        impl $trait<CBigInt> for &CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: CBigInt) -> Self::Output {
                bigint_ops::$op.call(self, rhs)
            }
        }
        impl $trait<&CBigInt> for CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: &CBigInt) -> Self::Output {
                bigint_ops::$op.call(self, rhs)
            }
        }
        impl $trait<&CBigInt> for &CBigInt {
            type Output = CBigInt;
            fn $op(self, rhs: &CBigInt) -> Self::Output {
                bigint_ops::$op.call(self, rhs)
            }
        }
        assign_op!($trait, $op, $assign_trait, $assign_op);
    };
    [bit_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        // Bit ops work the same as other ops.
        op_traits!(arith_op, [$trait, $op, $assign_trait, $assign_op]);
    };
    [shift_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        // Can't shift a BigInt by a BigInt.
    };
}

macro_rules! assign_op {
    [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident] => {
        // impl<T> $assign_trait<T> for CBigInt
        // where
        //     CBigInt: $trait<T, Output = CBigInt>,
        //     BigInt: From<T>,
        //     BigInt: $assign_trait,
        // {
        //     fn $assign_op(&mut self, rhs: T) {
        //         match self.decode_mut() {
        //             Decoded::Digit(_) => {
        //                 let lhs = std::mem::take(self);
        //                 *self = lhs.$op(rhs);
        //             }
        //             Decoded::Big(big) => {
        //                 big.$assign_op(BigInt::from(rhs));
        //             }
        //         }
        //     }
        // }

        // impl<'a, T> $assign_trait<T> for CBigInt
        // where
        //     CBigInt: $trait<T, Output = CBigInt>,
        //     T: ToDecodedCow<'a>,
        //     BigInt: $assign_trait,
        // {
        //     fn $assign_op(&mut self, rhs: T) {
        //         match self.decode_mut() {
        //             Decoded::Digit(_) => {
        //                 let lhs = std::mem::take(self);
        //                 *self = lhs.$op(rhs);
        //             }
        //             Decoded::Big(big) => {
        //                 big.$assign_op(BigInt::from(rhs));
        //             }
        //         }
        //     }
        // }
    };
}

macro_rules! prim_op {
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
                bigint_ops::$op.call_prim_rhs(self, rhs, BigInt::$op, |x: &BigInt, y| x.$op(y))
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
                bigint_ops::$op.call_prim_rhs(self, rhs, BigInt::$op, |x: &BigInt, y| x.$op(y))
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

        // impl $trait<$prim> for CBigInt {
        //     type Output = CBigInt;
        //     fn $op(self, rhs: $prim) -> Self::Output {
        //         if let Small(prim) = &self {
        //             if let Ok(promoted) = Digit::try_from(rhs) {
        //                 if let (result, false) = Overflowing::$op(prim, promoted) {
        //                     return result.into();
        //                 }
        //             }
        //         }
        //         BigInt::from(self).$op(rhs).into()
        //     }
        // }
        // impl $trait<CBigInt> for $prim {
        //     type Output = CBigInt;
        //     fn $op(self, rhs: CBigInt) -> Self::Output {
        //         if let Small(prim) = &rhs {
        //             if let Ok(promoted) = Digit::try_from(self) {
        //                 if let (result, false) = Overflowing::$op(promoted, *prim) {
        //                     return result.into();
        //                 }
        //             }
        //         }
        //         self.$op(BigInt::from(rhs)).into()
        //     }
        // }
        // ref_op!($trait<$prim> for CBigInt, $op);
        // ref_op!($trait<CBigInt> for $prim, $op);
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
        [int $(, $_1:tt)*], [$prim:ident, $to_prim:ident],
        bit_op, [
            $trait:ident,
            $op:ident,
            $assign_trait:ident,
            $assign_op:ident
        ]
    ] => {
    };
    [
        [float $(, $_1:tt)*], [$prim:ident, $to_prim:ident],
        $op:tt, $op_attrs:tt
    ] => {};
}

// impl Shl<i8> for CBigInt {
//     type Output = CBigInt;
//     fn shl(self, rhs: i8) -> CBigInt {
//         bigint_ops::shl.call(&self, rhs, BigInt::shl, |x, y| x.shl(y))
//     }
// }

macro_rules! prim_ops {
    [$($arg:tt),*] => {
        with_ops!(prim_op, [$($arg),*]);
    };
}

with_prims!(prim_ops, []);
with_ops!(op_traits, []);

#[test]
fn test() {
    use num_traits::Zero;

    let bin_ops: &[(
        &str,
        fn(CBigInt, CBigInt) -> CBigInt,
        fn(BigInt, BigInt) -> BigInt,
    )] = &[
        ("+", CBigInt::add, BigInt::add),
        ("-", CBigInt::sub, BigInt::sub),
        ("*", CBigInt::mul, BigInt::mul),
        ("/", CBigInt::div, BigInt::div),
        ("%", CBigInt::rem, BigInt::rem),
    ];
    let mut small_range = vec![Digit::MIN, Digit::MAX, -Digit::MAX];
    small_range.extend((-10..=10).into_iter());
    let mut range: Vec<_> = small_range.into_iter().map(BigInt::from).collect();
    range.push(BigInt::from(i128::MAX) * 2);
    range.push(BigInt::from(i128::MIN) * 2);

    for (op_name, cop, op) in bin_ops {
        for a in &range {
            for b in &range {
                if !b.is_zero() {
                    let expected = op(a.clone(), b.clone());
                    let actual =
                        BigInt::from(cop(CBigInt::from(a.clone()), CBigInt::from(b.clone())));
                    assert_eq!(
                        expected, actual,
                        "failed: {} {} {} == {} (got {})",
                        a, op_name, b, expected, actual
                    );
                }
            }
        }
    }
}
