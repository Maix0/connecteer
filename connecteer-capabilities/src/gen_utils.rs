use core::ops::Generator;

pub trait ConnecteerGenerator<Connection, Context>
where
    Self: for<'a> Generator<(&'a mut Connection, &'a mut Context), Return = ()>,
{
}

impl<G: ?Sized, Connection, Context> ConnecteerGenerator<Connection, Context> for G where
    Self: for<'a> Generator<(&'a mut Connection, &'a mut Context), Return = ()>
{
}

#[allow(clippy::type_complexity)]
#[doc(hidden)]
pub struct UnsafeHigherRankGenerator<'s, G, Conn, Ctx, Y, R>(
    G,
    ::core::marker::PhantomData<fn(&'s mut Conn, &'s mut Ctx) -> (Y, R)>,
)
where
    Conn: 's,
    Ctx: 's,
    G: Generator<(&'s mut Conn, &'s mut Ctx), Yield = Y, Return = R>;

impl<'s, G, Conn, Ctx, Y, R> UnsafeHigherRankGenerator<'s, G, Conn, Ctx, Y, R>
where
    Conn: 's,
    Ctx: 's,
    G: Generator<(&'s mut Conn, &'s mut Ctx), Yield = Y, Return = R>,
{
    #[doc(hidden)]
    #[inline]
    pub unsafe fn new(g: G) -> Self {
        Self(g, core::marker::PhantomData)
    }
}

impl<'s, G, Conn, Ctx, Y, R> Generator<(&mut Conn, &mut Ctx)>
    for UnsafeHigherRankGenerator<'s, G, Conn, Ctx, Y, R>
where
    Conn: 's,
    Ctx: 's,
    G: Generator<(&'s mut Conn, &'s mut Ctx), Yield = Y, Return = R>,
{
    type Yield = Y;
    type Return = R;

    #[inline]
    fn resume(
        self: ::core::pin::Pin<&mut Self>,
        cx: (&mut Conn, &mut Ctx),
    ) -> ::core::ops::GeneratorState<Y, R> {
        unsafe { self.map_unchecked_mut(|it| &mut it.0) }
            .resume(unsafe { ::core::mem::transmute(cx) })
    }
}

#[doc(hidden)]
pub struct BetweenYields();

impl BetweenYields {
    #[doc(hidden)]
    #[inline]
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
macro_rules! higher_order_gen {(
    $($static_move:ident)*
    |($arg1:ident, $arg2:ident) $(: (&mut $Type1:ty ,&mut $Type2:ty))? $(,)?|
    $body:block $(,)?
) => ({
    // extra safety: ensure no un-macro-ed `yield`s remain the `$body`:
    if false {
        #[allow(unused_mut, deref_nullptr)]
        let _: &dyn FnOnce(_) -> _ = &|(__arg1, __arg2) $(: (&mut $Type1, &mut $Type2))?| {
            let (mut $arg1, mut $arg2) $(: (&mut $Type1, &mut $Type2))? = unsafe {(&mut *::core::ptr::null_mut(), &mut *::core::ptr::null_mut())};
            macro_rules! yield_ {( $e:expr ) => ( ::core::mem::drop($e) )}
            $body
        };
        loop {}
    }
    let gen = $($static_move)* |(__arg1, __arg2) $(: (&mut $Type1, &mut $Type2))?| {
        #[allow(unused_mut)]
        let mut lt = $crate::gen_utils::BetweenYields();
        #[allow(unused_mut)]
        let (mut $arg1, mut $arg2) = (lt.adjust(__arg1), lt.adjust(__arg2));
        macro_rules! yield_ {( $e:expr ) => ({
            let (__arg1, __arg2) = yield $e;
            lt = $crate::gen_utils::BetweenYields();
            ($arg1, $arg2) = (lt.adjust(__arg1), lt.adjust(__arg2));
        })}
        $body
    };
    unsafe {
        $crate::gen_utils::UnsafeHigherRankGenerator::new(gen)
    }
})}

#[doc(hidden)]
#[inline]
pub fn gen_interface_check<G: Generator<(), Return = ()>>(g: G) -> G {
    g
}

#[doc(hidden)]
#[macro_export]
macro_rules! iter_generator {
    (for $v:pat in $gen:block {$($t:tt)*}) => {
        {
            let mut __gen = ::core::pin::pin!($crate::gen_utils::gen_interface_check($gen));
            while let ::core::ops::GeneratorState::Yielded($v) = ::core::ops::Generator::resume(__gen.as_mut(), ()) {
                $($t)*
            }
        }
    };
}
