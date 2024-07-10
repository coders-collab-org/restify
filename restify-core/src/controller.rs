pub trait Controller {
  type Context;
  type Return;

  fn path(&self) -> &str;
  fn configure(&self, ctx: &mut Self::Context) -> Self::Return;
}
