#[macro_export]
macro_rules! duplicate_arith_ops {
    ($($body:tt)*) => {
        duplicate! {
            [
                op_type op_trait op_fn op_test_pred;
                [arith] [Add]    [add] [always];
                [arith] [Sub]    [sub] [always];
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
        duplicate! {
            [
                op_type op_trait op_fn op_test_pred inverse_op_fn;
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
        duplicate! {
            [
                op_type op_trait op_fn    op_test_pred;
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
        duplicate_arith_ops! { $($body)* }
        duplicate_bit_ops! { $($body)* }
    }
}

#[macro_export]
macro_rules! duplicate_uprims {
    ($($body:tt)*) => {
        duplicate! {
            [
                prim;
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

#[macro_export]
macro_rules! duplicate_prims {
    ($($body:tt)*) => {
        duplicate_uprims! { $($body)* }
        duplicate! {
            [
                prim;
                [i8];
                [i16];
                [i32];
                [i64];
                [i128];
                [isize];
            ]
            $($body)*
        }
    }
}

#[macro_export]
macro_rules! bytes_to_uint {
    () => {
        pub trait SmallNum:
            Copy
            + Eq
            + num_traits::CheckedShl
            + num_traits::Num
            + std::ops::BitAnd<Output = Self>
            + std::ops::BitOr<Output = Self>
            + std::ops::Shr<u32, Output = Self>
        {
        }

        impl SmallNum for SmallInt {}
        impl SmallNum for SmallUint {}

        pub fn bytes_to_uint_be(bytes: &[u8]) -> Option<SmallUint> {
            let mut buf = [0u8; size_of::<SmallUint>()];
            if bytes.len() <= buf.len() {
                let start = buf.len() - bytes.len();
                buf[start..].copy_from_slice(bytes);
                Some(SmallUint::from_be_bytes(buf))
            } else {
                None
            }
        }

        pub fn bytes_to_uint_le(bytes: &[u8]) -> Option<SmallUint> {
            let mut buf = [0u8; size_of::<SmallUint>()];
            if bytes.len() <= buf.len() {
                buf[0..bytes.len()].copy_from_slice(bytes);
                Some(SmallUint::from_le_bytes(buf))
            } else {
                None
            }
        }

        #[test]
        fn test_bytes_to_uint_be() {
            assert_eq!(bytes_to_uint_be(&[0x00, 0x01]), Some(0x01));
            assert_eq!(bytes_to_uint_be(&[0x01, 0x00]), Some(0x0100));
            assert_eq!(bytes_to_uint_be(&[0x12, 0x34]), Some(0x1234));
            assert_eq!(
                bytes_to_uint_be(&[0xFF; size_of::<SmallUint>()]),
                Some(SmallUint::MAX)
            );
            assert_eq!(bytes_to_uint_be(&[0xFF; size_of::<SmallUint>() + 1]), None);
        }

        #[test]
        fn test_bytes_to_uint_le() {
            assert_eq!(bytes_to_uint_le(&[0x01, 0x00]), Some(0x01));
            assert_eq!(bytes_to_uint_le(&[0x00, 0x01]), Some(0x0100));
            assert_eq!(bytes_to_uint_le(&[0x34, 0x12]), Some(0x1234));
            assert_eq!(
                bytes_to_uint_le(&[0xFF; size_of::<SmallUint>()]),
                Some(SmallUint::MAX)
            );
            assert_eq!(bytes_to_uint_le(&[0xFF; size_of::<SmallUint>() + 1]), None);
        }
    };
}
