use std::collections::HashMap;

use axum_macros::FromRequestParts;
use uuid::Uuid;

use crate::app::AppState;

use super::{
  dto::{CreateTodoDto, UpdateTodoDto},
  entities::TodoEntity,
};

use axum::{extract::State, http::StatusCode, response::Response};

#[derive(FromRequestParts, Clone)]
pub struct TodoService {
  #[from_request(via(State))]
  state: AppState,
}

impl TodoService {
  pub async fn get_all(&self) -> HashMap<String, TodoEntity> {
    self.state.store.lock().await.clone()
  }

  pub async fn get_one(&self, id: String) -> Option<TodoEntity> {
    self.state.store.lock().await.get(&id).cloned()
  }

  pub async fn create(&self, dto: CreateTodoDto) -> TodoEntity {
    let id = Uuid::new_v4().to_string();

    let todo = TodoEntity {
      id: id.clone(),
      name: dto.name,
      done: false,
    };

    self
      .state
      .store
      .lock()
      .await
      .insert(id.clone(), todo.clone());

    todo
  }

  pub async fn update(&self, id: String, dto: UpdateTodoDto) -> Result<TodoEntity, Response> {
    let mut store = self.state.store.lock().await;

    if let Some(todo) = store.get_mut(&id) {
      if let Some(name) = dto.name {
        todo.name = name;
      }

      if let Some(done) = dto.done {
        todo.done = done;
      }

      return Ok(todo.clone());
    }

    return Err(
      Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(().into())
        .unwrap(),
    );
  }

  pub async fn delete(&self, id: String) -> Result<TodoEntity, Response> {
    self.state.store.lock().await.remove(&id).ok_or_else(|| {
      Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(().into())
        .unwrap()
    })
  }
}
