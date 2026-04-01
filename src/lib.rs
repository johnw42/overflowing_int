pub use crate::big_integer::BigInteger;
pub use crate::big_natural::BigNatural;
pub use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};

pub use crate::cbigint::cbigint_impl::CBigInt;

mod accum;
pub mod big_integer;
pub mod big_natural;
pub mod cbigint;
mod convert;
mod encoding;
mod macros;
mod num_trait_impls;
mod ops;

pub type SmallInt = i128;
pub type SmallUint = u128;

const SMALL_BITS: usize = std::mem::size_of::<SmallInt>() * 8;
