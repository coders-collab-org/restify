use crate::{module::resolve_module, Module};

pub type Router<S = ()> = axum::Router<S>;

pub trait IntoRouter<S, Ctx> {
  fn into_router(self, ctx: &mut Ctx) -> Router<S>;
}

impl<T, S, Ctx> IntoRouter<S, Ctx> for T
where
  S: Clone + Send + Sync + 'static,
  T: Module<Context = Ctx, ControllerContext = (), ControllerReturn = Router<S>>,
{
  fn into_router(self, ctx: &mut Ctx) -> Router<S> {
    let controllers = resolve_module(&self, ctx);
    let mut router = Router::new();

    for con in controllers {
      let details = con(&mut ());

      router = router.nest(&details.path, details.return_);
    }

    router
  }
}
