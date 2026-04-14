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

pub use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint, TryFromBigIntError,
};

use crate::{
    arc_encoding::ArcEncoding, box_encoding::BoxEncoding, cow_encoding::CowEncoding,
    rc_encoding::RcEncoding, signed::Int, unsigned::Uint,
};

pub mod arc_encoding;
pub mod big_number;
mod bounds;
mod box_encoding;
mod convert;
pub mod cow_encoding;
mod encoding;
pub mod identity_encoding;
mod macros;
mod num_ops;
pub mod rc_encoding;
mod shifted;
mod signed;
mod small_num;
mod trait_impl_tests;
mod trait_impls;
mod unsigned;

pub type CowBigInt<'a> = Int<'a, CowEncoding<'a, i128>>;
pub type CowBigUint<'a> = Uint<'a, CowEncoding<'a, u128>>;
pub type RcBigInt = Int<'static, RcEncoding<i128>>;
pub type RcBigUint = Uint<'static, RcEncoding<u128>>;
pub type RcBigIsize = Int<'static, RcEncoding<isize>>;
pub type RcBigUsize = Uint<'static, RcEncoding<usize>>;
pub type ArcBigInt = Int<'static, ArcEncoding<i128>>;
pub type ArcBigUint = Uint<'static, ArcEncoding<u128>>;
pub type ArcBigIsize = Int<'static, ArcEncoding<isize>>;
pub type ArcBigUsize = Uint<'static, ArcEncoding<usize>>;
pub type BoxBigInt = Int<'static, BoxEncoding<i128>>;
pub type BoxBigUint = Uint<'static, BoxEncoding<u128>>;
pub type BoxBigIsize = Int<'static, BoxEncoding<isize>>;
pub type BoxBigUsize = Uint<'static, BoxEncoding<usize>>;

// Only for benchmarking, not for general use.
pub mod bench {
    use super::*;
    use crate::identity_encoding::IdentityEncoding;

    pub type IdentityBigInt = Int<'static, IdentityEncoding<isize>>;
    pub type IdentityBigUint = Uint<'static, IdentityEncoding<usize>>;
}
