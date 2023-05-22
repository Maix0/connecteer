use crate::connection::Connection;
use crate::sealed::PublicUncallable;
use core::ops::{Generator, GeneratorState};
use serde::{de::DeserializeOwned, Serialize};

struct StructGen<'pipeline, G, Ctx, Con> {
    ctx: &'pipeline mut Ctx,
    con: &'pipeline mut Con,
    gen: G,
    _pin: core::marker::PhantomPinned,
}

impl<'pipeline, G, Ctx: Unpin, Con: Unpin> Generator<()> for StructGen<'pipeline, G, Ctx, Con>
where
    G: Generator<(&'pipeline mut Con, &'pipeline mut Ctx)>,
{
    type Yield = <G as Generator<(&'pipeline mut Con, &'pipeline mut Ctx)>>::Yield;
    type Return = <G as Generator<(&'pipeline mut Con, &'pipeline mut Ctx)>>::Return;

    fn resume(self: core::pin::Pin<&mut Self>, _: ()) -> GeneratorState<Self::Yield, Self::Return> {
        let self_ptr = unsafe { self.get_unchecked_mut() };
        let gen =
            unsafe { core::pin::Pin::new_unchecked(&mut *core::ptr::addr_of_mut!(self_ptr.gen)) };
        let ctx =
            unsafe { core::pin::Pin::new_unchecked(&mut *core::ptr::addr_of_mut!(self_ptr.ctx)) };
        let con =
            unsafe { core::pin::Pin::new_unchecked(&mut *core::ptr::addr_of_mut!(self_ptr.con)) };

        gen.resume((con.get_mut(), ctx.get_mut()))
    }
}

/// This is the only way to actually pass a message through the whole middleware chain
pub struct Pipeline<
    Con: Connection<Payload> + 'static + Unpin,
    Payload: Serialize + DeserializeOwned + 'static,
> {
    ctx: <Con as Connection<Payload>>::Ctx,
    con: Con,
    _marker: core::marker::PhantomData<fn() -> Payload>,
}

impl<Con: Connection<Payload> + Unpin, Payload: Serialize + DeserializeOwned>
    Pipeline<Con, Payload>
{
    pub fn new(c: Con, ctx: Con::Ctx) -> Self {
        Self {
            ctx,
            con: c,
            _marker: core::marker::PhantomData,
        }
    }

    pub fn ctx(&self) -> &Con::Ctx {
        &self.ctx
    }

    pub fn ctx_mut(&mut self) -> &mut Con::Ctx {
        &mut self.ctx
    }

    pub fn send(
        &mut self,
        message: Payload,
    ) -> impl Generator<(), Yield = Result<Con::Wrapped, Con::SendError>, Return = ()> + '_ {
        StructGen::<'_, _, <Con as Connection<Payload>>::Ctx, Con> {
            gen: Con::send(message, PublicUncallable),
            ctx: &mut self.ctx,
            con: &mut self.con,
            _pin: core::marker::PhantomPinned,
        }
    }

    pub fn receive(
        &mut self,
        message: Con::Wrapped,
    ) -> impl Generator<(), Yield = Result<Payload, Con::ReceiveError>, Return = ()> + '_ {
        StructGen {
            gen: Con::receive(message, PublicUncallable),
            ctx: &mut self.ctx,
            con: &mut self.con,
            _pin: core::marker::PhantomPinned,
        }
    }
}
