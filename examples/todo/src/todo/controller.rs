use std::collections::HashMap;

use axum::{extract::Path, response::Response, Json};
use restify::prelude::*;
use tower_http::trace::TraceLayer;

use super::{
  dto::{CreateTodoDto, UpdateTodoDto},
  entities::TodoEntity,
  services::TodoService,
};

#[derive(Injectable)]
pub struct TodoController {
  service: TodoService,
}

#[controller("/todo", wrap = TraceLayer::new_for_http())]
impl TodoController {
  #[get]
  async fn get_all(self) -> Json<HashMap<String, TodoEntity>> {
    Json(self.service.get_all().await)
  }

  #[get("/:id")]
  async fn get_one(self, Path((id,)): Path<(String,)>) -> Json<Option<TodoEntity>> {
    Json(self.service.get_one(id).await)
  }

  #[post]
  async fn create(self, Json(dto): Json<CreateTodoDto>) -> Json<TodoEntity> {
    Json(self.service.create(dto).await)
  }

  #[patch("/:id")]
  async fn update(
    self,
    Path((id,)): Path<(String,)>,
    Json(dto): Json<UpdateTodoDto>,
  ) -> Result<Json<TodoEntity>, Response> {
    self.service.update(id, dto).await.map(Json)
  }

  #[delete("/:id")]
  async fn delete(self, Path((id,)): Path<(String,)>) -> Result<Json<TodoEntity>, Response> {
    self.service.delete(id).await.map(Json)
  }
}
