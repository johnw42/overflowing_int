use std::mem::size_of;

pub type SmallInt = isize;
pub type SmallUint = usize;

const _: () = {
    assert!(size_of::<SmallInt>() == size_of::<SmallUint>());
};

crate::bytes_to_uint!();
