use restify::prelude::*;
use tower_http::trace::TraceLayer;

use crate::todo::TodoModule;

#[derive(Module)]
#[module(imports(TodoModule), controllers(AppController))]
pub struct AppModule;

pub struct AppController;

#[controller("/", wrap = TraceLayer::new_for_http())]
impl AppController {
  #[get]
  async fn up() -> &'static str {
    "UP!"
  }
}
