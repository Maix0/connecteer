use crate::sealed;
use serde::{de::DeserializeOwned, Serialize};

/// You can't implement this trait, you need to let the blanket implementation do its job by
/// implementing [`Middleware`](crate::Middleware) on your types
/// This type isn't used directly by the consumer, it is only used by this crate
pub trait Connection<Payload: Serialize + DeserializeOwned>:
    sealed::Sealed<Payload> + Unpin + Sized
{
    type Wrapped: Serialize + DeserializeOwned;

    type Ctx: Unpin;

    type SendError;
    type ReceiveError;
    type NextError;

    type SendGen: crate::gen_utils::ConnectionGenerator<
        Self,
        Self::Ctx,
        Yield = Result<Self::Wrapped, Self::SendError>,
    >;
    type ReceiveGen: crate::gen_utils::ConnectionGenerator<
        Self,
        Self::Ctx,
        Yield = Result<Payload, Self::ReceiveError>,
    >;

    fn send(input: Payload, _: sealed::PublicUncallable) -> Self::SendGen;
    fn receive(output: Self::Wrapped, _: sealed::PublicUncallable) -> Self::ReceiveGen;
}
