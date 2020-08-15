mod ops {
    use num_bigint::BigInt;
    use num_traits::Zero;
    use std::ops::{Add, AddAssign};

    enum S {
        Big(BigInt),
        Small(i32),
    }

    enum SmallOr<T> {
        Big(T),
        Small(i32),
    }

    trait Arg {
        type Output;
        fn arg(self) -> SmallOr<Self::Output>;
    }

    impl Arg for S {
        type Output = BigInt;
        fn arg(self) -> SmallOr<Self::Output> {
            match self {
                S::Big(x) => SmallOr::Big(x),
                S::Small(x) => SmallOr::Small(x),
            }
        }
    }

    impl<'a> Arg for &'a S {
        type Output = &'a BigInt;
        fn arg(self) -> SmallOr<Self::Output> {
            match self {
                S::Big(x) => SmallOr::Big(x),
                S::Small(x) => SmallOr::Small(*x),
            }
        }
    }

    impl<R> Add<R> for S
    where
        R: Arg,
        BigInt: Add<R::Output, Output = BigInt>,
        BigInt: Add<i32, Output = BigInt>,
        i32: Add<R::Output, Output = BigInt>,
    {
        type Output = S;

        fn add(self, rhs: R) -> Self::Output {
            match (self.arg(), rhs.arg()) {
                (SmallOr::Small(x), SmallOr::Small(y)) => S::Small(<i32 as Add<i32>>::add(x, y)),
                (SmallOr::Small(x), SmallOr::Big(y)) => S::Big(x.add(y)),
                (SmallOr::Big(x), SmallOr::Small(y)) => S::Big(<BigInt as Add<i32>>::add(x, y)),
                (SmallOr::Big(x), SmallOr::Big(y)) => S::Big(x.add(y)),
            }
        }
    }

    impl<'a, R> Add<R> for &'a S
    where
        R: Arg,
        &'a BigInt: Add<R::Output, Output = BigInt>,
        &'a BigInt: Add<i32, Output = BigInt>,
        i32: Add<R::Output, Output = BigInt>,
    {
        type Output = S;

        fn add(self, rhs: R) -> Self::Output {
            match (self.arg(), rhs.arg()) {
                (SmallOr::Small(x), SmallOr::Small(y)) => S::Small(<i32 as Add<i32>>::add(x, y)),
                (SmallOr::Small(x), SmallOr::Big(y)) => S::Big(x.add(y)),
                (SmallOr::Big(x), SmallOr::Small(y)) => S::Big(<&BigInt as Add<i32>>::add(x, y)),
                (SmallOr::Big(x), SmallOr::Big(y)) => S::Big(x.add(y)),
            }
        }
    }

    impl<R> AddAssign<R> for S
    where
        R: Arg,
        BigInt: AddAssign<R::Output>,
        BigInt: AddAssign<i32>,
        for<'a> &'a BigInt: Add<R::Output, Output = BigInt>,
        i32: Add<R::Output, Output = BigInt>,
    {
        fn add_assign(&mut self, rhs: R) {
            match self {
                S::Small(x) => *self = <&S as Add<R>>::add(self, rhs),
                S::Big(x) => match rhs.arg() {
                    SmallOr::Small(y) => <BigInt as AddAssign<i32>>::add_assign(x, y),
                    SmallOr::Big(y) => x.add_assign(y),
                },
            }
        }
    }

    #[test]
    fn test() {
        let b = || S::Small(0);
        let s = || S::Big(BigInt::zero());

        let _: S = b() + b();
        let _: S = b() + &b();
        let _: S = b() + s();
        let _: S = b() + &s();
        let _: S = s() + b();
        let _: S = s() + &b();
        let _: S = s() + s();
        let _: S = s() + &s();
        let _: S = &b() + b();
        let _: S = &b() + &b();
        let _: S = &b() + s();
        let _: S = &b() + &s();
        let _: S = &s() + b();
        let _: S = &s() + &b();
        let _: S = &s() + s();
        let _: S = &s() + &s();

        let mut x = s();
        x += b();
        x += &b();
        x += s();
        x += &s();
    }
}

mod unions {
    use num_bigint::BigInt;
    use std::convert::TryFrom;
    use std::mem::align_of;

    struct S(usize);

    enum Decoded<T> {
        Small(isize),
        Big(T),
    }

    impl S {
        fn new<I>(value: I) -> S
        where
            I: Copy,
            isize: TryFrom<I>,
            BigInt: From<I>,
        {
            if let Ok(value) = isize::try_from(value) {
                Self::encode(Decoded::Small(value))
            } else {
                Self::encode(Decoded::Big(BigInt::from(value)))
            }
        }

        fn encode(value: Decoded<BigInt>) -> S {
            debug_assert!(align_of::<BigInt>() > 1);
            debug_assert!(align_of::<BigInt>().is_power_of_two());

            let bigint = match value {
                Decoded::Small(value) => {
                    let shifted = value << 1;
                    if shifted >> 1 == value {
                        return S(shifted as usize | 1);
                    }
                    BigInt::from(value)
                }
                Decoded::Big(x) => x,
            };
            let ptr = Box::into_raw(Box::new(BigInt::from(bigint))) as usize;
            debug_assert_eq!(ptr & 1, 0);
            S(ptr)
        }

        fn decode(self) -> Decoded<BigInt> {
            unsafe {
                if self.0 & 1 == 0 {
                    let ptr = self.0 as *mut BigInt;
                    Decoded::Big(*Box::from_raw(ptr))
                } else {
                    Decoded::Small(self.0 as isize >> 1)
                }
            }
        }

        fn decode_ref(&self) -> Decoded<&BigInt> {
            if self.0 & 1 == 0 {
                unsafe {
                    let ptr = self.0 as *const BigInt;
                    Decoded::Big(&*ptr)
                }
            } else {
                Decoded::Small(self.0 as isize >> 1)
            }
        }

        fn decode_mut(&mut self) -> Decoded<&mut BigInt> {
            if self.0 & 1 == 0 {
                unsafe {
                    let ptr = self.0 as *mut BigInt;
                    Decoded::Big(&mut *ptr)
                }
            } else {
                Decoded::Small(self.0 as isize >> 1)
            }
        }
    }

    impl Clone for S {
        fn clone(&self) -> Self {
            if self.0 & 1 == 0 {
                unsafe {
                    let ptr = self.0 as *const BigInt;
                    S(Box::into_raw(Box::new((*ptr).clone())) as usize)
                }
            } else {
                S(self.0)
            }
        }
    }

    impl Drop for S {
        fn drop(&mut self) {
            if self.0 & 1 == 0 {
                unsafe {
                    let ptr = self.0 as *mut BigInt;
                    drop(Box::from_raw(ptr));
                }
            }
        }
    }
}
