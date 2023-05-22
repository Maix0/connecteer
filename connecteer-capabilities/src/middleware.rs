use crate::between_yields_lifetime;
use crate::connection::Connection;
use crate::gen_utils::UnsafeHigherRankGenerator;
use core::ops::{Generator, GeneratorState};
use serde::{de::DeserializeOwned, Serialize};

/// This is a trait that prevent an "outsider" to call some methods on trait, while still allowing
/// you to implement those traits
pub trait PublicUncallable: crate::sealed::PublicUncallableSealed {}

impl PublicUncallable for crate::sealed::PublicUncallable {}

pub trait Middleware<Payload: Serialize + DeserializeOwned> {
    /// This is the message type that is outputted by the middleware when sending messages (and
    /// inputted when receiving messages)
    type Message: Serialize + DeserializeOwned;
    /// The error type returned when wrapping an [`Message`](Self::Message)
    type WrapError;
    /// The error type returned when unwrapping an [`Message`](Self::Message) and also provide a
    /// way to "passthrough" errors made by middleware after them
    type UnwrapError;

    type Ctx: Unpin;
    type Next: Connection<Self::Message>;

    type WrapGen: for<'s, 'c> Generator<
            (&'s mut Self, &'c mut Self::Ctx),
            Yield = Result<Self::Message, Self::WrapError>,
            Return = (),
        > + 'static;
    type UnwrapGen: for<'s, 'c> Generator<
            (&'s mut Self, &'c mut Self::Ctx),
            Yield = Result<Payload, Self::UnwrapError>,
            Return = (),
        > + 'static;
    /// Transform an [`Message`](Self::Message) into a Unwrapped `Payload`
    fn wrap<Uncallable: PublicUncallable>(msg: Payload) -> Self::WrapGen;

    /// Transform an `Payload` into a Wrapped [`Message`](Self::Message)
    fn unwrap<Uncallable: PublicUncallable>(msg: Self::Message) -> Self::UnwrapGen;

    /// This function allows the system to bubble-up an error when wrapping a [`Message`](Self::Message)
    fn create_wrap_error<Uncallable: PublicUncallable>(
        &mut self,
        err: <Self::Next as Connection<Self::Message>>::SendError,
    ) -> Self::WrapError;

    /// This function allows the system to create an error when unwrapping a [`Message`](Self::Message)
    fn create_unwrap_error<Uncallable: PublicUncallable>(
        &mut self,
        err: <Self::Next as Connection<Self::Message>>::ReceiveError,
    ) -> Self::UnwrapError;

    fn get_next<Uncallable: PublicUncallable>(&mut self) -> &mut Self::Next;

    fn get_next_ctx<Uncallable: PublicUncallable>(
        c: &mut Self::Ctx,
    ) -> &mut <Self::Next as Connection<Self::Message>>::Ctx;
}

impl<M: Middleware<Payload>, Payload: Serialize + DeserializeOwned + 'static>
    crate::sealed::Sealed<Payload> for M
{
}

