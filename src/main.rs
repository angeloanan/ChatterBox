pub mod routes;

use axum::{routing::get, Router};
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let (chat_announcer, _guard) = tokio::sync::broadcast::channel::<String>(1);

    // build our application with a single route
    let app = Router::new().route(
        "/ws",
        get(move |ws| routes::ws::handler(ws, chat_announcer)),
    );

    println!("Listening on http://0.0.0.0:4000");
    axum::Server::bind(&"0.0.0.0:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
