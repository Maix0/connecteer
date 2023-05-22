#![allow(clippy::module_name_repetitions)]
pub struct PublicUncallable;

pub trait Sealed<P> {}

pub trait PublicUncallableSealed {}

impl PublicUncallableSealed for PublicUncallable {}
