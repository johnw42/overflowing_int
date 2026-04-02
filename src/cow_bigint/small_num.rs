use std::mem::size_of;

pub type SmallInt = i128;
pub type SmallUint = u128;

const _: () = {
    assert!(size_of::<SmallInt>() == size_of::<SmallUint>());
};

crate::bytes_to_uint!();
