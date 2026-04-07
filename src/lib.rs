pub use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};

pub mod big_number;
mod convert;
pub mod cow_encoding;
pub mod generic_bigint;
mod generic_bignum;
pub mod generic_biguint;
mod macros;
pub mod rc_encoding;
mod small_num;

pub type CowBigInt<'a> = generic_bigint::GenericBigInt<'a, cow_encoding::CowEncoding<'a, i128>>;
pub type CowBigUint<'a> = generic_biguint::GenericBigUint<'a, cow_encoding::CowEncoding<'a, u128>>;
pub type RcBigInt = generic_bigint::GenericBigInt<'static, rc_encoding::RcEncoding<isize>>;
pub type RcBigUint = generic_biguint::GenericBigUint<'static, rc_encoding::RcEncoding<usize>>;
