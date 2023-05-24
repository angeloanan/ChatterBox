pub mod db;
pub mod routes;
pub mod names;

use std::{sync::Arc, net::SocketAddr};

use axum::{routing::get, Extension, Router, extract::ConnectInfo};
use axum_client_ip::SecureClientIpSource;
use dotenvy::dotenv;
use sqlx::{Pool, Sqlite};
use tokio::sync::broadcast::Sender;

pub struct AppState {
    db: Pool<Sqlite>,
    chat_announcer: Sender<String>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db = db::init_db().await;
    db::assert_table(&db).await;

    let (chat_announcer, _guard) = tokio::sync::broadcast::channel::<String>(1);

    // Clean up database per hour
    let cloned_db = db.clone();
    tokio::spawn(async move {
        loop {
            db::cleanup_message(&cloned_db).await;
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    });

    let shared_state = Arc::new(AppState {
        db,
        chat_announcer,
    });

    // build our application with a single route
    let app = Router::new()
        .route("/ws", get(routes::ws::handler))
        .layer(SecureClientIpSource::ConnectInfo.into_extension())
        .layer(Extension(shared_state));

    println!("Listening on http://0.0.0.0:4000");
    axum::Server::bind(&"0.0.0.0:4000".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
