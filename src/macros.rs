macro_rules! call_macro {
    ($name:ident, $init_args:tt, $($final_args:tt),* $(,)?) => {
        $(call_macro!(@internal $name, $init_args, $final_args);)*
    };
    (@internal $name:ident, [$($init_arg:tt),* $(,)?], [$($final_arg:tt),* $(,)?]) => {
        $name!($($init_arg,)*$($final_arg),*);
    }
}

macro_rules! with_prims {
    ($macro:ident, $args:tt) => {
        call_macro!(
            $macro,
            $args,
            [[int, signed], [i8, to_i8]],
            [[int, signed], [i16, to_i16]],
            [[int, signed], [i32, to_i32]],
            [[int, signed], [i64, to_i64]],
            [[int, signed], [i128, to_i128]],
            [[int, signed], [isize, to_isize]],
            [[int, unsigned], [u8, to_u8]],
            [[int, unsigned], [u16, to_u16]],
            [[int, unsigned], [u32, to_u32]],
            [[int, unsigned], [u64, to_u64]],
            [[int, unsigned], [u128, to_u128]],
            [[int, unsigned], [usize, to_usize]],
            [[float], [f32, to_f32]],
            [[float], [f64, to_f64]],
        );
    };
}

macro_rules! with_ops {
    ($macro:ident, $args:tt) => {
        call_macro!(
            $macro,
            $args,
            [arith_op, [Add, add, AddAssign, add_assign]],
            [arith_op, [Sub, sub, SubAssign, sub_assign]],
            [arith_op, [Mul, mul, MulAssign, mul_assign]],
            [arith_op, [Div, div, DivAssign, div_assign]],
            [arith_op, [Rem, rem, RemAssign, rem_assign]],
            [shift_op, [Shl, shl, ShlAssign, shl_assign]],
            [shift_op, [Shr, shr, ShrAssign, shr_assign]],
            [bit_op, [BitAnd, bitand, BitAndAssign, bitand_assign]],
            [bit_op, [BitOr, bitor, BitOrAssign, bitor_assign]],
            [bit_op, [BitXor, bitxor, BitXorAssign, bitxor_assign]],
        );
    };
}
//
// macro_rules! with_ops_for_each_prim {
//     [$macro:ident, [$($arg:tt),*] $(, $prim_arg:tt)*] => {
//         with_ops!($macro, [$($arg,)* $($prim_arg,)*]);
//     };
// }
//
// macro_rules! with_prims_and_ops {
//     ($macro:ident, $args:tt) => {
//         with_prims!(with_ops_for_each_prim, [$macro, $args]);
//     };
// }
