//! A wrapper around `num-bigint` that provides faster, more memory-efficient
//! representations for integers that fit in primitive types.  It can serve as a
//! drop-in replacement for `num-bigint` in most cases.  The intended use case
//! is to choose one of the implementations and import it using renaming, such
//! as `use overflowing_int::ArcInt128 as BigInt;`.
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
//! In general, my recommendation is to use the `Arc`-based wrapper types
//! because they provide a good balance of performance and convenience, and they
//! have the ability to share big values between instances, which is often
//! relevant because many formulas include the same value more than once.  In
//! particular, the 64-bit `Arc` wrapper types are the smallest, making them an
//! ideal choice for cases where large values are rare, such as an interpreter
//! that needs to support arbitrary precision integers.
//!
//! The `Cow`-based wrapper types are a good choice if you want to avoid
//! depending on code that uses `unsafe`, or if you want to be able to borrow
//! small values from a big value without cloning.  However, they are less
//! convenient to use because they have lifetime parameters that must be
//! managed.
//!
//! The `Overflowing` wrapper types are the most naive implementation.  Only use
//! them if you don't care about sharing bignum values and you want to avoid
//! depending on code that uses `unsafe`.
//!
//! The following table summarizes the differences between the wrapper types,
//! assuming a 64-bit target architecture.  The "Max Small Bits" column
//! indicates the maximum number of bits that can be stored inline.
//!
//! | Type            | size_of | Max Small Bits | Uses `unsafe`? |
//! | --------------- | ------- | -------------- | -------------- |
//! | BigInt          |      32 |              0 |             no |
//! | BigUint         |      24 |              0 |             no |
//! | ArcInt64        |       8 |             63 |            yes |
//! | ArcUint64       |       8 |             63 |            yes |
//! | ArcInt128       |      16 |            127 |            yes |
//! | ArcUint128      |      16 |            127 |            yes |
//! | CowInt64        |      32 |             64 |             no |
//! | CowUint64       |      24 |             64 |             no |
//! | CowInt128       |      32 |            128 |             no |
//! | CowUint128      |      32 |            128 |             no |
//! | OverflowingI64  |      32 |             64 |             no |
//! | OverflowingU64  |      24 |             64 |             no |
//! | OverflowingI128 |      32 |            128 |             no |
//! | OverflowingU128 |      32 |            128 |             no |
//!
//! ## Safety
//!
//! All of the wrapper types in this crate are safe to use, and all of their
//! methods are safe to call.  However, if you're concerned about depending code
//! that uses `unsafe`, avoid using the `Arc`-based wrapper types, because they
//! use `unsafe` internally and assume pointers never have 1 in their LSB.
//!
//! ## Performance
//!
//! This crate has been heavily benchmarked to ensure optimal performance and to
//! highlight the performance characteristics of the different encoding types.
//!
//! For small numbers, the 64-bit wrapper types outperform `num-bigint` by a
//! factor of roughly 10x, and the 128-bit wrapper types outperform `num-bigint`
//! by a factor of roughly 3x.
//!
//! For big numbers, the overhead of the wrapper types is generally around a
//! factor of 2.
//!
//! There are no `Rc`-based wrapper types because benchmarking has shown that
//! they don't provide any meaningful performance benefits over the `Arc`-based
//! wrapper types.  In the case of small numbers, no reference counting is done,
//! and in the case of big numbers, the overhead of reference counting is
//! negligible compared to the overhead of performing arithmetic on big numbers.
//!
//! ## Additions to the `BigInt` and `BigUint` APIs
//!
//! The wrapper types provide a few methods that are not present on `BigInt` and
//! `BigUint`.  The `into_static` method, useful only for `Cow`-based types,
//! uses cloning if necessary to coerce a value into having a `'static`
//! lifetime.  The `reencode_from` method allows a value to be re-encoded from
//! one encoding type to another, which is useful when mixing encodings (though
//! I don't necessarily recommend mixing encodings in a single codebase).
//!
//! ## Testing
//!
//! This crate include a thorough set of unit tests, with a large number of
//! property-based tests using the `quickcheck` crate.  The property-based tests
//! are designed to test the consistency of the wrapper types with `BigInt` and
//! `BigUint`.  They include tests for all overloaded operators and other
//! methods implemented through traits.
//!
//! The tests assume the `num-bigint` types are themselves correct, so they are
//! not designed to catch bugs in `num-bigint`'s implementation.  Instead, they
//! are designed to catch bugs in the wrapper types' implementation, and to
//! ensure that the wrapper types are consistent with `num-bigint`'s types.
//!
//! ## Implementation Notes
//!
//! The implementation of this crate is in layers.  The lowest layer is the
//! `BigNumber` trait, which exposes the APIs of `BigInt` and `BigUint` through
//! a trait that includes all their overloaded operators in a single trait.
//!
//! The next layer is the `Encoding` trait, which defines how to encode small
//! values and big values, and how to convert between them.  This layer is where
//! the different encoding strategies are implemented.
//!
//! The top layer is the wrapper types, which are the types that users of this
//! crate will interact with.  They are implemented in terms of the Encoding
//! trait, and they provide roughly the same APIs as `BigInt` and `BigUint`,
//! with its same trait implementations, but with different performance
//! characteristics depending on the encoding used.
//!
//! Overloaded operators, implemented in the `num_ops` module, have their own
//! set of layers.  The top level is a set of traits for different categories of
//! operators: arithmetic, bitwise, shift, and the power operator.  These trait
//! provide the logic to dispatch different argument types to the most specific
//! overloads provided by `BigInt` and `BigUint`.  These operators come in many
//! combinations of argument types based which primitive type they accept and
//! whether the arguments are consumed or borrowed.
//!
//! In the next layer, the operator traits are implemented for specific
//! operators, and finally, the operators themselves are implemented on the
//! wrapper types by delegating to the operator trait implementations.
//!
//! This crate makes heavy use of macros to avoid code duplication.  The
//! `duplicate` crate used heavily, and the `paste` crate is indespensible for
//! building method names systematically.

