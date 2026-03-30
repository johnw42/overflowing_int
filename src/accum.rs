use std::mem::size_of;

use crate::SmallUint;

pub fn bytes_to_digit_be(bytes: &[u8]) -> Option<SmallUint> {
    let mut buf = [0u8; size_of::<SmallUint>()];
    if bytes.len() <= buf.len() {
        let start = buf.len() - bytes.len();
        buf[start..].copy_from_slice(bytes);
        Some(SmallUint::from_be_bytes(buf))
    } else {
        None
    }
}

pub fn bytes_to_digit_le(bytes: &[u8]) -> Option<SmallUint> {
    let mut buf = [0u8; size_of::<SmallUint>()];
    if bytes.len() <= buf.len() {
        buf[0..bytes.len()].copy_from_slice(bytes);
        Some(SmallUint::from_le_bytes(buf))
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
        bytes_to_digit_be(&[0xFF; size_of::<SmallUint>()]),
        Some(SmallUint::MAX)
    );
    assert_eq!(bytes_to_digit_be(&[0xFF; size_of::<SmallUint>() + 1]), None);
}

#[test]
fn test_bytes_to_digit_le() {
    assert_eq!(bytes_to_digit_le(&[0x01, 0x00]), Some(0x01));
    assert_eq!(bytes_to_digit_le(&[0x00, 0x01]), Some(0x0100));
    assert_eq!(bytes_to_digit_le(&[0x34, 0x12]), Some(0x1234));
    assert_eq!(
        bytes_to_digit_le(&[0xFF; size_of::<SmallUint>()]),
        Some(SmallUint::MAX)
    );
    assert_eq!(bytes_to_digit_le(&[0xFF; size_of::<SmallUint>() + 1]), None);
}
