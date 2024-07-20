mod controller;
mod module;

pub use controller::Controller;
pub use controller::ControllerDetails;
pub use module::{Module, ModuleDetails};

#[cfg(feature = "axum")]
pub mod axum;

pub type BoxedModule<Ctx, ConCtx, ConRet> =
  Box<dyn Module<Context = Ctx, ControllerContext = ConCtx, ControllerReturn = ConRet>>;

pub type BoxedControllerFn<Ctx, Ret> = Box<dyn Fn(&mut Ctx) -> ControllerDetails<Ret>>;
