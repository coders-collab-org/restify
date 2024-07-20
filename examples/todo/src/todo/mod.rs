mod controller;
pub mod dto;
pub mod entities;
pub mod services;

pub use controller::TodoController;

use restify::Module;

#[derive(Module)]
#[module(controllers(TodoController), middlewares(Self::hello_middleware))]
pub struct TodoModule;

impl TodoModule {
  pub fn hello_middleware(&self, _ctx: &mut ()) {
    println!("I'm middleware");
  }
}
