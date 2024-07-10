use restify::prelude::*;

use crate::todo::TodoModule;

#[derive(Module)]
#[module(imports(TodoModule), controllers(AppController))]
pub struct AppModule;

pub struct AppController;

#[controller("/")]
impl AppController {
  #[get]
  async fn up() -> &'static str {
    "UP!"
  }
}
