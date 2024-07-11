use app::{AppModule, AppState};

use restify::axum::IntoRouter;

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
