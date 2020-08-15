macro_rules! call_macro {
    ($name:ident, $init_args:tt, $($final_args:tt),* $(,)?) => {
        $(call_macro!(@internal $name, $init_args, $final_args);)*
    };
    (@internal $name:ident, [$($init_arg:tt),*], [$($final_arg:tt),*]) => {
        $name!($($init_arg,)*$($final_arg),*);
    }
}

macro_rules! with_prims {
    ($macro:ident, $args:tt) => {
        call_macro!(
            $macro,
            $args,
            [int_prim, [i8, to_i8]],
            [int_prim, [i16, to_i16]],
            [int_prim, [i32, to_i32]],
            [int_prim, [i64, to_i64]],
            [int_prim, [i128, to_i128]],
            [int_prim, [isize, to_isize]],
            [int_prim, [u8, to_u8]],
            [int_prim, [u16, to_u16]],
            [int_prim, [u32, to_u32]],
            [int_prim, [u64, to_u64]],
            [int_prim, [u128, to_u128]],
            [int_prim, [usize, to_usize]],
            [float_prim, [f32, to_f32]],
            [float_prim, [f64, to_f64]],
        );
    };
}

macro_rules! with_ops {
    ($macro:ident, $args:tt) => {
        call_macro!(
            $macro,
            $args,
            [arith_op, [Add, add, AddAssign, add_assign, overflowing_add]],
            [arith_op, [Sub, sub, SubAssign, sub_assign, overflowing_sub]],
            [arith_op, [Mul, mul, MulAssign, mul_assign, overflowing_mul]],
            [arith_op, [Div, div, DivAssign, div_assign, overflowing_div]],
            [arith_op, [Rem, rem, RemAssign, rem_assign, overflowing_rem]],
            [shift_op, [Shl, shl, ShlAssign, shl_assign, overflowing_shl]],
            [shift_op, [Shr, shr, ShrAssign, shr_assign, overflowing_shr]],
            [bit_op, [BitAnd, bitand, BitAndAssign, bitand_assign]],
            [bit_op, [BitOr, bitor, BitOrAssign, bitor_assign]],
            [bit_op, [BitXor, bitxor, BitXorAssign, bitxor_assign]],
        );
    };
}

macro_rules! with_ops_for_each_prim {
    [$macro:ident, [$($arg:tt),*], $prim_type:ident, [$prim:ident, $to_prim:ident]] => {
        with_ops!($macro, [$($arg,)* $prim_type, [$prim, $to_prim]]);
    };
}

macro_rules! with_prims_and_ops {
    ($macro:ident, $args:tt) => {
        with_prims!(with_ops_for_each_prim, [$macro, $args]);
    };
}
