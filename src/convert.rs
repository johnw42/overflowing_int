use crate::generic_bigint::GenericBigInt;
use crate::generic_bignum::encoding::{Decode, Decoded, Encoding};
use crate::generic_biguint::GenericBigUint;
use crate::small_num::SmallNumber;
use duplicate::duplicate_item;
use num_bigint::{BigInt, BigUint, Sign, ToBigInt, ToBigUint};
use num_traits::ToPrimitive;
use paste::paste;
use serde::de;
use std::borrow::Cow;
use std::convert::TryFrom;

#[duplicate_item(
    mod_name  BigNumType GenericBigNumWrapper signedness;
    [bigint]  [BigInt]   [GenericBigInt]      [signed];
    [biguint] [BigUint]  [GenericBigUint]     [unsigned];
)]
pub mod mod_name {

    use crate::duplicate_prims_with_signedness;

    use super::*;

    impl<'a, E: Encoding<'a, Big = BigNumType>> From<&'a BigNumType> for GenericBigNumWrapper<'a, E> {
        fn from(value: &'a BigNumType) -> Self {
            Self::from_big_cow(Cow::Borrowed(value))
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> From<BigNumType> for GenericBigNumWrapper<'a, E> {
        fn from(value: BigNumType) -> Self {
            Self::from_big(value)
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> From<GenericBigNumWrapper<'a, E>> for BigNumType {
        fn from(value: GenericBigNumWrapper<'a, E>) -> Self {
            value.0.into_big()
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> From<&GenericBigNumWrapper<'a, E>> for BigNumType {
        fn from(value: &GenericBigNumWrapper<'a, E>) -> Self {
            value.clone().0.into_big()
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> From<bool> for GenericBigNumWrapper<'a, E> {
        fn from(value: bool) -> Self {
            u8::from(value).into()
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> ToBigInt for GenericBigNumWrapper<'a, E> {
        fn to_bigint(&self) -> Option<BigInt> {
            self.with_decoded(|decoded| match decoded {
                Decoded::Small(small) => Some(small.to_bigint()),
                Decoded::Big(big) => big.to_bigint(),
            })
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> ToBigUint for GenericBigNumWrapper<'a, E> {
        fn to_biguint(&self) -> Option<BigUint> {
            self.with_decoded(|decoded| match decoded {
                Decoded::Small(small) => small.to_biguint(),
                Decoded::Big(big) => big.to_biguint(),
            })
        }
    }

    duplicate_prims_with_signedness! { signedness;
        paste! {
            impl<'a, E: Encoding<'a, Big=BigNumType>> From<prim> for GenericBigNumWrapper<'a, E> {
                fn from(value: prim) -> Self {
                    #[allow(irrefutable_let_patterns)]
                    #[allow(clippy::unnecessary_fallible_conversions)]
                    if let Ok(n) = E::Small::try_from(value) {
                        Self::from_small(n)
                    } else {
                        Self::from_big(E::Big::from(value))
                    }
                }
            }

            impl<'a, E: Encoding<'a, Big=BigNumType>> TryFrom<GenericBigNumWrapper<'a, E>> for prim {
                type Error = ();

                fn try_from(value: GenericBigNumWrapper<'a, E>) -> Result<Self, Self::Error> {
                    prim::try_from(&value)
                }
            }

            impl<'a, E: Encoding<'a, Big=BigNumType>> TryFrom<&GenericBigNumWrapper<'a, E>> for prim {
                type Error = ();
                fn try_from(value: &GenericBigNumWrapper<'a, E>) -> Result<Self, Self::Error> {
                    value.0.0.clone().with_decoded(|decoded| match decoded {
                        Decoded::Small(n) => n.[<to_ prim>]().ok_or(()),
                        Decoded::Big(n) => n.[<to_ prim>]().ok_or(()),
                    })
                }
            }
        }
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> From<BigUint> for GenericBigInt<'a, E> {
    fn from(value: BigUint) -> Self {
        Self::from_big(BigInt::from_biguint(Sign::Plus, value))
    }
}

// impl<'a, E: Encoding<'a, Big = BigInt>> TryFrom<GenericBigInt<'a, E>> for BigUint {
//     type Error = ();

//     fn try_from(value: GenericBigInt<'a, E>) -> Result<Self, Self::Error> {
//         match value.0.0.decode() {
//             Decoded::Small(n) => n.try_into().map_err(|_| ()),
//             Decoded::Big(n) => n.into_owned().try_into().map_err(|_| ()),
//         }
//     }
// }

// impl<'a, E: Encoding<'a, Big = BigInt>> TryFrom<&GenericBigInt<'a, E>> for BigUint {
//     type Error = ();

//     fn try_from(value: &GenericBigInt<'a, E>) -> Result<Self, Self::Error> {
//         value.clone().try_into()
//     }
// }
