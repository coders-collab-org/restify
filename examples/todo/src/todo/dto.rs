use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateTodoDto {
  pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodoDto {
  #[serde(default)]
  pub name: Option<String>,

  #[serde(default)]
  pub done: Option<bool>,
}
