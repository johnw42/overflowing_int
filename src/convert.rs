use crate::encoding::{Decoded, Encode, Encoding};
use crate::signed::GenericSignedBigNum;
use crate::small_num::SmallNumber;
use crate::unsigned::GenericUnsignedBigNum;
use duplicate::duplicate_item;
use num_bigint::{BigInt, BigUint, Sign, ToBigInt, ToBigUint};
use num_traits::ToPrimitive;
use paste::paste;
use std::borrow::Cow;
use std::convert::TryFrom;

#[duplicate_item(
    mod_name  BigNumType GenericBigNumWrapper signedness;
    [bigint]  [BigInt]   [GenericSignedBigNum]      [signed];
    [biguint] [BigUint]  [GenericUnsignedBigNum]     [unsigned];
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
            self.0.with_decoded(|decoded| match decoded {
                Decoded::Small(small) => Some(small.to_bigint()),
                Decoded::Big(big) => big.to_bigint(),
            })
        }
    }

    impl<'a, E: Encoding<'a, Big = BigNumType>> ToBigUint for GenericBigNumWrapper<'a, E> {
        fn to_biguint(&self) -> Option<BigUint> {
            self.0.with_decoded(|decoded| match decoded {
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
                    value.0.clone().with_decoded(|decoded| match decoded {
                        Decoded::Small(n) => n.[<to_ prim>]().ok_or(()),
                        Decoded::Big(n) => n.[<to_ prim>]().ok_or(()),
                    })
                }
            }
        }
    }
}

impl<'a, E: Encoding<'a, Big = BigInt>> From<BigUint> for GenericSignedBigNum<'a, E> {
    fn from(value: BigUint) -> Self {
        Self::from_big(BigInt::from_biguint(Sign::Plus, value))
    }
}
