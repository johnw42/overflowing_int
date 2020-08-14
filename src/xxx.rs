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
    let s = || S::Small(0);
    let b = || S::Big(BigInt::zero());

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
