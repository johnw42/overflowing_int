pub use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};

pub mod big_number;
mod convert;
pub mod cow_encoding;
mod generic_bignum;
pub mod generic_signed_bignum;
pub mod generic_unsigned_bignum;
mod macros;
pub mod rc_encoding;
mod small_num;

pub type CowBigInt<'a> =
    generic_signed_bignum::GenericSignedBigNum<'a, cow_encoding::CowEncoding<'a, i128>>;
pub type CowBigUint<'a> =
    generic_unsigned_bignum::GenericUnsignedBigNum<'a, cow_encoding::CowEncoding<'a, u128>>;
pub type RcBigInt =
    generic_signed_bignum::GenericSignedBigNum<'static, rc_encoding::RcEncoding<isize>>;
pub type RcBigUint =
    generic_unsigned_bignum::GenericUnsignedBigNum<'static, rc_encoding::RcEncoding<usize>>;
