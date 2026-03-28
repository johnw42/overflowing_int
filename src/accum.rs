use std::mem::size_of;

use num_bigint::Sign::{self, *};

use crate::{Digit, Udigit};

pub fn make_accum(value: Digit) -> (Sign, Udigit) {
    if value == 0 {
        (NoSign, 0)
    } else if value >= 0 {
        (Plus, value as Udigit)
    } else {
        (Minus, value.unsigned_abs() as Udigit)
    }
}

pub fn accum_to_digit(sign: Sign, accum: Udigit) -> Option<Digit> {
    let accum = accum as Digit;
    if accum >= 0 {
        Some(match sign {
            Plus => accum,
            Minus => -accum,
            NoSign => 0,
        })
    } else {
        None
    }
}

pub fn accum_be(bytes: &[u8]) -> Option<Udigit> {
    if bytes.len() <= size_of::<Digit>() {
        let mut accum = 0;
        for &byte in bytes {
            accum = accum << 8 | byte as Udigit;
        }
        Some(accum)
    } else {
        None
    }
}

pub fn accum_le(bytes: &[u8]) -> Option<Udigit> {
    if bytes.len() <= size_of::<Digit>() {
        let mut accum = 0;
        for (i, &byte) in bytes.iter().enumerate() {
            accum |= (byte as Udigit) << (8 * i);
        }
        Some(accum)
    } else {
        None
    }
}
