use crate::connection::Connection;
use crate::middleware::{Middleware, PublicUncallable};
use core::ops::{Generator, GeneratorState};
use serde::{de::DeserializeOwned, Serialize};

/// This type can be used when implementing an Middleware that doesn't modify the message, but only act with side effects
/// (for example logging message, or filtering)
#[repr(transparent)]
pub struct Wrapper<T>(pub T);

impl<T> Wrapper<T> {
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: serde::Serialize> serde::Serialize for Wrapper<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Wrapper<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Self)
    }
    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <T as serde::Deserialize>::deserialize_in_place(deserializer, &mut place.0)
    }
}
pub struct SingleGen<T>(pub Option<T>);
impl<R, T> Generator<R> for SingleGen<T> {
    type Yield = T;
    type Return = ();

    fn resume(self: core::pin::Pin<&mut Self>, _: R) -> GeneratorState<Self::Yield, Self::Return> {
        let opt = unsafe { self.map_unchecked_mut(|s| &mut s.0).get_unchecked_mut() };
        match opt.take() {
            Some(v) => GeneratorState::Yielded(v),
            None => GeneratorState::Complete(()),
        }
    }
}
pub struct Identity;

impl<Payload> crate::sealed::Sealed<Wrapper<Payload>> for Identity {}

impl<Payload: Serialize + DeserializeOwned + 'static> Connection<Wrapper<Payload>> for Identity {
    type Ctx = ();
    type Wrapped = Payload;
    type NextError = core::convert::Infallible;
    type SendError = core::convert::Infallible;
    type ReceiveError = core::convert::Infallible;

    type SendGen = SingleGen<Result<Payload, Self::SendError>>;
    type ReceiveGen = SingleGen<Result<Wrapper<Payload>, Self::ReceiveError>>;
    fn send(input: Wrapper<Payload>, _: crate::sealed::PublicUncallable) -> Self::SendGen {
        SingleGen(Some(Ok(input.into_inner())))
    }

    fn receive(output: Self::Wrapped, _: crate::sealed::PublicUncallable) -> Self::ReceiveGen {
        SingleGen(Some(Ok(Wrapper(output))))
    }
}
/// This is the "Base" of all [`Middleware`](crate::Middleware) chain.
/// This is the only way to have a middleware that doesn't ask for an `Next` middleware.
pub struct Base;

impl<Payload: DeserializeOwned + Serialize + 'static> Middleware<Payload> for Base {
    type Message = Wrapper<Payload>;

    type WrapError = core::convert::Infallible;
    type UnwrapError = core::convert::Infallible;

    type Ctx = ();
    type Next = Identity;

    type WrapGen = SingleGen<Result<Self::Message, Self::WrapError>>;
    type UnwrapGen = SingleGen<Result<Payload, Self::UnwrapError>>;

    fn wrap<Uncallable: PublicUncallable>(msg: Payload) -> Self::WrapGen {
        SingleGen(Some(Ok(Wrapper(msg))))
    }

    fn unwrap<Uncallable: PublicUncallable>(msg: Self::Message) -> Self::UnwrapGen {
        SingleGen(Some(Ok(msg.into_inner())))
    }

    fn create_wrap_error<Uncallable: PublicUncallable>(
        &mut self,
        err: core::convert::Infallible,
    ) -> Self::WrapError {
        err
    }

    fn create_unwrap_error<Uncallable: PublicUncallable>(
        &mut self,
        err: core::convert::Infallible,
    ) -> Self::UnwrapError {
        err
    }
    fn get_next_ctx<Uncallable: PublicUncallable>(
        _: &mut Self::Ctx,
    ) -> &mut <Self::Next as Connection<Self::Message>>::Ctx {
        unsafe { core::ptr::NonNull::<()>::dangling().as_mut() }
    }
    fn get_next<Uncallable: PublicUncallable>(&mut self) -> &mut Self::Next {
        unsafe { core::ptr::NonNull::<Identity>::dangling().as_mut() }
    }
}
