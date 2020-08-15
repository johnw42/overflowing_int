pub use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};

pub use cbigint::*;

pub mod cbigint;
//pub mod clever;
mod encoding;

type Digit = i64;
type Udigit = u64;
