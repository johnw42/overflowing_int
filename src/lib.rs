pub use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};

use crate::{
    cow_encoding::CowEncoding,
    rc_encoding::RcEncoding,
    {signed::GenericSignedBigNum, unsigned::GenericUnsignedBigNum},
};

pub mod big_number;
mod convert;
pub mod cow_encoding;
mod encoding;
mod macros;
mod num_ops;
pub mod rc_encoding;
mod signed;
mod small_num;
mod trait_impl_tests;
mod trait_impls;
mod unsigned;

pub type CowBigInt<'a> = GenericSignedBigNum<'a, CowEncoding<'a, i128>>;
pub type CowBigUint<'a> = GenericUnsignedBigNum<'a, CowEncoding<'a, u128>>;
pub type RcBigInt = GenericSignedBigNum<'static, RcEncoding<isize>>;
pub type RcBigUint = GenericUnsignedBigNum<'static, RcEncoding<usize>>;
