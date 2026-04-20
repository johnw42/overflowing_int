#![cfg(test)]
#![allow(unused_imports)]
use crate::encoding::{Decode, Decoded, Encode, Encoding};
use crate::signed::Int;
use crate::small_num::SmallNumber;
use crate::unsigned::Uint;
use crate::{CowBigInt, CowBigUint, duplicate_encoded_types, duplicate_signed_encoded_types};
use crate::{duplicate_iprims, duplicate_prims, duplicate_uprims};
use duplicate::duplicate_item;
use num_bigint::{
    BigInt, BigUint, ParseBigIntError, RandomBits, Sign, UniformBigInt, UniformBigUint,
};
use num_integer::{Integer, Roots};
use num_traits::{
    CheckedAdd, CheckedDiv, CheckedEuclid, CheckedMul, CheckedSub, ConstZero, Euclid, FromBytes,
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

duplicate_encoded_types! { mod encoding_tag {
    use super::*;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    use serde::{Deserialize, Serialize};

    //
    // Binary
    //

    #[quickcheck]
    fn test_binary(n: ImplType) -> bool {
        dbg!(format!("{:b}", n)) == dbg!(format!("{:b}", EncodedType::from(n)))
    }

    //
    // CheckedAdd
    //

    #[quickcheck]
    fn test_checked_add(a: ImplType, b: ImplType) -> bool {
        CheckedAdd::checked_add(&a, &b)
            == CheckedAdd::checked_add(&EncodedType::from(a), &EncodedType::from(b)).map(Into::into)
    }

    #[quickcheck]
    fn test_checked_div(a: ImplType, b: ImplType) -> bool {
        CheckedDiv::checked_div(&a, &b)
            == CheckedDiv::checked_div(&EncodedType::from(a), &EncodedType::from(b)).map(Into::into)
    }

    //
    // CheckedDiv
    //

    #[quickcheck]
    fn test_checked_mul(a: ImplType, b: ImplType) -> bool {
        CheckedMul::checked_mul(&a, &b)
            == CheckedMul::checked_mul(&EncodedType::from(a), &EncodedType::from(b)).map(Into::into)
    }

    //
    // CheckedSub
    //

    #[quickcheck]
    fn test_checked_sub(a: ImplType, b: ImplType) -> bool {
        CheckedSub::checked_sub(&a, &b)
            == CheckedSub::checked_sub(&EncodedType::from(a), &EncodedType::from(b)).map(Into::into)
    }

    //
    // ConstZero
    //

    #[quickcheck]
    fn test_const_zero() -> bool {
        <EncodedType as ConstZero>::ZERO.is_zero()
    }

    //
    // Debug
    //

    #[quickcheck]
    fn test_debug(n: ImplType) -> bool {
        let raw_display = format!("{:?}", n);
        let test_display = format!("{:?}", EncodedType::from(n));
        test_display.contains(&raw_display)
    }

    //
    // Deserialize
    //

    #[quickcheck]
    fn test_deserialize(n: ImplType) -> bool {
        let s = serde_json::to_string(&EncodedType::from(n.clone())).unwrap();
        serde_json::from_str::<EncodedType>(&s).ok().map(Into::into) == Some(n)
    }

    //
    // Display
    //

    #[quickcheck]
    fn test_display(n: ImplType) -> bool {
        format!("{}", n) == format!("{}", EncodedType::from(n))
    }

    //
    // LowerHex
    //

    #[quickcheck]
    fn test_lower_hex(n: ImplType) -> bool {
        format!("{:x}", n) == format!("{:x}", EncodedType::from(n))
    }

    //
    // Octal
    //

    #[quickcheck]
    fn test_octal(n: ImplType) -> bool {
        format!("{:o}", n) == format!("{:o}", EncodedType::from(n))
    }

    //
    // UpperHex
    //

    #[quickcheck]
    fn test_upper_hex(n: ImplType) -> bool {
        format!("{:X}", n) == format!("{:X}", EncodedType::from(n))
    }

    //
    // FromStr
    //

    #[quickcheck]
    fn test_from_str(n: ImplType) -> bool {
        let s = n.to_string();
        ImplType::from_str(&s).ok() == EncodedType::from_str(&s).ok().map(Into::into)
    }

    //
    // Integer
    //

    #[quickcheck]
    fn test_div_floor(a: ImplType, b: ImplType) -> TestResult {
        if b.is_zero() {
            return TestResult::discard();
        }
        TestResult::from_bool(
            Integer::div_floor(&a, &b)
                == Integer::div_floor(&EncodedType::from(a), &EncodedType::from(b)).into(),
        )
    }

    #[quickcheck]
    fn test_mod_floor(a: ImplType, b: ImplType) -> TestResult {
        if b.is_zero() {
            return TestResult::discard();
        }
        TestResult::from_bool(
            Integer::mod_floor(&a, &b)
                == Integer::mod_floor(&EncodedType::from(a), &EncodedType::from(b)).into(),
        )
    }

    #[quickcheck]
    fn test_gcd(a: ImplType, b: ImplType) -> TestResult {
        if a.is_zero() || b.is_zero() {
            return TestResult::discard();
        }
        assert_eq!(
            Integer::gcd(&a, &b),
            Integer::gcd(&EncodedType::from(a), &EncodedType::from(b)).into(),
        );
        TestResult::passed()
    }

    #[quickcheck]
    fn test_lcm(a: ImplType, b: ImplType) {
        assert_eq!(
            Integer::lcm(&a, &b),
            Integer::lcm(&EncodedType::from(a), &EncodedType::from(b)).into(),
        );
    }

    #[quickcheck]
    fn test_is_multiple_of(a: ImplType, b: ImplType) -> TestResult {
        if b.is_zero() {
            return TestResult::discard();
        }
        assert_eq!(
            Integer::is_multiple_of(&a, &b),
            Integer::is_multiple_of(&EncodedType::from(a), &EncodedType::from(b)),
        );
        TestResult::passed()
    }

    #[quickcheck]
    fn test_is_even(n: ImplType) {
        assert_eq!(n.is_even(), EncodedType::from(n).is_even());
    }

    #[quickcheck]
    fn test_is_odd(n: ImplType) {
        assert_eq!(n.is_odd(), EncodedType::from(n).is_odd());
    }

    #[quickcheck]
    fn test_div_rem(a: ImplType, b: ImplType) -> TestResult {
        if b.is_zero() {
            return TestResult::discard();
        }
        let (q, r) = Integer::div_rem(&a, &b);
        let (eq, er) = Integer::div_rem(&EncodedType::from(a), &EncodedType::from(b));
        TestResult::from_bool(q == eq.into() && r == er.into())
    }


    #[quickcheck]
    fn test_gcd_lcm(a: ImplType, b: ImplType)  {
        let (gcd, lcm) = Integer::gcd_lcm(&a, &b);
        let (actual_gcd, actual_lcm) = Integer::gcd_lcm(&EncodedType::from(a), &EncodedType::from(b));
        assert_eq!(
            (gcd, lcm),
            (actual_gcd.into(), actual_lcm.into()),
        );
    }

    //
    // Roots
    //

    #[quickcheck]
    fn test_nth_root(a: ImplType, degree: u8) -> TestResult {
        let degree = degree as u32 + 1;
        if a < ImplType::zero() && degree.is_multiple_of(2) {
            return TestResult::discard();
        }
        TestResult::from_bool(a.nth_root(degree) == EncodedType::from(a).nth_root(degree).into())
    }

    //
    // Num
    //

    #[quickcheck]
    fn test_from_str_radix(n: ImplType, radix: u8) -> bool {
        let radix = (radix % 35) as u32 + 2;
        let s = n.to_str_radix(radix);
        ImplType::from_str_radix(&s, radix).ok()
            == EncodedType::from_str_radix(&s, radix).ok().map(Into::into)
    }

    #[test]
    fn test_one() {
        assert_eq!(EncodedType::one(), EncodedType::from(ImplType::one()));
    }

    #[quickcheck]
    fn test_is_one(n: ImplType) {
        assert!(EncodedType::one().is_one());
        assert_eq!(&n.is_one(), &EncodedType::from(n.clone()).is_one());
        assert_eq!(&n.is_one(), &EncodedType::from(n).is_one());
    }

    #[test]
    fn test_zero() {
        assert_eq!(EncodedType::zero(), EncodedType::from(ImplType::zero()));
    }

    #[quickcheck]
    fn test_is_zero(n: ImplType) {
        assert!(EncodedType::zero().is_zero());
        assert_eq!(n.is_zero(), EncodedType::from(n.clone()).is_zero());
        assert_eq!(n.is_zero(), EncodedType::from(n).is_zero());
    }

    //
    // Serialize
    //

    #[quickcheck]
    fn test_serialize(n: ImplType) {
        assert_eq!(
            serde_json::to_string(&EncodedType::from(n.clone())).ok(),
            serde_json::to_string(&n).ok(),
        )
    }

    //
    // ToBytes
    //

    #[quickcheck]
    fn test_to_be_bytes(n: ImplType) {
        assert_eq!(
            ToBytes::to_be_bytes(&n),
            ToBytes::to_be_bytes(&EncodedType::from(n)),
        );
    }

    #[quickcheck]
    fn test_to_le_bytes(n: ImplType) {
        assert_eq!(
            ToBytes::to_le_bytes(&n),
            ToBytes::to_le_bytes(&EncodedType::from(n)),
        );
    }

    //
    // FromBytes
    //

    #[quickcheck]
    fn test_from_be_bytes(bytes: Vec<u8>) {
        assert_eq!(
            &ImplType::from_be_bytes(&bytes),
            &ImplType::from(EncodedType::from_be_bytes(&bytes)),
        );
    }

    #[quickcheck]
    fn test_from_le_bytes(bytes: Vec<u8>) {
        assert_eq!(
            &ImplType::from_le_bytes(&bytes),
            &ImplType::from(EncodedType::from_le_bytes(&bytes)),
        );
    }

    //
    // Ord
    //

    #[quickcheck]
    fn test_cmp(a: ImplType, b: ImplType) {
        assert_eq!(a.cmp(&b), EncodedType::from(a).cmp(&EncodedType::from(b)));
    }

    //
    // FromPrimitive
    //

    duplicate_prims! { paste! {
        #[quickcheck]
        fn [<test_from_ prim>](n: prim) {
            assert_eq!(
                &EncodedType::[<from_ prim>](n).map(ImplType::from),
                &ImplType::[<from_ prim>](n),
            );
        }
    } }

    //
    // ToPrimitive
    //

    duplicate_prims! { paste! {
        #[quickcheck]
        fn [<test_to_ prim>](n: ImplType) {
            assert_eq!(
                &ToPrimitive::[<to_ prim>](&n),
                &ToPrimitive::[<to_ prim>](&EncodedType::from(n)),
            );
        }
    } }
} }

mod signed {
    crate::duplicate_signed_encoded_types! {
    mod encoding_tag {
        use quickcheck_macros::quickcheck;

        use super::super::*;

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
    } }
}
