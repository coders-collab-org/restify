use restify::prelude::*;
pub struct TodoController;

#[controller("/todo")]
impl TodoController {
  #[get]
  async fn get_todo() -> &'static str {
    "todo"
  }
}
