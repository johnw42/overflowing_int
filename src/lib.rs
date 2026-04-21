//! A wrapper around `num-bigint` that provides faster, more memory-efficient
//! representations for integers that fit in primitive types.  It can serve as a
//! drop-in replacement for `num-bigint` in many cases.
//!
//! ## Caveats
//!
//! Certain methods necessarily have different signatures.  In particular,
//! `from_bigint`, `to_bigint` and `magnitude` have different signatures, and
//! this crate has its own version of `TryFromBigIntError`.
//!
//! As nice as it would be for the wrapper types and `num-bigint`'s types to
//! implement a common trait, it isn't feasible because so much of the
//! functionality is provided by traits where the necessary trait bounds can't
//! be expressed as a `where` clause on the trait itself.
//!
//! ## Choosing an Implementation
//!
//! If you're working with numbers that are always, or almost always, big, this
//! crate is not for you.  Use `BigInt` and `BigUint` from `num-bigint` instead,
//! because they offer better performance for big values, and the overhead of
//! this crate's wrapper types is not worth it in that case.
//!
//! If you're working with numbers that often small and you can deal with
//! managing lifetimes, use `CowBigInt` and `CowBigUint` for the best
//! performance.
//!
//! If you'd rather avoid dealing with lifetimes, or you frequently need to
//! share values, `RcBigInt` and `RcBigUint` are a good choice.
//!
//! If you are working with number that are almost alway small enough to fit in
//! an 63 bits, your best option is ot use `RcBigIsize` and `RcBigUsize`, which
//! use the same amount of stack space as a pointer.
//!
//! If you need to share values across threads, `ArcBigInt` and its relatives
//! are a good choice.
//!
//! In some cases, `BoxBigInt` and its relatives may be better for for
//! performance than their `Arc` or `Rc` counterparts, but benchmark your code
//! to be sure!

pub use crate::{convert::TryFromBigIntError, encoding::Encoding};
pub use num_bigint::{BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint};

use crate::{
    arc_encoding::ArcEncoding, cow_encoding::CowEncoding, enum_encoding::EnumEncoding, signed::Int,
    unsigned::Uint,
};

pub mod arc_encoding;
pub mod big_number;
mod bignum_encoding;
mod bounds;
mod convert;
pub mod cow_encoding;
pub mod encoding;
mod enum_encoding;
mod macros;
mod num_ops;
mod num_tests;
mod shifted;
pub mod signed;
mod small_num;
mod trait_impl_tests;
mod trait_impls;
pub mod unsigned;

pub type ArcBigInt = Int<ArcEncoding<i128>>;
pub type ArcBigUint = Uint<ArcEncoding<u128>>;
pub type ArcBigIsize = Int<ArcEncoding<isize>>;
pub type ArcBigUsize = Uint<ArcEncoding<usize>>;
pub type CowBigInt<'a> = Int<CowEncoding<'a, i128>>;
pub type CowBigUint<'a> = Uint<CowEncoding<'a, u128>>;
pub type EnumBigInt = Int<EnumEncoding<i128>>;
pub type EnumBigUint = Uint<EnumEncoding<u128>>;

// Only for benchmarking, not for general use.
#[doc(hidden)]
pub mod bench {
    use super::*;

    pub type IdentityBigInt = Int<BigInt>;
    pub type IdentityBigUint = Uint<BigUint>;
}
