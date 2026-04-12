#![cfg(test)]
#![allow(unused_imports)]
use crate::encoding::{Decode, Decoded, Encode, Encoding};
use crate::signed::Int;
use crate::small_num::SmallNumber;
use crate::unsigned::Uint;
use crate::{CowBigInt, CowBigUint, RcBigInt, RcBigUint, duplicate_generic_bignum_types};
use crate::{duplicate_iprims, duplicate_prims, duplicate_uprims};
use duplicate::duplicate_item;
use num_bigint::{
    BigInt, BigUint, ParseBigIntError, RandomBits, Sign, UniformBigInt, UniformBigUint,
};
use num_integer::{Integer, Roots};
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedEuclid, CheckedMul, CheckedSub, Euclid, FromBytes,
    FromPrimitive, Num, One, Signed, ToBytes, ToPrimitive, Zero,
};
use paste::paste;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformSampler};
use rand::prelude::Distribution;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{Binary, Formatter, LowerHex, Octal, UpperHex};
use std::ops::{Neg, Not};
use std::str::FromStr;

duplicate_generic_bignum_types! { mod bignum_tag {
    use super::*;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    use serde::{Deserialize, Serialize};

    //
    // Binary
    //

    #[quickcheck]
    fn test_binary(n: RawType) -> bool {
        dbg!(format!("{:b}", n)) == dbg!(format!("{:b}", EncodedType::from(n)))
    }

    //
    // CheckedAdd
    //

    #[quickcheck]
    fn test_checked_add(a: RawType, b: RawType) -> bool {
        CheckedAdd::checked_add(&a, &b)
            == CheckedAdd::checked_add(&EncodedType::from(a), &EncodedType::from(b)).map(Into::into)
    }

    #[quickcheck]
    fn test_checked_div(a: RawType, b: RawType) -> bool {
        CheckedDiv::checked_div(&a, &b)
            == CheckedDiv::checked_div(&EncodedType::from(a), &EncodedType::from(b)).map(Into::into)
    }

    //
    // CheckedDiv
    //

    #[quickcheck]
    fn test_checked_mul(a: RawType, b: RawType) -> bool {
        CheckedMul::checked_mul(&a, &b)
            == CheckedMul::checked_mul(&EncodedType::from(a), &EncodedType::from(b)).map(Into::into)
    }

    //
    // CheckedSub
    //

    #[quickcheck]
    fn test_checked_sub(a: RawType, b: RawType) -> bool {
        CheckedSub::checked_sub(&a, &b)
            == CheckedSub::checked_sub(&EncodedType::from(a), &EncodedType::from(b)).map(Into::into)
    }

    //
    // Debug
    //

    #[quickcheck]
    fn test_debug(n: RawType) -> bool {
        let raw_display = format!("{:?}", n);
        let test_display = format!("{:?}", EncodedType::from(n));
        test_display.contains(&raw_display)
    }

    //
    // Deserialize
    //

    #[quickcheck]
    fn test_deserialize(n: RawType) -> bool {
        let s = serde_json::to_string(&EncodedType::from(n.clone())).unwrap();
        serde_json::from_str::<EncodedType>(&s).ok().map(Into::into) == Some(n)
    }

    //
    // Display
    //

    #[quickcheck]
    fn test_display(n: RawType) -> bool {
        format!("{}", n) == format!("{}", EncodedType::from(n))
    }

    //
    // LowerHex
    //

    #[quickcheck]
    fn test_lower_hex(n: RawType) -> bool {
        format!("{:x}", n) == format!("{:x}", EncodedType::from(n))
    }

    //
    // Octal
    //

    #[quickcheck]
    fn test_octal(n: RawType) -> bool {
        format!("{:o}", n) == format!("{:o}", EncodedType::from(n))
    }

    //
    // UpperHex
    //

    #[quickcheck]
    fn test_upper_hex(n: RawType) -> bool {
        format!("{:X}", n) == format!("{:X}", EncodedType::from(n))
    }

    //
    // FromStr
    //

    #[quickcheck]
    fn test_from_str(n: RawType) -> bool {
        let s = n.to_string();
        RawType::from_str(&s).ok() == EncodedType::from_str(&s).ok().map(Into::into)
    }

    //
    // Integer
    //

    #[quickcheck]
    fn test_div_floor(a: RawType, b: RawType) -> TestResult {
        if b.is_zero() {
            return TestResult::discard();
        }
        TestResult::from_bool(
            Integer::div_floor(&a, &b)
                == Integer::div_floor(&EncodedType::from(a), &EncodedType::from(b)).into(),
        )
    }

    #[quickcheck]
    fn test_mod_floor(a: RawType, b: RawType) -> TestResult {
        if b.is_zero() {
            return TestResult::discard();
        }
        TestResult::from_bool(
            Integer::mod_floor(&a, &b)
                == Integer::mod_floor(&EncodedType::from(a), &EncodedType::from(b)).into(),
        )
    }

    #[quickcheck]
    fn test_gcd(a: RawType, b: RawType) -> TestResult {
        if a.is_zero() || b.is_zero() {
            return TestResult::discard();
        }
        TestResult::eq(
            &Integer::gcd(&a, &b),
            &Integer::gcd(&EncodedType::from(a), &EncodedType::from(b)).into(),
        )
    }

    #[quickcheck]
    fn test_lcm(a: RawType, b: RawType) -> TestResult {
        TestResult::eq(
            &Integer::lcm(&a, &b),
            &Integer::lcm(&EncodedType::from(a), &EncodedType::from(b)).into(),
        )
    }

    #[quickcheck]
    fn test_is_multiple_of(a: RawType, b: RawType) -> TestResult {
        if b.is_zero() {
            return TestResult::discard();
        }
        TestResult::eq(
            &Integer::is_multiple_of(&a, &b),
            &Integer::is_multiple_of(&EncodedType::from(a), &EncodedType::from(b)),
        )
    }

    #[quickcheck]
    fn test_is_even(n: RawType) -> TestResult {
        TestResult::eq(&n.is_even(), &EncodedType::from(n).is_even())
    }

    #[quickcheck]
    fn test_is_odd(n: RawType) -> TestResult {
        TestResult::eq(&n.is_odd(), &EncodedType::from(n).is_odd())
    }

    #[quickcheck]
    fn test_div_rem(a: RawType, b: RawType) -> TestResult {
        if b.is_zero() {
            return TestResult::discard();
        }
        let (q, r) = Integer::div_rem(&a, &b);
        let (eq, er) = Integer::div_rem(&EncodedType::from(a), &EncodedType::from(b));
        TestResult::from_bool(q == eq.into() && r == er.into())
    }

    //
    // Roots
    //

    #[quickcheck]
    fn test_nth_root(a: RawType, degree: u8) -> TestResult {
        let degree = degree as u32 + 1;
        if a < RawType::zero() && degree.is_multiple_of(2) {
            return TestResult::discard();
        }
        TestResult::from_bool(a.nth_root(degree) == EncodedType::from(a).nth_root(degree).into())
    }

    //
    // Num
    //

    #[quickcheck]
    fn test_from_str_radix(n: RawType, radix: u8) -> bool {
        let radix = (radix % 35) as u32 + 2;
        let s = n.to_str_radix(radix);
        RawType::from_str_radix(&s, radix).ok()
            == EncodedType::from_str_radix(&s, radix).ok().map(Into::into)
    }

    #[test]
    fn test_one() {
        assert_eq!(EncodedType::one(), EncodedType::from(RawType::one()));
    }

    #[quickcheck]
    fn test_is_one(n: RawType) {
        assert!(EncodedType::one().is_one());
        assert_eq!(&n.is_one(), &EncodedType::from(n.clone()).is_one());
        assert_eq!(&n.is_one(), &EncodedType::from(n).is_one());
    }

    #[test]
    fn test_zero() {
        assert_eq!(EncodedType::zero(), EncodedType::from(RawType::zero()));
    }

    #[quickcheck]
    fn test_is_zero(n: RawType) {
        assert!(EncodedType::zero().is_zero());
        assert_eq!(n.is_zero(), EncodedType::from(n.clone()).is_zero());
        assert_eq!(n.is_zero(), EncodedType::from(n).is_zero());
    }

    //
    // Serialize
    //

    #[quickcheck]
    fn test_serialize(n: RawType) -> TestResult {
        TestResult::eq(
            &serde_json::to_string(&EncodedType::from(n.clone())).ok(),
            &serde_json::to_string(&n).ok(),
        )
    }

    //
    // ToBytes
    //

    #[quickcheck]
    fn test_to_be_bytes(n: RawType) -> TestResult {
        TestResult::eq(
            &ToBytes::to_be_bytes(&n),
            &ToBytes::to_be_bytes(&EncodedType::from(n)),
        )
    }

    #[quickcheck]
    fn test_to_le_bytes(n: RawType) -> TestResult {
        TestResult::eq(
            &ToBytes::to_le_bytes(&n),
            &ToBytes::to_le_bytes(&EncodedType::from(n)),
        )
    }

    //
    // FromBytes
    //

    #[quickcheck]
    fn test_from_be_bytes(bytes: Vec<u8>) -> TestResult {
        TestResult::eq(
            &RawType::from_be_bytes(&bytes),
            &RawType::from(EncodedType::from_be_bytes(&bytes)),
        )
    }

    #[quickcheck]
    fn test_from_le_bytes(bytes: Vec<u8>) -> TestResult {
        TestResult::eq(
            &RawType::from_le_bytes(&bytes),
            &RawType::from(EncodedType::from_le_bytes(&bytes)),
        )
    }

    //
    // Ord
    //

    #[quickcheck]
    fn test_cmp(a: RawType, b: RawType) -> TestResult {
        TestResult::eq(&a.cmp(&b), &EncodedType::from(a).cmp(&EncodedType::from(b)))
    }

    //
    // FromPrimitive
    //

    duplicate_prims! { paste! {
        #[quickcheck]
        fn [<test_from_ prim>](n: prim) -> TestResult {
            TestResult::eq(
                &EncodedType::[<from_ prim>](n).map(RawType::from),
                &RawType::[<from_ prim>](n),
            )
        }
    } }

    //
    // ToPrimitive
    //

    duplicate_prims! { paste! {
        #[quickcheck]
        fn [<test_to_ prim>](n: RawType) -> TestResult {
            TestResult::eq(
                &ToPrimitive::[<to_ prim>](&n),
                &ToPrimitive::[<to_ prim>](&EncodedType::from(n)),
            )
        }
    } }
} }

