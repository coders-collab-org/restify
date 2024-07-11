mod controller;
pub mod dto;
pub mod entities;
pub mod services;

pub use controller::TodoController;

use restify::Module;

use crate::app::AppState;

#[derive(Module)]
#[module(controllers(TodoController), state(AppState))]
pub struct TodoModule;
