mod controller;

pub use controller::TodoController;

use restify::Module;

#[derive(Module)]
#[module(controllers(TodoController))]
pub struct TodoModule;
