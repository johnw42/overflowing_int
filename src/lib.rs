pub use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};

pub use cbigint::*;

mod accum;
pub mod big_integer;
pub mod big_natural;
pub mod cbigint;
mod checked;
mod convert;
mod encoding;
mod num_trait_impls;
mod ops;
mod to_cow;

pub type Digit = i128;
pub type Udigit = u128;

const DIGIT_BITS: usize = std::mem::size_of::<Digit>() * 8;
