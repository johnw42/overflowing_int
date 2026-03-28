use std::mem::size_of;

use num_bigint::Sign::{self, *};

use crate::{Digit, Udigit};

pub fn sign_and_magnitude(value: Digit) -> (Sign, Udigit) {
    match value.cmp(&0) {
        std::cmp::Ordering::Equal => (NoSign, 0),
        std::cmp::Ordering::Greater => (Plus, value as Udigit),
        std::cmp::Ordering::Less => (Minus, value.unsigned_abs()),
    }
}

pub fn try_apply_sign(sign: Sign, magnitude: Udigit) -> Option<Digit> {
    Digit::try_from(magnitude)
        .ok()
        .map(|signed_magnitude| match sign {
            Plus => signed_magnitude,
            Minus => -signed_magnitude,
            NoSign => 0,
        })
}

pub fn bytes_to_digit_be(bytes: &[u8]) -> Option<Udigit> {
    let mut buf = [0u8; size_of::<Udigit>()];
    if bytes.len() <= buf.len() {
        let start = buf.len() - bytes.len();
        buf[start..].copy_from_slice(bytes);
        Some(Udigit::from_be_bytes(buf))
    } else {
        None
    }
}

pub fn bytes_to_digit_le(bytes: &[u8]) -> Option<Udigit> {
    let mut buf = [0u8; size_of::<Udigit>()];
    if bytes.len() <= buf.len() {
        buf[0..bytes.len()].copy_from_slice(bytes);
        Some(Udigit::from_le_bytes(buf))
    } else {
        None
    }
}

#[test]
fn test_bytes_to_digit_be() {
    assert_eq!(bytes_to_digit_be(&[0x00, 0x01]), Some(0x01));
    assert_eq!(bytes_to_digit_be(&[0x01, 0x00]), Some(0x0100));
    assert_eq!(bytes_to_digit_be(&[0x12, 0x34]), Some(0x1234));
    assert_eq!(
        bytes_to_digit_be(&[0xFF; size_of::<Udigit>()]),
        Some(Udigit::MAX)
    );
    assert_eq!(bytes_to_digit_be(&[0xFF; size_of::<Udigit>() + 1]), None);
}

#[test]
fn test_bytes_to_digit_le() {
    assert_eq!(bytes_to_digit_le(&[0x01, 0x00]), Some(0x01));
    assert_eq!(bytes_to_digit_le(&[0x00, 0x01]), Some(0x0100));
    assert_eq!(bytes_to_digit_le(&[0x34, 0x12]), Some(0x1234));
    assert_eq!(
        bytes_to_digit_le(&[0xFF; size_of::<Udigit>()]),
        Some(Udigit::MAX)
    );
    assert_eq!(bytes_to_digit_le(&[0xFF; size_of::<Udigit>() + 1]), None);
}
