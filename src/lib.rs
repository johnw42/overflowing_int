#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

pub use crate::{convert::TryFromBigIntError, encoding::Encoding};
pub use num_bigint::{BigInt, BigUint, ParseBigIntError, Sign, ToBigInt, ToBigUint};

use crate::{
    encoding::arc::ArcEncoding,
    encoding::cow::CowEncoding,
    encoding::decoded::DecodedEncoding,
    wrappers::{Int, Uint},
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

/// An integer encoded as either an `i64` or an `Arc<BigInt>`, depending on the value.
pub type ArcInt64 = Int<ArcEncoding<i64>>;

/// An unsigned integer encoded as either a `u64` or an `Arc<BigUint>`, depending on the value.
pub type ArcUint64 = Uint<ArcEncoding<u64>>;

/// An integer encoded as either an `i128` or a `Arc<BigInt>`, depending on the value.
pub type ArcInt128 = Int<ArcEncoding<i128>>;

/// An unsigned integer encoded as either an `u128` or an `Arc<BigUint>`, depending on the value.
pub type ArcUint128 = Uint<ArcEncoding<u128>>;

/// An integer encoded as either an `i64` or a `Cow<'a, BigInt>`, depending on the value.
pub type CowInt64<'a> = Int<CowEncoding<'a, i64>>;

/// An unsigned integer encoded as either a `u64` or a `Cow<'a, BigUint>`, depending on the value.
pub type CowUint64<'a> = Uint<CowEncoding<'a, u64>>;

/// An integer encoded as either an `i128` or a `Cow<'a, BigInt>`, depending on the value.
pub type CowInt128<'a> = Int<CowEncoding<'a, i128>>;

/// An unsigned integer encoded as either a `u128` or a `Cow<'a, BigUint>`, depending on the value.
pub type CowUint128<'a> = Uint<CowEncoding<'a, u128>>;

/// An integer encoded as either an `i64` or a `BigInt`, depending on the value.
pub type OverflowingI64 = Int<DecodedEncoding<i64>>;

/// An unsigned integer encoded as either a `u64` or a `BigUint`, depending on the value.
pub type OverflowingU64 = Uint<DecodedEncoding<u64>>;

/// An integer encoded as either an `i128` or a `BigInt`, depending on the value.
pub type OverflowingI128 = Int<DecodedEncoding<i128>>;

/// An unsigned integer encoded as either a `u128` or a `BigUint`, depending on the value.
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
