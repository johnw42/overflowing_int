#![allow(unused)]

pub use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};

use crate::{cow_encoding::CowEncoding, generic_bigint::GenericBigInt};

//pub use crate::cow_bigint::bigint_impl::CowBigInt;

pub mod big_number;
pub mod cow_encoding;
pub mod generic_bigint;
mod generic_bignum;
mod macros;
//pub mod rc_bignum;
mod small_num;

pub type CowBigInt<'a> = GenericBigInt<'a, CowEncoding<'a, i128, BigInt>>;
pub type CowBigUint<'a> = GenericBigInt<'a, CowEncoding<'a, u128, BigUint>>;
