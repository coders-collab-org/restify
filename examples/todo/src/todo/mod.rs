mod controller;
pub mod dto;
pub mod entities;
pub mod services;

pub use controller::TodoController;

use restify::Module;

#[derive(Module)]
#[module(controllers(TodoController))]
pub struct TodoModule;
