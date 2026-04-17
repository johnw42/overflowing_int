#![cfg(test)]

use crate::encoding::Encoding;
use num_bigint::{BigInt, BigUint, Sign};
use num_integer::Integer;
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Pow, Zero};
use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;

use crate::duplicate_encoded_types;

#[derive(Clone, Debug)]
struct AsciiRadixBytes<const SIGNED: bool> {
    bytes: Vec<u8>,
    radix: u32,
    has_sign: bool,
}

impl<const SIGNED: bool> Arbitrary for AsciiRadixBytes<SIGNED> {
    fn arbitrary(g: &mut Gen) -> Self {
        let radix = (u32::arbitrary(g) % 34) + 2;
        let mut bytes = Vec::with_capacity(g.size() + 1);
        let mut has_sign = false;
        if SIGNED {
            if bool::arbitrary(g) {
                bytes.push(u8::arbitrary(g) | b'-');
                has_sign = true;
            } else if bool::arbitrary(g) {
                bytes.push(u8::arbitrary(g) | b'+');
                has_sign = true;
            }
        }
        for _ in 0..g.size() {
            let digit_value = (u32::arbitrary(g) % radix) as u8;
            if digit_value < 10 {
                bytes.push(b'0' + digit_value);
            } else if bool::arbitrary(g) {
                bytes.push(b'a' + digit_value - 10);
            } else {
                bytes.push(b'A' + digit_value - 10);
            }
        }
        Self {
            bytes,
            radix,
            has_sign,
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let bytes = self.bytes.clone();
        let start = if self.has_sign { 1 } else { 0 };
        let has_sign = self.has_sign;
        let radix = self.radix;

        Box::new((start..bytes.len() - 1).map(move |_| {
            let mut new_bytes = bytes.clone();
            new_bytes.remove(start);
            Self {
                bytes: new_bytes,
                radix,
                has_sign,
            }
        }))
    }
}

#[derive(Clone, Debug)]
struct BinaryRadixBytes {
    bytes: Vec<u8>,
    radix: u32,
}

impl Arbitrary for BinaryRadixBytes {
    fn arbitrary(g: &mut Gen) -> Self {
        let radix = (u32::arbitrary(g) % 254) + 2;
        let bytes = (0..g.size())
            .map(|_| (u32::arbitrary(g) % radix) as u8)
            .collect::<Vec<_>>();
        Self { bytes, radix }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let mut bytes = self.bytes.clone();
        let radix = self.radix;
        Box::new((0..bytes.len()).map(move |_| {
            bytes.pop();
            Self {
                bytes: bytes.clone(),
                radix,
            }
        }))
    }
}

duplicate_encoded_types! {
mod encoding_tag {
    use super::*;

    fn to_impl(a: &EncodedType) -> ImplType {
        ImplType::from(a.clone())
    }

    #[quickcheck]
    fn test_into_static(a: EncodedType) {
        assert_eq!(a.clone(), a.into_static());
    }

    #[quickcheck]
    fn test_borrow(a: EncodedType) {
        assert_eq!(a, a.borrow());
    }

    #[test]
    fn test_zero() {
        assert!(EncodedType::ZERO.is_zero());
    }

    #[quickcheck]
    fn test_parse_bytes(radix_bytes: AsciiRadixBytes<IS_SIGNED>) {
        let expected = ImplType::parse_bytes(&radix_bytes.bytes, radix_bytes.radix);
        let actual = EncodedType::parse_bytes(&radix_bytes.bytes, radix_bytes.radix).map(ImplType::from);
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_to_u32_digits(a: EncodedType) {
        let expected = to_impl(&a).to_u32_digits();
        let actual = a.to_u32_digits();
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_to_u64_digits(a: EncodedType) {
        let expected = to_impl(&a).to_u64_digits();
        let actual = a.to_u64_digits();
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_iter_u32_digits(a: EncodedType) {
        let expected = to_impl(&a).iter_u32_digits().collect::<Vec<_>>();
        let actual = a.iter_u32_digits().collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_iter_u64_digits(a: EncodedType) {
        let expected = to_impl(&a).iter_u64_digits().collect::<Vec<_>>();
        let actual = a.iter_u64_digits().collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_to_str_radix(a: EncodedType, radix: u32) {
        let radix = (radix % 34) + 2;
        let expected = to_impl(&a).to_str_radix(radix);
        let actual = a.to_str_radix(radix);
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_bits(a: EncodedType) {
        let expected = to_impl(&a).bits();
        let actual = a.bits();
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_checked_add(a: EncodedType, b: EncodedType) {
        let expected = CheckedAdd::checked_add(&to_impl(&a), &to_impl(&b));
        let actual = CheckedAdd::checked_add(&a, &b).map(ImplType::from);
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_checked_sub(a: EncodedType, b: EncodedType) {
        let expected = CheckedSub::checked_sub(&to_impl(&a), &to_impl(&b));
        let actual = CheckedSub::checked_sub(&a, &b).map(ImplType::from);
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_checked_mul(a: EncodedType, b: EncodedType) {
        let expected = CheckedMul::checked_mul(&to_impl(&a), &to_impl(&b));
        let actual = CheckedMul::checked_mul(&a, &b).map(ImplType::from);
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_checked_div(a: EncodedType, b: EncodedType) {
        let expected = CheckedDiv::checked_div(&to_impl(&a), &to_impl(&b));
        let actual = CheckedDiv::checked_div(&a, &b).map(ImplType::from);
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_pow(a: EncodedType, exponent: u32) {
        let exponent = exponent % 16;
        let expected = to_impl(&a).pow(exponent);
        let actual = ImplType::from(a.pow(exponent));
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_modpow(a: EncodedType, exponent: EncodedType, modulus: EncodedType) -> TestResult {
        if modulus.is_zero() || exponent < EncodedType::ZERO {
            return TestResult::discard();
        }
        let exponent = exponent % EncodedType::from(16u32);
        let expected = to_impl(&a).modpow(&to_impl(&exponent), &to_impl(&modulus));
        let actual = ImplType::from(a.modpow(&exponent, &modulus));
        assert_eq!(expected, actual);
        TestResult::passed()
    }

    #[quickcheck]
    fn test_modinv(a: EncodedType, modulus: EncodedType) -> TestResult {
        if modulus.is_zero(){
            return TestResult::discard();
        }
        let expected = to_impl(&a).modinv(&to_impl(&modulus));
        let actual = a.modinv(&modulus).map(ImplType::from);
        assert_eq!(expected, actual);
        TestResult::passed()
    }

    #[quickcheck]
    fn test_sqrt(a: EncodedType) -> TestResult {
        if a < EncodedType::ZERO {
            return TestResult::discard();
        }
        let expected = to_impl(&a).sqrt();
        let actual = ImplType::from(a.sqrt());
        assert_eq!(expected, actual);
        TestResult::passed()
    }

    #[quickcheck]
    fn test_cbrt(a: EncodedType) {
        let expected = to_impl(&a).cbrt();
        let actual = ImplType::from(a.cbrt());
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_nth_root(a: EncodedType, n: u32) -> TestResult {
        // We limit n to 16 to avoid transfroming the number to 1 or -1 most of the time.
        let n = 1 + n % 16;
        if n.is_even() && a < EncodedType::ZERO {
            return TestResult::discard();
        }
        let expected = to_impl(&a).nth_root(n);
        let actual = ImplType::from(a.nth_root(n));
        assert_eq!(expected, actual);
        TestResult::passed()
    }

    #[quickcheck]
    fn test_trailing_zeros(a: EncodedType, extra_zeros: u32) {
        let a = a << (extra_zeros % 16);
        let expected = to_impl(&a).trailing_zeros();
        let actual = a.trailing_zeros();
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_bit(a: EncodedType, n: u64) {
        let n = n % (a.bits() + 16);
        let expected = to_impl(&a).bit(n);
        let actual = a.bit(n);
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_set_bit(mut a: EncodedType, n: u64, value: bool) {
        let n = n % (a.bits() + 16);
        let mut expected = to_impl(&a);
        expected.set_bit(n, value);
        a.set_bit(n, value);
        let actual = to_impl(&a);
        assert_eq!(expected, actual);
    }
} }

mod signed {
    crate::duplicate_signed_encoded_types! {
    mod encoding_tag {
        use super::super::*;

        fn to_impl(a: &EncodedType) -> ImplType {
            ImplType::from(a.clone())
        }

        type UnsignedEncodedType = <EncodedType as Encoding<'static>>::Unsigned;

        #[derive(Clone, Debug)]
        struct ArbSign(Sign);

        impl Arbitrary for ArbSign {
            fn arbitrary(g: &mut Gen) -> Self {
                // Make Sign::Zero relatively rare.
                let nonzero_weight = 5;
                let mut options = Vec::with_capacity(2 * nonzero_weight + 1);
                for _ in 0..nonzero_weight {
                    options.push(Sign::Plus);
                    options.push(Sign::Minus);
                }
                options.push(Sign::NoSign);
                ArbSign(
                    *g.choose(&options)
                    .unwrap(),
                )
            }
        }

        #[quickcheck]
        fn test_new(ArbSign(sign): ArbSign, magnitude: Vec<u32>) {
            let expected = BigInt::new(sign, magnitude.clone());
            let actual = BigInt::from(EncodedType::new(sign, magnitude));
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_from_biguint(ArbSign(sign): ArbSign, a: UnsignedEncodedType) {
            let expected = BigInt::from_biguint(sign, BigUint::from(a.clone()));
            let actual = BigInt::from(EncodedType::from_biguint(sign, a));
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_from_slice(ArbSign(sign): ArbSign, magnitude: Vec<u32>) {
            let expected = BigInt::from_slice(sign, &magnitude);
            let actual = BigInt::from(EncodedType::from_slice(sign, &magnitude));
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_assign_from_slice(ArbSign(sign): ArbSign, magnitude: Vec<u32>) {
            let mut expected = BigInt::default();
            expected.assign_from_slice(sign, &magnitude);
            let mut actual = EncodedType::default();
            actual.assign_from_slice(sign, &magnitude);
            let actual = BigInt::from(actual);
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_from_bytes_be(ArbSign(sign): ArbSign, bytes: Vec<u8>) {
            let expected = BigInt::from_bytes_be(sign, &bytes);
            let actual = BigInt::from(EncodedType::from_bytes_be(sign, &bytes));
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_from_bytes_le(ArbSign(sign): ArbSign, bytes: Vec<u8>) {
            let expected = BigInt::from_bytes_le(sign, &bytes);
            let actual = BigInt::from(EncodedType::from_bytes_le(sign, &bytes));
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_from_signed_bytes_be(digits: Vec<u8>) {
            let expected = BigInt::from_signed_bytes_be(&digits);
            let actual = BigInt::from(EncodedType::from_signed_bytes_be(&digits));
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_from_signed_bytes_le(digits: Vec<u8>) {
            let expected = BigInt::from_signed_bytes_le(&digits);
            let actual = BigInt::from(EncodedType::from_signed_bytes_le(&digits));
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_from_radix_be(ArbSign(sign): ArbSign, BinaryRadixBytes { bytes, radix }: BinaryRadixBytes) {
            let expected = BigInt::from_radix_be(sign, &bytes, radix);
            let actual = EncodedType::from_radix_be(sign, &bytes, radix).map(BigInt::from);
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_from_radix_le(ArbSign(sign): ArbSign, BinaryRadixBytes { bytes, radix }: BinaryRadixBytes) {
            let expected = BigInt::from_radix_le(sign, &bytes, radix);
            let actual = EncodedType::from_radix_le(sign, &bytes, radix).map(BigInt::from);
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_to_bytes_be(a: EncodedType) {
            let expected = to_impl(&a).to_bytes_be();
            let actual = a.to_bytes_be();
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_to_bytes_le(a: EncodedType) {
            let expected = to_impl(&a).to_bytes_le();
            let actual = a.to_bytes_le();
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_magnitude(a: EncodedType) {
            let impl_value = to_impl(&a);
            let expected = impl_value.magnitude();
            let actual = &BigUint::from(a.magnitude());
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_sign(a: EncodedType) {
            let expected = to_impl(&a).sign();
            let actual = a.sign();
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_into_parts(a: EncodedType) {
            let expected = to_impl(&a).into_parts();
            let (sign, magnitude) = a.into_parts();
            let actual = (sign, BigUint::from(magnitude));
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_to_biguint(a: EncodedType) {
            let expected = to_impl(&a).to_biguint();
            let actual = a.to_biguint().map(BigUint::from);
            assert_eq!(expected, actual);
        }
    } }
}

mod unsigned {
    crate::duplicate_unsigned_encoded_types! {
    mod encoding_tag {
        use super::super::*;

        fn to_impl(a: &EncodedType) -> ImplType {
            ImplType::from(a.clone())
        }

        #[quickcheck]
        fn test_new(magnitude: Vec<u32>) {
            let expected = BigUint::new(magnitude.clone());
            let actual = BigUint::from(EncodedType::new(magnitude));
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_from_slice(magnitude: Vec<u32>) {
            let expected = BigUint::from_slice(&magnitude);
            let actual = BigUint::from(EncodedType::from_slice(&magnitude));
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_assign_from_slice(magnitude: Vec<u32>) {
            let mut expected = BigUint::default();
            expected.assign_from_slice(&magnitude);
            let mut actual = EncodedType::default();
            actual.assign_from_slice(&magnitude);
            let actual = BigUint::from(actual);
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_from_bytes_be(bytes: Vec<u8>) {
            let expected = BigUint::from_bytes_be(&bytes);
            let actual = BigUint::from(EncodedType::from_bytes_be(&bytes));
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_from_bytes_le(bytes: Vec<u8>) {
            let expected = BigUint::from_bytes_le(&bytes);
            let actual = BigUint::from(EncodedType::from_bytes_le(&bytes));
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_from_radix_be(BinaryRadixBytes { bytes, radix }: BinaryRadixBytes) {
            let expected = BigUint::from_radix_be(&bytes, radix);
            let actual = EncodedType::from_radix_be(&bytes, radix).map(BigUint::from);
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_from_radix_le(BinaryRadixBytes { bytes, radix }: BinaryRadixBytes) {
            let expected = BigUint::from_radix_le(&bytes, radix);
            let actual = EncodedType::from_radix_le(&bytes, radix).map(BigUint::from);
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_to_bytes_be(a: EncodedType) {
            let expected = to_impl(&a).to_bytes_be();
            let actual = a.to_bytes_be();
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_to_bytes_le(a: EncodedType) {
            let expected = to_impl(&a).to_bytes_le();
            let actual = a.to_bytes_le();
            assert_eq!(expected, actual);
        }


        #[quickcheck]
        fn test_trailing_ones(a: EncodedType, extra_ones: u32) {
            let a = a | EncodedType::from((1u32 << (extra_ones % 16)) - 1u32);
            let expected = to_impl(&a).trailing_ones();
            let actual = a.trailing_ones();
            assert_eq!(expected, actual);
        }

        #[quickcheck]
        fn test_count_ones(a: EncodedType) {
            let expected = to_impl(&a).count_ones();
            let actual = a.count_ones();
            assert_eq!(expected, actual);
        }
    } }
}
