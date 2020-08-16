use std::mem::align_of;
use std::mem::size_of;

use num_bigint::BigInt;

use crate::Digit;

union Bits {
    digit: Digit,
    size: usize,
}

const PREFER_DIGIT: bool = size_of::<Digit>() > size_of::<usize>();

impl Bits {
    #[inline]
    fn from_digit(digit: Digit) -> Bits {
        if PREFER_DIGIT {
            Bits { digit }
        } else {
            Bits {
                size: digit as usize,
            }
        }
    }

    #[inline]
    fn from_ptr(ptr: *mut BigInt) -> Bits {
        if PREFER_DIGIT {
            Bits {
                digit: ptr as Digit,
            }
        } else {
            Bits { size: ptr as usize }
        }
    }

    #[inline]
    fn digit(&self) -> Digit {
        unsafe {
            if PREFER_DIGIT {
                self.digit
            } else {
                self.size as Digit
            }
        }
    }

    #[inline]
    fn ptr(&self) -> *mut BigInt {
        unsafe {
            if PREFER_DIGIT {
                self.digit as *mut BigInt
            } else {
                self.size as *mut BigInt
            }
        }
    }
}

pub struct Encoded(Bits);

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Decoded<T> {
    Small(Digit),
    Big(T),
}

impl Encoded {
    // pub fn new<I>(value: I) -> Encoded
    // where
    //     I: Copy,
    //     Digit: TryFrom<I>,
    //     BigInt: From<I>,
    // {
    //     if let Ok(value) = Digit::try_from(value) {
    //         Decoded::Small(value).encode()
    //     } else {
    //         Decoded::Big(BigInt::from(value)).encode()
    //     }
    // }

    pub fn zero() -> Encoded {
        Encoded(Bits::from_digit(1))
    }

    pub fn one() -> Encoded {
        Encoded(Bits::from_digit(3))
    }

    pub fn is_zero(&self) -> bool {
        self.0.digit() == 1
    }

    pub fn is_one(&self) -> bool {
        self.0.digit() == 3
    }

    pub fn decode(self) -> Decoded<BigInt> {
        unsafe {
            if self.0.digit() & 1 == 0 {
                let ptr = self.0.ptr();
                Decoded::Big(*Box::from_raw(ptr))
            } else {
                Decoded::Small(self.0.digit() >> 1)
            }
        }
    }

    pub fn decode_ref(&self) -> Decoded<&BigInt> {
        unsafe {
            if self.0.digit() & 1 == 0 {
                Decoded::Big(&*self.0.ptr())
            } else {
                Decoded::Small(self.0.digit() >> 1)
            }
        }
    }

    pub fn decode_mut(&mut self) -> Decoded<&mut BigInt> {
        unsafe {
            if self.0.digit() & 1 == 0 {
                Decoded::Big(&mut *self.0.ptr())
            } else {
                Decoded::Small(self.0.digit() >> 1)
            }
        }
    }
}

// impl From<Encoded> for Decoded<BigInt> {
//     fn from(x: Encoded) -> Self {
//         x.decode()
//     }
// }
//
// impl<'a> From<&'a Encoded> for Decoded<&'a BigInt> {
//     fn from(x: &'a Encoded) -> Self {
//         x.decode_ref()
//     }
// }

impl Clone for Encoded {
    fn clone(&self) -> Self {
        unsafe {
            if self.0.digit() & 1 == 0 {
                let ptr = self.0.ptr();
                Encoded(Bits::from_ptr(Box::into_raw(Box::new((*ptr).clone()))))
            } else {
                Encoded(Bits::from_digit(self.0.digit()))
            }
        }
    }
}

impl Drop for Encoded {
    fn drop(&mut self) {
        unsafe {
            if self.0.digit() & 1 == 0 {
                drop(Box::from_raw(self.0.ptr()));
            }
        }
    }
}

impl Decoded<BigInt> {
    pub fn encode(self) -> Encoded {
        debug_assert!(align_of::<BigInt>() > 1);
        debug_assert!(align_of::<BigInt>().is_power_of_two());

        let bigint = match self {
            Decoded::Small(value) => {
                let shifted = value << 1;
                if shifted >> 1 == value {
                    return Encoded(Bits::from_digit(shifted | 1));
                }
                BigInt::from(value)
            }
            Decoded::Big(x) => x,
        };
        let ptr = Box::into_raw(Box::new(BigInt::from(bigint)));
        debug_assert_eq!(ptr as usize & 1, 0);
        Encoded(Bits::from_ptr(ptr))
    }
}
