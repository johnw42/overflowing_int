trait Overflowing: Sized {
    fn add(self, rhs: Self) -> (Self, bool);
    fn sub(self, rhs: Self) -> (Self, bool);
    fn mul(self, rhs: Self) -> (Self, bool);
    fn div(self, rhs: Self) -> (Self, bool);
    fn rem(self, rhs: Self) -> (Self, bool);
    // fn neg(self) -> (Self, bool);
    // fn abs(self) -> (Self, bool);
}

macro_rules! each_prim {
    [int_prim, [$prim:ident, $to_prim:ident]] => {
        impl Overflowing for $prim {
            fn add(self, rhs: Self) -> (Self, bool) { self.overflowing_add(rhs) }
            fn sub(self, rhs: Self) -> (Self, bool) { self.overflowing_sub(rhs) }
            fn mul(self, rhs: Self) -> (Self, bool) { self.overflowing_mul(rhs) }
            fn div(self, rhs: Self) -> (Self, bool) { self.overflowing_div(rhs) }
            fn rem(self, rhs: Self) -> (Self, bool) { self.overflowing_rem(rhs) }
            // fn neg(self) -> (Self, bool) { self.overflowing_neg() }
            // fn abs(self) -> (Self, bool) { self.overflowing_abs() }
        }
    };
    [$($_:tt)*] => {};
}

with_prims!(each_prim, []);
