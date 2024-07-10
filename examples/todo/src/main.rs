use app::AppModule;
use restify::axum::IntoRouter;

mod app;
mod todo;

#[tokio::main]
async fn main() {
  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  axum::serve(listener, AppModule.into_router(&mut ()))
    .await
    .unwrap();
}