#[duplicate_item(
    mod_name         EncodedType;
    [signed_cow_ops] [CowBigInt];
    [signed_rc_ops]  [RcBigInt];
)]
mod mod_name {
    use quickcheck_macros::quickcheck;

    use super::*;

    //
    // CheckedEuclid
    //

    #[quickcheck]
    fn test_checked_div_euclid(a: BigInt, b: BigInt) -> TestResult {
        TestResult::from_bool(
            CheckedEuclid::checked_div_euclid(&a, &b)
                == CheckedEuclid::checked_div_euclid(&EncodedType::from(a), &EncodedType::from(b))
                    .map(Into::into),
        )
    }

    #[quickcheck]
    fn test_checked_rem_euclid(a: BigInt, b: BigInt) -> TestResult {
        TestResult::from_bool(
            CheckedEuclid::checked_rem_euclid(&a, &b)
                == CheckedEuclid::checked_rem_euclid(&EncodedType::from(a), &EncodedType::from(b))
                    .map(Into::into),
        )
    }

    //
    // Euclid
    //

    #[quickcheck]
    fn test_div_euclid(a: BigInt, b: BigInt) -> TestResult {
        if b.is_zero() {
            return TestResult::discard();
        }
        TestResult::from_bool(
            Euclid::div_euclid(&a, &b)
                == Euclid::div_euclid(&EncodedType::from(a), &EncodedType::from(b)).into(),
        )
    }

