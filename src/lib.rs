pub use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};

use crate::{
    cow_encoding::CowEncoding,
    generic_bignum::{signed::GenericSignedBigNum, unsigned::GenericUnsignedBigNum},
    rc_encoding::RcEncoding,
};

pub mod big_number;
mod convert;
pub mod cow_encoding;
mod generic_bignum;
mod macros;
pub mod rc_encoding;
mod small_num;

pub type CowBigInt<'a> = GenericSignedBigNum<'a, CowEncoding<'a, i128>>;
pub type CowBigUint<'a> = GenericUnsignedBigNum<'a, CowEncoding<'a, u128>>;
pub type RcBigInt = GenericSignedBigNum<'static, RcEncoding<isize>>;
pub type RcBigUint = GenericUnsignedBigNum<'static, RcEncoding<usize>>;
