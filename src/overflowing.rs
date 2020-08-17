pub trait Overflowing: Sized {
    fn add(self, rhs: Self) -> (Self, bool);
    fn sub(self, rhs: Self) -> (Self, bool);
    fn mul(self, rhs: Self) -> (Self, bool);
    fn div(self, rhs: Self) -> (Self, bool);
    fn rem(self, rhs: Self) -> (Self, bool);
    fn shl(self, rhs: u32) -> (Self, bool);
    fn shr(self, rhs: u32) -> (Self, bool);
    fn bit_and(self, rhs: Self) -> (Self, bool);
    fn bit_or(self, rhs: Self) -> (Self, bool);
    fn bit_xor(self, rhs: Self) -> (Self, bool);
}

macro_rules! impl_overflowing {
    [[int $(, $_1:tt)*], [$prim:ident, $to_prim:ident]] => {
        impl Overflowing for $prim {
            fn add(self, rhs: Self) -> (Self, bool) { self.overflowing_add(rhs) }
            fn sub(self, rhs: Self) -> (Self, bool) { self.overflowing_sub(rhs) }
            fn mul(self, rhs: Self) -> (Self, bool) { self.overflowing_mul(rhs) }
            fn div(self, rhs: Self) -> (Self, bool) { self.overflowing_div(rhs) }
            fn rem(self, rhs: Self) -> (Self, bool) { self.overflowing_rem(rhs) }
            fn shl(self, rhs: u32) -> (Self, bool) { self.overflowing_shl(rhs) }
            fn shr(self, rhs: u32) -> (Self, bool) { self.overflowing_shr(rhs) }
            fn bit_and(self, rhs: Self) -> (Self, bool) { (self & rhs, false) }
            fn bit_or(self, rhs: Self) -> (Self, bool) { (self | rhs, false) }
            fn bit_xor(self, rhs: Self) -> (Self, bool) { (self ^ rhs, false) }
        }
    };
    [$($_:tt)*] => {};
}

with_prims!(impl_overflowing, []);
