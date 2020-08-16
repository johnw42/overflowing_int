use std::mem::align_of;

use num_bigint::BigInt;

pub use inner::*;

use crate::Digit;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Decoded<T> {
    Small(Digit),
    Big(T),
}

#[cfg(feature = "unsafe_encoding")]
mod inner {
    use super::*;

    pub struct Encoded(usize);

    #[cfg(feature = "unsafe_encoding")]
    impl Encoded {
        fn is_ptr(&self) -> bool {
            self.0 & 1 == 0
        }

        fn is_digit(&self) -> bool {
            !self.is_ptr()
        }

        unsafe fn ptr(&self) -> *mut BigInt {
            debug_assert!(self.is_ptr());
            self.0 as *mut BigInt
        }

        unsafe fn digit(&self) -> Digit {
            debug_assert!(self.is_digit());
            self.0 as isize >> 1
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
            unsafe {
                if self.is_ptr() {
                    Decoded::Big(*Box::from_raw(self.ptr()))
                } else {
                    Decoded::Small(self.digit())
                }
            }
        }

        pub fn decode_ref(&self) -> Decoded<&BigInt> {
            unsafe {
                if self.is_ptr() {
                    Decoded::Big(&*self.ptr())
                } else {
                    Decoded::Small(self.digit())
                }
            }
        }

        pub fn decode_mut(&mut self) -> Decoded<&mut BigInt> {
            unsafe {
                if self.is_ptr() {
                    Decoded::Big(&mut *self.ptr())
                } else {
                    Decoded::Small(self.digit())
                }
            }
        }
    }

    impl Clone for Encoded {
        fn clone(&self) -> Self {
            unsafe {
                if self.is_ptr() {
                    Encoded(Box::into_raw(Box::new((*self.ptr()).clone())) as usize)
                } else {
                    Encoded(self.0)
                }
            }
        }
    }

    impl Drop for Encoded {
        fn drop(&mut self) {
            if self.is_ptr() {
                unsafe {
                    drop(Box::from_raw(self.ptr()));
                }
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
            Encoded(Decoded::Small(0))
        }

        pub fn one() -> Encoded {
            Encoded(Decoded::Small(1))
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
                &Decoded::Small(n) => Decoded::Small(n),
                Decoded::Big(n) => Decoded::Big(n),
            }
        }

        pub fn decode_mut(&mut self) -> Decoded<&mut BigInt> {
            match &mut self.0 {
                &mut Decoded::Small(n) => Decoded::Small(n),
                Decoded::Big(n) => Decoded::Big(n),
            }
        }
    }
}

impl From<BigInt> for Decoded<BigInt> {
    fn from(x: BigInt) -> Self {
        Decoded::Big(x)
    }
}

impl From<Decoded<BigInt>> for BigInt {
    fn from(x: Decoded<BigInt>) -> Self {
        match x {
            Decoded::Small(n) => n.into(),
            Decoded::Big(n) => n,
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
//
// impl<'a> From<&'a mut Encoded> for Decoded<&'a mut BigInt> {
//     fn from(x: &'a mut Encoded) -> Self {
//         x.decode_mut()
//     }
// }

impl Decoded<BigInt> {
    #[cfg(not(feature = "unsafe_encoding"))]
    pub fn encode(self) -> Encoded {
        Encoded(self)
    }

    #[cfg(feature = "unsafe_encoding")]
    pub fn encode(self) -> Encoded {
        debug_assert!(align_of::<BigInt>() > 1);
        debug_assert!(align_of::<BigInt>().is_power_of_two());

        #[cfg(debug_assertions)]
        let value = BigInt::from(self.clone());

        let do_encode = || {
            let bigint = match self {
                Decoded::Small(value) => {
                    let shifted = value << 1;
                    if shifted >> 1 == value {
                        return Encoded(shifted as usize | 1);
                    }
                    BigInt::from(value)
                }
                Decoded::Big(x) => x,
            };
            let ptr = Box::into_raw(Box::new(BigInt::from(bigint))) as usize;
            debug_assert_eq!(ptr & 1, 0);
            Encoded(ptr)
        };
        let result = do_encode();
        #[cfg(debug_assertions)]
        assert_eq!(
            BigInt::from(result.clone().decode()),
            value,
            "{:x}",
            result.0
        );
        result
    }
}
