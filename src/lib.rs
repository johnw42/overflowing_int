pub use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};

pub use cbigint::*;

#[macro_use]
mod macros;

pub mod cbigint;
//pub mod clever;
mod encoding;
mod overflowing;

type Digit = isize;
type Udigit = usize;
