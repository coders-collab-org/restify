use std::{collections::HashMap, ops::Deref, sync::Arc};

use restify::prelude::*;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;

use crate::todo::{entities::TodoEntity, TodoModule};

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
