use crate::cbigint::CBigInt;
use crate::decoded::Decoded;
use crate::overflowing::Overflowing;
use crate::to_cow::{ToCow, ToDecodedCow};
use crate::Digit;
use num_bigint::BigInt;
use std::borrow::Cow;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

struct BigIntOp {
    digits: fn(Digit, Digit) -> Option<Digit>,
    owned: fn(BigInt, BigInt) -> BigInt,
    owned_borrowed: fn(BigInt, &BigInt) -> BigInt,
    borrowed_owned: for<'a> fn(&'a BigInt, BigInt) -> BigInt,
    borrowed: for<'a> fn(&'a BigInt, &'a BigInt) -> BigInt,
}

impl BigIntOp {
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
}

macro_rules! bigint_op {
    [arith_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        pub(super) const $op: BigIntOp = BigIntOp {
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
        };
    };
    [bit_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        pub(super) const $op: BigIntOp = BigIntOp {
            digits: |lhs, rhs| Some($trait::$op(lhs, rhs)),
            owned: |lhs: BigInt, rhs: BigInt| $trait::$op(lhs, rhs),
            owned_borrowed: |lhs: BigInt, rhs: &BigInt| $trait::$op(lhs, rhs),
            borrowed_owned: |lhs: &BigInt, rhs: BigInt| $trait::$op(lhs, rhs),
            borrowed: |lhs: &BigInt, rhs: &BigInt| $trait::$op(lhs, rhs),
        };
    };
    [$($_1:tt),*] => {};
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
    [shift_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        //assign_op!($trait, $op, $assign_trait, $assign_op);
    };
    [bit_op, [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident]] => {
        op_traits!(arith_op, [$trait, $op, $assign_trait, $assign_op]);
        // impl<T> $assign_trait<T> for CBigInt
        // where
        //     CBigInt: $trait<T, Output = CBigInt>,
        //     BigInt: $assign_trait<T>,
        // {
        //     fn $assign_op(&mut self, rhs: T) {
        //         match self.decode_mut() {
        //             Decoded::Digit(_) => {
        //                 let lhs = std::mem::take(self);
        //                 *self = lhs.$op(rhs);
        //             }
        //             Decoded::Big(big) => {
        //                 big.$assign_op(rhs);
        //             }
        //         }
        //     }
        // }
    };
}

macro_rules! assign_op {
    [$trait:ident, $op:ident, $assign_trait:ident, $assign_op:ident] => {
        impl<T> $assign_trait<T> for CBigInt
        where
            CBigInt: $trait<T, Output = CBigInt>,
            BigInt: From<T>,
            BigInt: $assign_trait,
        {
            fn $assign_op(&mut self, rhs: T) {
                match self.decode_mut() {
                    Decoded::Digit(_) => {
                        let lhs = std::mem::take(self);
                        *self = lhs.$op(rhs);
                    }
                    Decoded::Big(big) => {
                        big.$assign_op(BigInt::from(rhs));
                    }
                }
            }
        }
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
            fn $op(self, rhs: $prim) -> Self::Output {
                BigInt::from(self).$op(rhs).into()
                // if let Small(lhs) = &self {
                //     if let Ok(rhs) = u32::try_from(rhs) {
                //         if let (result, false) = lhs.$overflowing_op(rhs) {
                //             return result.into();
                //         }
                //     }
                // }
                // BigInt::from(self).$op(rhs).into()
            }
        }
        impl $assign_trait<$prim> for CBigInt {
            fn $assign_op(&mut self, rhs: $prim) {
                let mut lhs = BigInt::from(std::mem::take(self));
                lhs.$assign_op(rhs);
                *self = lhs.into();
            }
        }
        // ref_op!($trait<$prim> for CBigInt, $op);
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
