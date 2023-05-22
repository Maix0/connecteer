use core::ops::{Generator, GeneratorState};

#[allow(clippy::type_complexity)]
pub struct UnsafeHigherRankGenerator<'s, 'c, G, Conn, Ctx, Y, R>(
    G,
    ::core::marker::PhantomData<fn(&'s mut Conn, &'c mut Ctx) -> (Y, R)>,
)
where
    Conn: 's,
    Ctx: 'c,
    G: Generator<(&'s mut Conn, &'c mut Ctx), Yield = Y, Return = R>;

impl<'s, 'c, G, Conn, Ctx, Y, R> UnsafeHigherRankGenerator<'s, 'c, G, Conn, Ctx, Y, R>
where
    Conn: 's,
    Ctx: 'c,
    G: Generator<(&'s mut Conn, &'c mut Ctx), Yield = Y, Return = R>,
{
    pub unsafe fn new(g: G) -> Self {
        Self(g, core::marker::PhantomData)
    }
}

impl<'s, 'c, G, Conn, Ctx, Y, R> Generator<(&mut Conn, &mut Ctx)>
    for UnsafeHigherRankGenerator<'s, 'c, G, Conn, Ctx, Y, R>
where
    Conn: 's,
    Ctx: 'c,
    G: Generator<(&'s mut Conn, &'c mut Ctx), Yield = Y, Return = R>,
{
    type Yield = Y;
    type Return = R;

    fn resume(
        self: ::core::pin::Pin<&mut Self>,
        cx: (&mut Conn, &mut Ctx),
    ) -> GeneratorState<Y, R> {
        unsafe { self.map_unchecked_mut(|it| &mut it.0) }
            .resume(unsafe { ::core::mem::transmute(cx) })
    }
}

pub struct BetweenYields();

impl BetweenYields {
    pub fn adjust<'between_yields, 'too_big, ResumeArg: ?Sized>(
        &'between_yields self,
        resume_arg: &'too_big mut ResumeArg,
    ) -> &'between_yields mut ResumeArg
    where
        'too_big: 'between_yields,
    {
        resume_arg
    }
}

#[macro_export]
macro_rules! between_yields_lifetime {
    ( as $lt:ident ) => {
        #[allow(unused_mut)]
        let mut $lt = $crate::gen_utils::BetweenYields();
        macro_rules! yield_ {
            ( $e:expr ) => {
                match (yield $e, $lt = $crate::gen_utils::BetweenYields()).0 {
                    (a, b) => ($lt.adjust(a), $lt.adjust(b)),
                }
            };
        }
    };
}
