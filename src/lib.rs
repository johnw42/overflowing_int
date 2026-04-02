pub use crate::big_integer::BigInteger;
pub use crate::big_natural::BigNatural;
pub use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};

pub use crate::cow_bigint::bigint_impl::CowBigInt;

pub mod big_integer;
pub mod big_natural;
pub mod cow_bigint;
mod macros;
pub mod rc_bigint;
mod small_uint;

pub type SmallInt = i128;
pub type SmallUint = u128;

const SMALL_BITS: usize = std::mem::size_of::<SmallInt>() * 8;
