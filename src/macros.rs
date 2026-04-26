#![doc(hidden)]

#[macro_export]
macro_rules! duplicate_unsigned_encoded_types {
    ([$EncodedType:ident, $encoding_tag:ident] $($body:tt)*) => {
        duplicate::duplicate! {
            [
                IS_SIGNED  signedness  ImplType   $encoding_tag       $EncodedType;
                [false]    [unsigned]  [BigUint]  [cow_unsigned]      [$crate::CowUint128::<'static>];
                [false]    [unsigned]  [BigUint]  [arc_unsigned]      [$crate::ArcUint128];
                [false]    [unsigned]  [BigUint]  [identity_unsigned] [$crate::bench::IdentityBigUint];
                [false]    [unsigned]  [BigUint]  [enum_unsigned]     [$crate::OverflowingU128];
            ]
            $($body)*
        }
    };
    ($($body:tt)*) => {
        $crate::duplicate_unsigned_encoded_types! { [EncodedType, encoding_tag] $($body)* }
    }
}

#[macro_export]
macro_rules! duplicate_signed_encoded_types {
    ([$EncodedType:ident, $encoding_tag:ident] $($body:tt)*) => {
        duplicate::duplicate! {
            [
                IS_SIGNED  signedness  ImplType  $encoding_tag     $EncodedType                     UnsignedEncodedType;
                [true]     [signed]    [BigInt]  [cow_signed]      [$crate::CowInt128::<'static>]   [$crate::CowUint128::<'static>];
                [true]     [signed]    [BigInt]  [arc_signed]      [$crate::ArcInt128]              [$crate::ArcUint128];
                [true]     [signed]    [BigInt]  [identity_signed] [$crate::bench::IdentityBigInt]  [$crate::bench::IdentityBigUint];
                [true]     [signed]    [BigInt]  [enum_signed]     [$crate::OverflowingI128]        [$crate::OverflowingU128];
            ]
            $($body)*
        }
    };
    ($($body:tt)*) => {
        $crate::duplicate_signed_encoded_types! { [EncodedType, encoding_tag] $($body)* }
    }
}

#[macro_export]
macro_rules! duplicate_encoded_types {
    ($($body:tt)*) => {
        $crate::duplicate_signed_encoded_types! { $($body)* }
        $crate::duplicate_unsigned_encoded_types! { $($body)* }
    };
}

#[macro_export]
macro_rules! duplicate_generic_bignum {
    ($($body:tt)*) => {
        duplicate::duplicate! {
            [
                signedness EncodedType    ImplType;
                [signed]   [Int]          [BigInt];
                [unsigned] [Uint]         [BigUint];
            ]
            $($body)*
        }
    };
}

#[macro_export]
macro_rules! duplicate_arith_ops {
    ($($body:tt)*) => {
        duplicate::duplicate! {
            [
                op_type OpTrait  op_fn op_test_pred;
                [arith] [Add]    [add] [always];
                [arith] [Sub]    [sub] [can_subtract];
                [arith] [Mul]    [mul] [always];
                [arith] [Div]    [div] [nonzero_rhs];
                [arith] [Rem]    [rem] [nonzero_rhs];
            ]
            $($body)*
        }
    }
}

#[macro_export]
macro_rules! duplicate_shift_ops {
    ($($body:tt)*) => {
        duplicate::duplicate! {
            [
                op_type OpTrait  op_fn op_test_pred inverse_op_fn;
                [shift] [Shl]    [shl] [always]     [shr];
                [shift] [Shr]    [shr] [always]     [shl];
            ]
            $($body)*
        }
    }
}

#[macro_export]
macro_rules! duplicate_bit_ops {
    ($($body:tt)*) => {
        duplicate::duplicate! {
            [
                op_type OpTrait  op_fn    op_test_pred;
                [bit]   [BitAnd] [bitand] [always];
                [bit]   [BitOr]  [bitor]  [always];
                [bit]   [BitXor] [bitxor] [always]
            ]
            $($body)*
        }
    }
}

#[macro_export]
macro_rules! duplicate_arith_and_bit_ops {
    ($($body:tt)*) => {
        $crate::duplicate_arith_ops! { $($body)* }
        $crate::duplicate_bit_ops! { $($body)* }
    }
}

#[macro_export]
macro_rules! duplicate_iprims {
    ($($body:tt)*) => {
        duplicate::duplicate! {
            [
                prim    uprim;
                [i8]    [u8];
                [i16]   [u16];
                [i32]   [u32];
                [i64]   [u64];
                [i128]  [u128];
                [isize] [usize];
            ]
            $($body)*
        }
    }
}

#[macro_export]
macro_rules! duplicate_uprims {
    ($($body:tt)*) => {
        duplicate::duplicate! {
            [
                prim    iprim;
                [u8]    [i8];
                [u16]   [i16];
                [u32]   [i32];
                [u64]   [i64];
                [u128]  [i128];
                [usize] [isize];
            ]
            $($body)*
        }
    }
}

#[macro_export]
macro_rules! duplicate_prims {
    ($($body:tt)*) => {
        $crate::duplicate_uprims! { $($body)* }
        $crate::duplicate_iprims! { $($body)* }
    }
}

#[macro_export]
macro_rules! duplicate_iprims_if_unsigned {
    (signed; $($body:tt)*) => {
    };
    (unsigned; $($body:tt)*) => {
        $crate::duplicate_iprims! { $($body)* }
    };
}

#[macro_export]
macro_rules! duplicate_uprims_and_iprims_if_signed {
    (signed; $($body:tt)*) => {
        $crate::duplicate_iprims! { $($body)* }
        $crate::duplicate_uprims! { $($body)* }
    };
    (unsigned; $($body:tt)*) => {
        $crate::duplicate_uprims! { $($body)* }
    };
}

#[macro_export]
macro_rules! duplicate_prims_with_signedness {
    (signed; $($body:tt)*) => {
        $crate::duplicate_iprims! { $($body)* }
    };
    (unsigned; $($body:tt)*) => {
        $crate::duplicate_uprims! { $($body)* }
    };
}
