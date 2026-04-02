pub trait EncodedBigNum<'a>
where
    Self: Sized + Clone + From<Encoding<Self::Small, Self::BigEncoding>>,
{
    type Small: SmallNum;
    type Big: Clone;
    type BigEncoding: Clone;

    const ZERO: Self;
    const ONE: Self;

    fn from_small(s: Self::Small) -> Self;
    fn from_big(b: Self::Big) -> Self;
    fn from_big_cow(b: Cow<'a, Self::Big>) -> Self;
    fn decode(self) -> Encoding<Self::Small, Self::BigEncoding>;
    fn decode_ref(&self) -> Encoding<Self::Small, &Self::Big>;
    fn small(&self) -> Option<Self::Small>;
    fn big_ref(&self) -> Option<&Self::Big>;
    fn big_cow<'b>(&self) -> Cow<'b, Self::Big>;
    fn update_encoding(&mut self, f: impl FnOnce(&mut Encoding<Self::Small, Self::BigEncoding>));
}

pub enum Encoding<S, T> {
    Small(S),
    Big(T),
}
