pub use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};

pub use cbigint::*;
use digits::*;

mod accum;
pub mod cbigint;
mod num_trait_impls;
//pub mod clever;
mod checked;
mod convert;
mod encoding;
mod ops;
mod to_cow;

#[cfg(feature = "i128_digit")]
pub mod digits {
    pub type Digit = i128;
    pub type Udigit = u128;
}

#[cfg(not(feature = "i128_digit"))]
pub mod digits {
    pub type Digit = isize;
    pub type Udigit = usize;
}

const DIGIT_BITS: usize = std::mem::size_of::<Digit>() * 8;
