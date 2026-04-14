#[macro_export]
macro_rules! duplicate_unsigned_encoded_types {
    ($($body:tt)*) => {
        duplicate::duplicate! {
            [
                signedness  ImplType   encoding_tag        EncodedType;
                [unsigned]  [BigUint]  [cow_unsigned]      [$crate::CowBigUint::<'static>];
                [unsigned]  [BigUint]  [rc_unsigned]       [$crate::RcBigUint];
                [unsigned]  [BigUint]  [rc_usize]          [$crate::RcBigUsize];
                [unsigned]  [BigUint]  [arc_unsigned]      [$crate::ArcBigUint];
                [unsigned]  [BigUint]  [identity_unsigned] [$crate::bench::IdentityBigUint::<'static>];
                [unsigned]  [BigUint]  [box_unsigned]      [$crate::BoxBigUint];
            ]
            $($body)*
        }
    };
}

#[macro_export]
macro_rules! duplicate_signed_encoded_types {
    ($($body:tt)*) => {
        duplicate::duplicate! {
            [
                signedness  ImplType  encoding_tag      EncodedType;
                [signed]    [BigInt]  [cow_signed]      [$crate::CowBigInt::<'static>];
                [signed]    [BigInt]  [rc_signed]       [$crate::RcBigInt];
                [signed]    [BigInt]  [rc_isize]        [$crate::RcBigIsize];
                [signed]    [BigInt]  [arc_signed]      [$crate::ArcBigInt];
                [signed]    [BigInt]  [identity_signed] [$crate::bench::IdentityBigInt::<'static>];
                [signed]    [BigInt]  [box_signed]      [$crate::BoxBigInt];
            ]
            $($body)*
        }
    };
}

#[macro_export]
macro_rules! duplicate_encoded_types {
    ($($body:tt)*) => {
        crate::duplicate_signed_encoded_types! { $($body)* }
        crate::duplicate_unsigned_encoded_types! { $($body)* }
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
