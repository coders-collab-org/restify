# Restify

[![Crates.io](https://img.shields.io/crates/v/restify.svg)](https://crates.io/crates/restify)
[![Docs.rs](https://docs.rs/restify/badge.svg)](https://docs.rs/restify)

**Restify** is a powerful Rust framework designed to streamline the process of building web servers, offering support for multiple web frameworks like Axum, Actix, Rocket (and more in the future), while seamlessly integrating OpenAPI specification generation for effortless API documentation.

## Features

- **Multi-Framework Support:** Build your application using your preferred web framework, currently supporting Axum and with plans for Actix and Rocket integration.
- **Declarative Routing:** Define routes and controllers using intuitive macros, making your code clean and readable.
- **Automatic OpenAPI Generation:** Restify automatically generates OpenAPI documentation based on your defined routes and data structures.
- **Modular Structure:** Organize your application into modules for better maintainability and scalability.
- **State Management:** Easily manage shared application state.
- **Middleware Support:** Benefit from the middleware capabilities of your chosen web framework.

## Roadmap

- [x] Axum
- [ ] Actix
- [ ] Rocket
- [ ] OpenAPI

## Installation

Add the following to your `Cargo.toml` file, choosing the feature for your desired web framework:

```toml
[dependencies]
restify = { version = "0.0.2", features = ["axum"] }

# Dependencies for your chosen web framework (example for Axum)
axum = "0.6"
tokio = { version = "1", features = ["full"] }
```

## Usage

**1. Define your data structures:**

```rust
// /todo/entities.rs

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct TodoEntity {
  pub id: String,
  pub name: String,
  pub done: bool,
}
```

```rust
// /todo/dto.rs

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
```

**2. Create your service:**

```rust
// /todo/service.rs

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
```

**3. Create your controller:**

```rust
// /todo/controller.rs

use axum::{extract::Path, response::Response, Json};
use axum_macros::FromRequestParts;
use restify::prelude::*;
use tower_http::trace::TraceLayer;

use crate::app::AppState;
use super::{
  dto::{CreateTodoDto, UpdateTodoDto},
  entities::TodoEntity,
  services::TodoService,
};

#[derive(FromRequestParts)]
#[from_request(state(AppState))]
pub struct TodoController {
  service: TodoService,
}

#[controller("/todo", state(AppState), wrap = TraceLayer::new_for_http())]
impl TodoController {
  #[get]
  async fn get_all(self) -> Json<HashMap<String, TodoEntity>> {
    Json(self.service.get_all().await)
  }
  // ... other routes for creating, updating, and deleting todos
}
```

**4. Define your modules:**

```rust
/// /todo/mod.rs
use restify::Module;

use crate::app::AppState;

#[derive(Module)]
#[module(controllers(TodoController), state(AppState))]
pub struct TodoModule;
```

```rust
// /app.rs

use std::{collections::HashMap, ops::Deref, sync::Arc};

use restify::prelude::*;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;

use crate::todo::{entities::TodoEntity, TodoModule};

#[derive(Module)]
#[module(imports(TodoModule), controllers(AppController), state(AppState))]
pub struct AppModule;

pub struct AppController;

#[controller("/", state(AppState), wrap = TraceLayer::new_for_http())]
impl AppController {
  #[get]
  async fn up() -> &'static str {
    "UP!"
  }
}

#[derive(Clone, Default)]
pub struct AppState(Arc<AppStateInner>);

#[derive(Default)]
pub struct AppStateInner {
  pub store: Mutex<HashMap<String, TodoEntity>>,
}

impl Deref for AppState {
  type Target = AppStateInner;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
```

**5. Create your application (example for Axum):**

```rust
use app::{AppModule, AppState};

use restify::axum::IntoRouter; // Import specific to Axum

mod app;
mod todo;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  axum::serve(
    listener,
    AppModule
      .into_router(&mut ())
      .with_state(AppState::default()),
  )
  .await
  .unwrap();
}
```

## Configuration

Restify offers configuration options through the `controller` and `Module` macros. These options allow you to specify:

- **Path:** The base path for the controller's routes.
- **State:** The type of shared application state to inject into the controller (for axum).
- **Wrap:** Middleware layers to apply to the controller's routes (using the middleware mechanisms of your chosen framework).

## Contribution

Contributions are welcome! If you'd like to contribute to Restify, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and write tests.
4. Submit a pull request.

## License

This project is licensed under the [MIT License][license].

## Acknowledgements

- **Axum, Actix, Rocket:** The supported web frameworks.
- **OpenAPI Specification:** The standard for defining REST
