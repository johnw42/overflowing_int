#[cfg(feature = "arbitrary")]
pub trait ArbitraryBounds<'a>: arbitrary::Arbitrary<'a> {}

#[cfg(feature = "arbitrary")]
impl<'a, T> ArbitraryBounds<'a> for T where T: arbitrary::Arbitrary<'a> {}

#[cfg(not(feature = "arbitrary"))]
pub trait ArbitraryBounds<'a> {}

#[cfg(not(feature = "arbitrary"))]
impl<'a, T> ArbitraryBounds<'a> for T {}

#[cfg(any(test, feature = "quickcheck"))]
pub trait QuickcheckBounds: quickcheck::Arbitrary {}

#[cfg(any(test, feature = "quickcheck"))]
impl<T> QuickcheckBounds for T where T: quickcheck::Arbitrary {}

#[cfg(not(any(test, feature = "quickcheck")))]
pub trait QuickcheckBounds {}

#[cfg(not(any(test, feature = "quickcheck")))]
impl<T> QuickcheckBounds for T {}

#[cfg(any(test, feature = "rand"))]
pub trait RandBounds: rand::distributions::uniform::SampleUniform {}

#[cfg(any(test, feature = "rand"))]
impl<T> RandBounds for T where T: rand::distributions::uniform::SampleUniform {}

#[cfg(not(any(test, feature = "rand")))]
pub trait RandBounds {}

#[cfg(not(any(test, feature = "rand")))]
impl<T> RandBounds for T {}

#[cfg(any(test, feature = "serde"))]
pub trait SerdeBounds: serde::Serialize + for<'de> serde::Deserialize<'de> {}

#[cfg(any(test, feature = "serde"))]
impl<T> SerdeBounds for T where T: serde::Serialize + for<'de> serde::Deserialize<'de> {}

#[cfg(not(any(test, feature = "serde")))]
pub trait SerdeBounds {}

#[cfg(not(any(test, feature = "serde")))]
impl<T> SerdeBounds for T {}