pub use crate::{convert::TryFromBigIntError, encoding::Encoding};
pub use num_bigint::{BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint};

use crate::{
    encoding::arc::ArcEncoding, encoding::cow::CowEncoding, encoding::decoded::DecodedEncoding,
    wrappers::int::Int, wrappers::uint::Uint,
};

mod convert;
mod encoding;
mod macros;
mod num_ops;
mod num_tests;
mod num_traits;
mod trait_impl_tests;
mod trait_impls;
pub mod wrappers;

pub type ArcInt64 = Int<ArcEncoding<i64>>;
pub type ArcUint64 = Uint<ArcEncoding<u64>>;
pub type ArcInt128 = Int<ArcEncoding<i128>>;
pub type ArcUint128 = Uint<ArcEncoding<u128>>;
pub type CowInt64<'a> = Int<CowEncoding<'a, i64>>;
pub type CowUint64<'a> = Uint<CowEncoding<'a, u64>>;
pub type CowInt128<'a> = Int<CowEncoding<'a, i128>>;
pub type CowUint128<'a> = Uint<CowEncoding<'a, u128>>;
pub type OverflowingI64 = Int<DecodedEncoding<i64>>;
pub type OverflowingU64 = Uint<DecodedEncoding<u64>>;
pub type OverflowingI128 = Int<DecodedEncoding<i128>>;
pub type OverflowingU128 = Uint<DecodedEncoding<u128>>;

// Only for benchmarking, not for general use.
#[doc(hidden)]
pub mod bench {
    use super::*;

    pub type IdentityBigInt = Int<BigInt>;
    pub type IdentityBigUint = Uint<BigUint>;
}

#[cfg(target_pointer_width = "64")]
const _: () = {
    use std::mem::size_of;

    assert!(size_of::<ArcInt64>() == 8);
    assert!(size_of::<ArcUint64>() == 8);
    assert!(size_of::<ArcInt128>() == 16);
    assert!(size_of::<ArcUint128>() == 16);
    assert!(size_of::<CowInt64<'_>>() == 32);
    assert!(size_of::<CowUint64<'_>>() == 24);
    assert!(size_of::<CowInt128<'_>>() == 32);
    assert!(size_of::<CowUint128<'_>>() == 32);
    assert!(size_of::<OverflowingI64>() == 32);
    assert!(size_of::<OverflowingU64>() == 24);
    assert!(size_of::<OverflowingI128>() == 32);
    assert!(size_of::<OverflowingU128>() == 32);
};
