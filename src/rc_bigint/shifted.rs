use crate::rc_bigint::small_num::SmallNum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shifted<S>(S);

impl<S> Shifted<S>
where
    S: SmallNum,
{
    pub const fn try_new(s: S) -> Option<Self> {
        s.checked_shl(1).map(|shifted| Shifted(shifted | S::one()))
    }

    pub const fn validate(self) -> Option<S> {
        if self.0 & S::one() == S::one() {
            Some(self.0 >> 1)
        } else {
            None
        }
    }
}
