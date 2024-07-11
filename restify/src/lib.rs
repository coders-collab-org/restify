#[cfg(feature = "macros")]
pub use restify_macros::*;

pub use restify_core::*;

pub mod prelude {
  pub use restify_core::{BoxedControllerFn, BoxedModule, Controller, Module};
  #[cfg(feature = "macros")]
  pub use restify_macros::{controller, Module};
}
