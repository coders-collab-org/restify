use restify::prelude::*;
use tower_http::trace::TraceLayer;

pub struct TodoController;

#[controller("/todo", wrap = TraceLayer::new_for_http())]
impl TodoController {
  #[get]
  async fn get_todo() -> &'static str {
    "todo"
  }
}
