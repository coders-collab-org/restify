use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct TodoEntity {
  pub id: String,
  pub name: String,
  pub done: bool,
}
