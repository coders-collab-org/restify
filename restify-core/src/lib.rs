mod controller;
mod module;

pub use controller::Controller;
pub use module::Module;

#[cfg(feature = "axum")]
pub mod axum;

pub type BoxedModule<Ctx, ConCtx, ConRet> =
  Box<dyn Module<Context = Ctx, ControllerContext = ConCtx, ControllerReturn = ConRet>>;

pub type BoxedController<Ctx, Ret> = Box<dyn Controller<Context = Ctx, Return = Ret>>;