    #[quickcheck]
    fn test_rem_euclid(a: BigInt, b: BigInt) -> TestResult {
        if b.is_zero() {
            return TestResult::discard();
        }
        TestResult::from_bool(
            Euclid::rem_euclid(&a, &b)
                == Euclid::rem_euclid(&EncodedType::from(a), &EncodedType::from(b)).into(),
        )
    }

    //
    // Neg
    //

    #[quickcheck]
    fn test_neg(n: BigInt) -> bool {
        -n.clone() == (-EncodedType::from(n)).into()
    }

    //
    // Not
    //

    #[quickcheck]
    fn test_not(n: BigInt) -> bool {
        !n.clone() == (!EncodedType::from(n)).into()
    }

    //
    // Signed
    //

    #[quickcheck]
    fn test_abs(n: BigInt) -> bool {
        Signed::abs(&n) == Signed::abs(&EncodedType::from(n)).into()
    }

    #[quickcheck]
    fn test_signum(n: BigInt) -> bool {
        Signed::signum(&n) == Signed::signum(&EncodedType::from(n)).into()
    }

    #[quickcheck]
    fn test_is_positive(n: BigInt) -> bool {
        Signed::is_positive(&n) == Signed::is_positive(&EncodedType::from(n))
    }

    #[quickcheck]
    fn test_is_negative(n: BigInt) -> bool {
        Signed::is_negative(&n) == Signed::is_negative(&EncodedType::from(n))
    }
}
