use num_bigint::BigInt;

pub use inner::Encoded;

use crate::decoded::Decoded;

#[cfg(feature = "unsafe_encoding")]
mod inner {
    use std::mem::align_of;

    use crate::{Digit, Udigit};

    use super::*;
    use crate::decoded::DecodedMut;

    pub struct Encoded(pub Udigit);

    #[cfg(feature = "unsafe_encoding")]
    impl Encoded {
        pub(super) fn is_ptr(&self) -> bool {
            self.0 & 1 == 0
        }

        pub(super) fn is_digit(&self) -> bool {
            !self.is_ptr()
        }

        pub(super) fn from_bigint(value: BigInt) -> Self {
            let ptr = Box::into_raw(Box::new(value));
            let result = Self(ptr as Udigit);
            debug_assert!(std::ptr::eq(result.ptr() as Udigit, ptr));
            result
        }

        pub(super) fn from_digit(digit: Digit) -> Option<Self> {
            let shifted = digit << 1;
            debug_assert_eq!(shifted & 1, 0);
            if shifted >> 1 == digit {
                let result = Self(shifted as Udigit | 1);
                debug_assert_eq!(result.digit(), digit);
                Some(result)
            } else {
                None
            }
        }

        pub(super) fn ptr(&self) -> *mut BigInt {
            debug_assert!(self.is_ptr());
            self.0 as *mut BigInt
        }

        pub(super) fn digit(&self) -> Digit {
            debug_assert!(self.is_digit());
            self.0 as Digit >> 1
        }

        pub fn zero() -> Encoded {
            Encoded(1)
        }

        pub fn one() -> Encoded {
            Encoded(3)
        }

        pub fn is_zero(&self) -> bool {
            self.0 == 1
        }

        pub fn is_one(&self) -> bool {
            self.0 == 3
        }

        pub fn decode(self) -> Decoded<BigInt> {
            if self.is_ptr() {
                let b = unsafe { Box::from_raw(self.ptr()) };
                std::mem::forget(self);
                Decoded::Big(*b)
            } else {
                Decoded::Digit(self.digit())
            }
        }

        pub fn decode_ref(&self) -> Decoded<&BigInt> {
            if self.is_ptr() {
                Decoded::Big(unsafe { &*self.ptr() })
            } else {
                Decoded::Digit(self.digit())
            }
        }

        pub fn decode_mut(&mut self) -> Decoded<&mut BigInt> {
            if self.is_ptr() {
                Decoded::Big(unsafe { &mut *self.ptr() })
            } else {
                Decoded::Digit(self.digit())
            }
        }

        pub fn encode(decoded: Decoded<BigInt>) -> Encoded {
            debug_assert!(align_of::<BigInt>() > 1);
            debug_assert!(align_of::<BigInt>().is_power_of_two());

            #[cfg(debug_assertions)]
            let value = BigInt::from(decoded.clone());

            let do_encode = || {
                let bigint = match decoded {
                    Decoded::Digit(digit) => {
                        if let Some(encoded) = Encoded::from_digit(digit) {
                            return encoded;
                        }
                        BigInt::from(digit)
                    }
                    Decoded::Big(x) => x,
                };
                Encoded::from_bigint(BigInt::from(bigint))
            };
            let result = do_encode();
            #[cfg(debug_assertions)]
            assert_eq!(
                value,
                BigInt::from(result.clone().decode()),
                "{:x}",
                result.0
            );
            result
        }
    }

    impl Clone for Encoded {
        fn clone(&self) -> Self {
            if self.is_ptr() {
                let payload = unsafe { (*self.ptr()).clone() };
                Encoded::from_bigint(payload)
            } else {
                Encoded(self.0)
            }
        }
    }

    impl Drop for Encoded {
        fn drop(&mut self) {
            if self.is_ptr() {
                drop(unsafe { Box::from_raw(self.ptr()) });
            }
        }
    }
}

#[cfg(not(feature = "unsafe_encoding"))]
mod inner {
    use super::*;

    #[derive(Clone)]
    pub struct Encoded(pub Decoded<BigInt>);

    impl Encoded {
        pub fn zero() -> Encoded {
            Encoded(Decoded::Digit(0))
        }

        pub fn one() -> Encoded {
            Encoded(Decoded::Digit(1))
        }

        pub fn is_zero(&self) -> bool {
            self.0 == Self::zero().0
        }

        pub fn is_one(&self) -> bool {
            self.0 == Self::one().0
        }

        pub fn decode(self) -> Decoded<BigInt> {
            self.0
        }

        pub fn decode_ref(&self) -> Decoded<&BigInt> {
            match &self.0 {
                &Decoded::Digit(n) => Decoded::Digit(n),
                Decoded::Big(n) => Decoded::Big(n),
            }
        }

        pub fn decode_mut(&mut self) -> Decoded<&mut BigInt> {
            match &mut self.0 {
                &mut Decoded::Digit(n) => Decoded::Digit(n),
                Decoded::Big(n) => Decoded::Big(n),
            }
        }

        pub fn encode(decoded: Decoded<BigInt>) -> Encoded {
            Encoded(decoded)
        }
    }
}
