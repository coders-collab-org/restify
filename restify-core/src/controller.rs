use std::borrow::Cow;

pub trait Controller {
  type Context;
  type Return;

  fn configure(ctx: &mut Self::Context) -> ControllerDetails<Self::Return>;
}

pub struct ControllerDetails<Ret> {
  pub path: Cow<'static, str>,
  pub return_: Ret,
}
