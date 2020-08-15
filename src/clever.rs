use num_bigint::BigInt;
use num_traits::Zero;
use std::ops::{Add, AddAssign};

type Digit = i64;

enum VarInt {
    Big(BigInt),
    Small(Digit),
}

enum SmallOr<T> {
    Big(T),
    Small(Digit),
}

trait Arg {
    type Output;
    fn arg(self) -> SmallOr<Self::Output>;
}

impl Arg for VarInt {
    type Output = BigInt;
    fn arg(self) -> SmallOr<Self::Output> {
        match self {
            VarInt::Big(x) => SmallOr::Big(x),
            VarInt::Small(x) => SmallOr::Small(x),
        }
    }
}

impl<'a> Arg for &'a VarInt {
    type Output = &'a BigInt;
    fn arg(self) -> SmallOr<Self::Output> {
        match self {
            VarInt::Big(x) => SmallOr::Big(x),
            VarInt::Small(x) => SmallOr::Small(*x),
        }
    }
}

impl Arg for Digit {
    type Output = Digit;

    fn arg(self) -> SmallOr<Self::Output> {
        SmallOr::Small(self)
    }
}

impl<'a> Arg for &'a Digit {
    type Output = &'a Digit;

    fn arg(self) -> SmallOr<Self::Output> {
        SmallOr::Small(*self)
    }
}

impl<R> Add<R> for VarInt
where
    R: Arg,
    BigInt: Add<R::Output, Output = BigInt>,
    BigInt: Add<Digit, Output = BigInt>,
    Digit: Add<R::Output, Output = BigInt>,
{
    type Output = VarInt;

    fn add(self, rhs: R) -> Self::Output {
        match (self.arg(), rhs.arg()) {
            (SmallOr::Small(x), SmallOr::Small(y)) => {
                VarInt::Small(<Digit as Add<Digit>>::add(x, y))
            }
            (SmallOr::Small(x), SmallOr::Big(y)) => VarInt::Big(x.add(y)),
            (SmallOr::Big(x), SmallOr::Small(y)) => VarInt::Big(<BigInt as Add<Digit>>::add(x, y)),
            (SmallOr::Big(x), SmallOr::Big(y)) => VarInt::Big(x.add(y)),
        }
    }
}

impl<'a, R> Add<R> for &'a VarInt
where
    R: Arg,
    &'a BigInt: Add<R::Output, Output = BigInt>,
    &'a BigInt: Add<Digit, Output = BigInt>,
    Digit: Add<R::Output, Output = BigInt>,
{
    type Output = VarInt;

    fn add(self, rhs: R) -> Self::Output {
        match (self.arg(), rhs.arg()) {
            (SmallOr::Small(x), SmallOr::Small(y)) => {
                VarInt::Small(<Digit as Add<Digit>>::add(x, y))
            }
            (SmallOr::Small(x), SmallOr::Big(y)) => VarInt::Big(x.add(y)),
            (SmallOr::Big(x), SmallOr::Small(y)) => VarInt::Big(<&BigInt as Add<Digit>>::add(x, y)),
            (SmallOr::Big(x), SmallOr::Big(y)) => VarInt::Big(x.add(y)),
        }
    }
}

impl<R> AddAssign<R> for VarInt
where
    R: Arg,
    BigInt: AddAssign<R::Output>,
    BigInt: AddAssign<Digit>,
    for<'a> &'a BigInt: Add<R::Output, Output = BigInt>,
    Digit: Add<R::Output, Output = BigInt>,
{
    fn add_assign(&mut self, rhs: R) {
        match self {
            VarInt::Small(x) => *self = <&VarInt as Add<R>>::add(self, rhs),
            VarInt::Big(x) => match rhs.arg() {
                SmallOr::Small(y) => <BigInt as AddAssign<Digit>>::add_assign(x, y),
                SmallOr::Big(y) => x.add_assign(y),
            },
        }
    }
}

#[test]
fn test() {
    let i: Digit = 0;
    let v = || VarInt::Small(0);
    let b = || VarInt::Big(BigInt::zero());

    let _: VarInt = v() + v();
    let _: VarInt = v() + &v();
    let _: VarInt = v() + i;
    let _: VarInt = v() + &i;
    let _: VarInt = i + v();
    let _: VarInt = i + &v();
    let _: VarInt = &v() + v();
    let _: VarInt = &v() + &v();
    let _: VarInt = &v() + i;
    let _: VarInt = &v() + &i;
    let _: VarInt = &i + v();
    let _: VarInt = &i + &v();
    let _: VarInt = v() + v();
    let _: VarInt = v() + &v();
    let _: VarInt = v() + b();
    let _: VarInt = v() + &b();
    let _: VarInt = b() + v();
    let _: VarInt = b() + &v();
    let _: VarInt = b() + b();
    let _: VarInt = b() + &b();
    let _: VarInt = &v() + v();
    let _: VarInt = &v() + &v();
    let _: VarInt = &v() + b();
    let _: VarInt = &v() + &b();
    let _: VarInt = &b() + v();
    let _: VarInt = &b() + &v();
    let _: VarInt = &b() + b();
    let _: VarInt = &b() + &b();

    let mut x = b();
    x += v();
    x += &v();
    x += b();
    x += &b();
}
