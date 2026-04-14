pub use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};

use crate::{
    box_encoding::BoxEncoding, cow_encoding::CowEncoding, rc_encoding::RcEncoding, signed::Int,
    unsigned::Uint,
};

pub mod big_number;
mod bounds;
mod box_encoding;
mod convert;
pub mod cow_encoding;
mod encoding;
pub mod identity_encoding;
mod macros;
mod num_ops;
pub mod rc_encoding;
mod shifted;
mod signed;
mod small_num;
mod trait_impl_tests;
mod trait_impls;
mod unsigned;

pub type CowBigInt<'a> = Int<'a, CowEncoding<'a, i128>>;
pub type CowBigUint<'a> = Uint<'a, CowEncoding<'a, u128>>;
pub type RcBigInt = Int<'static, RcEncoding<i128>>;
pub type RcBigUint = Uint<'static, RcEncoding<u128>>;
pub type RcBigIsize = Int<'static, RcEncoding<isize>>;
pub type RcBigUsize = Uint<'static, RcEncoding<usize>>;
pub type BoxBigInt = Int<'static, BoxEncoding<i128>>;
pub type BoxBigUint = Uint<'static, BoxEncoding<u128>>;

// Only for benchmarking, not for general use.
pub mod bench {
    use super::*;
    use crate::identity_encoding::IdentityEncoding;

    pub type IdentityBigInt = Int<'static, IdentityEncoding<isize>>;
    pub type IdentityBigUint = Uint<'static, IdentityEncoding<usize>>;
}
