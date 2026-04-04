use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

use num_bigint::{BigUint, Sign};
use num_integer::Roots;
use num_traits::ConstZero;

use crate::bignum_encoding::EncodedBigNum;
use crate::small_num::SmallNum;

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    use super::*;

    /// An arbitrary `Sign` for testing purposes.  Usually not `NoSign` to avoid generating too many zeroes.
    #[derive(Clone, Debug)]
    struct ArbSign(Sign);

    type RcBigInt = GenericBigNum<'static, E>;

    impl Arbitrary for ArbSign {
        fn arbitrary(g: &mut Gen) -> Self {
            ArbSign(match u8::arbitrary(g).cmp(&0) {
                Ordering::Less => Sign::Minus,
                Ordering::Equal => Sign::NoSign,
                Ordering::Greater => Sign::Plus,
            })
        }
    }

    #[quickcheck]
    fn prop_new_matches_bigint(sign: ArbSign, digits: Vec<u32>) -> bool {
        E::Big::from(GenericBigNum::new(sign.0, digits.clone())) == E::Big::new(sign.0, digits)
    }

    #[quickcheck]
    fn prop_from_slice_matches_bigint(sign: ArbSign, digits: Vec<u32>) -> bool {
        E::Big::from(GenericBigNum::from_slice(sign.0, &digits))
            == E::Big::from_slice(sign.0, &digits)
    }

    #[quickcheck]
    fn prop_from_bytes_be_matches_bigint(sign: ArbSign, bytes: Vec<u8>) -> bool {
        E::Big::from(GenericBigNum::from_bytes_be(sign.0, &bytes))
            == E::Big::from_bytes_be(sign.0, &bytes)
    }

    #[quickcheck]
    fn prop_from_bytes_le_matches_bigint(sign: ArbSign, bytes: Vec<u8>) -> bool {
        E::Big::from(GenericBigNum::from_bytes_le(sign.0, &bytes))
            == E::Big::from_bytes_le(sign.0, &bytes)
    }

    #[quickcheck]
    fn prop_from_signed_bytes_be_matches_bigint(bytes: Vec<u8>) {
        assert_eq!(
            E::Big::from(GenericBigNum::from_signed_bytes_be(&bytes)),
            E::Big::from_signed_bytes_be(&bytes)
        );
    }

    #[quickcheck]
    fn prop_from_signed_bytes_le_matches_bigint(bytes: Vec<u8>) -> bool {
        E::Big::from(GenericBigNum::from_signed_bytes_le(&bytes))
            == E::Big::from_signed_bytes_le(&bytes)
    }

    #[quickcheck]
    fn prop_to_bytes_be_matches_bigint(n: GenericBigNum) -> bool {
        E::Big::from(&n).to_bytes_be() == n.to_bytes_be()
    }

    #[quickcheck]
    fn prop_to_bytes_le_matches_bigint(n: GenericBigNum) -> bool {
        E::Big::from(&n).to_bytes_le() == n.to_bytes_le()
    }

    #[quickcheck]
    fn prop_to_signed_bytes_be_matches_bigint(n: GenericBigNum) -> bool {
        E::Big::from(&n).to_signed_bytes_be() == n.to_signed_bytes_be()
    }

    #[quickcheck]
    fn prop_to_signed_bytes_le_matches_bigint(n: GenericBigNum) -> bool {
        E::Big::from(&n).to_signed_bytes_le() == n.to_signed_bytes_le()
    }

    #[quickcheck]
    fn prop_sign_matches_bigint(n: GenericBigNum) -> bool {
        E::Big::from(&n).sign() == n.sign()
    }

    #[quickcheck]
    fn prop_bit_matches_bigint(n: GenericBigNum, bit: u64) {
        let bit = bit % 1024;
        assert_eq!(E::Big::from(n.clone()).bit(bit), n.bit(bit));
    }

    #[quickcheck]
    fn prop_bits_matches_bigint(n: GenericBigNum) -> bool {
        E::Big::from(&n).bits() == n.bits()
    }

    #[quickcheck]
    fn prop_to_biguint_matches_bigint(n: GenericBigNum) -> bool {
        E::Big::from(&n).to_biguint() == n.to_biguint()
    }

    #[quickcheck]
    fn prop_checked_add_matches_bigint(n1: GenericBigNum, n2: GenericBigNum) -> bool {
        n1.checked_add(&n2).into() == E::Big::from(&n1).checked_add(&E::Big::from(&n2))
    }

    #[quickcheck]
    fn prop_checked_sub_matches_bigint(n1: GenericBigNum, n2: GenericBigNum) -> bool {
        n1.checked_sub(&n2).into() == E::Big::from(&n1).checked_sub(&E::Big::from(&n2))
    }

    #[quickcheck]
    fn prop_checked_mul_matches_bigint(n1: GenericBigNum, n2: GenericBigNum) -> bool {
        n1.checked_mul(&n2).into() == E::Big::from(&n1).checked_mul(&E::Big::from(&n2))
    }

    #[quickcheck]
    fn prop_checked_div_matches_bigint(n1: GenericBigNum, n2: GenericBigNum) -> bool {
        n1.checked_div(&n2).into() == E::Big::from(&n1).checked_div(&E::Big::from(&n2))
    }

    #[quickcheck]
    fn prop_pow_matches_bigint(n: GenericBigNum, exp: u32) -> bool {
        let k = exp % 16;
        E::Big::from(&n).pow(k) == n.pow(k).into()
    }

    #[quickcheck]
    fn prop_modpow_matches_bigint(
        n: GenericBigNum,
        exp: GenericBigNum,
        modulus: GenericBigNum,
    ) -> bool {
        E::Big::from(&n).modpow(&E::Big::from(&exp), &E::Big::from(&modulus))
            == n.modpow(&exp, &modulus).into()
    }

    #[quickcheck]
    fn prop_sqrt_matches_bigint(n: GenericBigNum) -> bool {
        E::Big::from(&n).sqrt() == n.sqrt().into()
    }

    #[quickcheck]
    fn prop_cbrt_matches_bigint(n: GenericBigNum) -> bool {
        E::Big::from(&n).cbrt() == n.cbrt().into()
    }

    #[quickcheck]
    fn prop_nth_root_matches_bigint(n: GenericBigNum, k: u32) -> bool {
        let k = k % 16 + 1;
        E::Big::from(&n).nth_root(k) == n.nth_root(k).into()
    }

    #[quickcheck]
    fn prop_to_u32_digits_matches_bigint(n: GenericBigNum) -> bool {
        E::Big::from(&n).to_u32_digits().eq(&mut n.to_u32_digits())
    }

    #[quickcheck]
    fn prop_to_u64_digits_matches_bigint(n: GenericBigNum) -> bool {
        E::Big::from(&n).to_u64_digits().eq(&mut n.to_u64_digits())
    }

    #[quickcheck]
    fn prop_modinv_matches_bigint(n: GenericBigNum, modulus: GenericBigNum) -> bool {
        E::Big::from(&n).modinv(&E::Big::from(&modulus)) == n.modinv(&modulus).map(E::Big::from)
    }

    #[quickcheck]
    fn prop_trailing_zeros_matches_bigint(n: GenericBigNum) -> bool {
        E::Big::from(&n).trailing_zeros() == n.trailing_zeros()
    }

    #[quickcheck]
    fn prop_set_bit_matches_bigint(mut n1: GenericBigNum, bit: u64, value: bool) -> bool {
        let bit = bit % (n1.bits() + 16);
        let mut n2 = E::Big::from(&n1);
        n1.set_bit(bit, value);
        n2.set_bit(bit, value);
        E::Big::from(n1) == n2
    }
}