impl<M: Middleware<Payload> + 'static, Payload: Serialize + DeserializeOwned + 'static>
    Connection<Payload> for M
{
    type Ctx = M::Ctx;
    type Wrapped = <M::Next as Connection<M::Message>>::Wrapped;

    type SendError = M::WrapError;
    type ReceiveError = M::UnwrapError;
    type NextError = <M::Next as Connection<M::Message>>::ReceiveError;

    type SendGen = impl for<'s, 'c> Generator<
            (&'s mut Self, &'c mut Self::Ctx),
            Yield = Result<Self::Wrapped, Self::SendError>,
            Return = (),
        > + 'static;

    type ReceiveGen = impl for<'s, 'c> Generator<
            (&'s mut Self, &'c mut Self::Ctx),
            Yield = Result<Payload, Self::ReceiveError>,
            Return = (),
        > + 'static;

    #[allow(clippy::no_effect_underscore_binding)]
    fn send(input: Payload, _: crate::sealed::PublicUncallable) -> Self::SendGen {
        let gen = static move |(s, ctx): (&'_ mut Self, &'_ mut Self::Ctx)| {
            between_yields_lifetime!(as lt);
            let (mut s, mut ctx) = (lt.adjust(s), lt.adjust(ctx));

            let mut gen_ptr = ::core::pin::pin!(M::wrap::<crate::sealed::PublicUncallable>(input));
            let _pin = core::marker::PhantomPinned;

            loop {
                match gen_ptr.as_mut().resume((s, ctx)) {
                    GeneratorState::Yielded(Ok(v)) => {
                        let mut ret =
                            core::pin::pin!(<M::Next>::send(v, crate::sealed::PublicUncallable));
                        let mut ret = ret.as_mut();
                        let mut next = s.get_next::<crate::sealed::PublicUncallable>();
                        while let GeneratorState::Yielded(v) = ret.as_mut().resume((
                            next,
                            M::get_next_ctx::<crate::sealed::PublicUncallable>(ctx),
                        )) {
                            let y = v.map_err(|e| {
                                s.create_wrap_error::<crate::sealed::PublicUncallable>(e)
                            });

                            (s, ctx) = yield_!(y);
                            next = s.get_next::<crate::sealed::PublicUncallable>();
                        }
                        continue;
                    }
                    GeneratorState::Yielded(Err(e)) => {
                        (s, ctx) = yield_!(Err(e));
                    }
                    GeneratorState::Complete(()) => return,
                };
            }
        };
        unsafe { UnsafeHigherRankGenerator::new(gen) }
    }

    #[allow(clippy::no_effect_underscore_binding)]
    fn receive(output: Self::Wrapped, _: crate::sealed::PublicUncallable) -> Self::ReceiveGen where
    {
        let gen = static move |(s, ctx): (&'_ mut Self, &'_ mut Self::Ctx)| {
            between_yields_lifetime!(as lt);
            let (mut s, mut ctx) = (lt.adjust(s), lt.adjust(ctx));
            let mut s_ptr = s as *mut Self;
            let mut ctx_ptr = ctx as *mut Self::Ctx;

            let mut next = M::get_next::<crate::sealed::PublicUncallable>(s);
            let mut next_ctx = M::get_next_ctx::<crate::sealed::PublicUncallable>(ctx);
            let mut gen_ptr =
                core::pin::pin!(<M::Next>::receive(output, crate::sealed::PublicUncallable));
            let _pin = core::marker::PhantomPinned;

            loop {
                match gen_ptr.as_mut().resume((next, next_ctx)) {
                    GeneratorState::Yielded(Ok(v)) => {
                        let mut ret =
                            core::pin::pin!(M::unwrap::<crate::sealed::PublicUncallable>(v));
                        // FROM HERE UNTIL THE END OF THE BLOCK, YOU AREN'T ALLOWED TO USE EITHER
                        // `next` or `next_ctx`
                        while let GeneratorState::Yielded(v) = ret
                            .as_mut()
                            .resume((unsafe { &mut *s_ptr }, unsafe { &mut *ctx_ptr }))
                        {
                            (s, ctx) = yield_!(v);

                            s_ptr = s as _;
                            ctx_ptr = ctx as _;
                            next = M::get_next::<crate::sealed::PublicUncallable>(s);
                            next_ctx = M::get_next_ctx::<crate::sealed::PublicUncallable>(ctx);
                        }
                    }
                    GeneratorState::Yielded(Err(e)) => {
                        (s, ctx) =
                            yield_!(Err(unsafe { &mut *s_ptr }
                                .create_unwrap_error::<crate::sealed::PublicUncallable>(e)));
                        s_ptr = s as _;
                        ctx_ptr = ctx as _;
                        next = M::get_next::<crate::sealed::PublicUncallable>(s);
                        next_ctx = M::get_next_ctx::<crate::sealed::PublicUncallable>(ctx);
                    }
                    GeneratorState::Complete(()) => return,
                };
            }
        };
        unsafe { UnsafeHigherRankGenerator::new(gen) }
    }
}

/*
*
*/
