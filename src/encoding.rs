use std::convert::TryFrom;
use std::mem::align_of;
use std::mem::size_of;

use num_bigint::BigInt;

use crate::Digit;

union Union {
    digit: Digit,
    size: usize,
}

const PREFER_DIGIT: bool = size_of::<Digit>() > size_of::<usize>();

impl Union {
    fn from_digit(digit: Digit) -> Union {
        if PREFER_DIGIT {
            Union { digit }
        } else {
            Union {
                size: digit as usize,
            }
        }
    }

    fn from_ptr(ptr: *mut BigInt) -> Union {
        if PREFER_DIGIT {
            Union {
                digit: ptr as Digit,
            }
        } else {
            Union { size: ptr as usize }
        }
    }

    unsafe fn digit(&self) -> Digit {
        if PREFER_DIGIT {
            self.digit
        } else {
            self.size as Digit
        }
    }

    unsafe fn ptr(&self) -> *mut BigInt {
        if PREFER_DIGIT {
            self.digit as *mut BigInt
        } else {
            self.size as *mut BigInt
        }
    }
}

pub struct Encoded(Union);

pub enum Decoded<T> {
    Small(Digit),
    Big(T),
}

impl Encoded {
    fn new<I>(value: I) -> Encoded
    where
        I: Copy,
        Digit: TryFrom<I>,
        BigInt: From<I>,
    {
        if let Ok(value) = Digit::try_from(value) {
            Decoded::Small(value).encode()
        } else {
            Decoded::Big(BigInt::from(value)).encode()
        }
    }

    fn decode(self) -> Decoded<BigInt> {
        unsafe {
            if self.0.digit() & 1 == 0 {
                let ptr = self.0.ptr();
                Decoded::Big(*Box::from_raw(ptr))
            } else {
                Decoded::Small(self.0.digit() >> 1)
            }
        }
    }

    fn decode_ref(&self) -> Decoded<&BigInt> {
        unsafe {
            if self.0.digit() & 1 == 0 {
                Decoded::Big(&*self.0.ptr())
            } else {
                Decoded::Small(self.0.digit() >> 1)
            }
        }
    }

    fn decode_mut(&mut self) -> Decoded<&mut BigInt> {
        unsafe {
            if self.0.digit() & 1 == 0 {
                Decoded::Big(&mut *self.0.ptr())
            } else {
                Decoded::Small(self.0.digit() >> 1)
            }
        }
    }
}

impl Clone for Encoded {
    fn clone(&self) -> Self {
        unsafe {
            if self.0.digit() & 1 == 0 {
                let ptr = self.0.ptr();
                Encoded(Union::from_ptr(Box::into_raw(Box::new((*ptr).clone()))))
            } else {
                Encoded(Union::from_digit(self.0.digit()))
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
    fn encode(self) -> Encoded {
        debug_assert!(align_of::<BigInt>() > 1);
        debug_assert!(align_of::<BigInt>().is_power_of_two());

        let bigint = match self {
            Decoded::Small(value) => {
                let shifted = value << 1;
                if shifted >> 1 == value {
                    return Encoded(Union::from_digit(shifted | 1));
                }
                BigInt::from(value)
            }
            Decoded::Big(x) => x,
        };
        let ptr = Box::into_raw(Box::new(BigInt::from(bigint)));
        debug_assert_eq!(ptr as usize & 1, 0);
        Encoded(Union::from_ptr(ptr))
    }
}
