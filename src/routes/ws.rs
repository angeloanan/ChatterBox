use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::Response,
    Extension,
};
use futures::{
    prelude::*,
    stream::{SplitSink, SplitStream},
};
use sqlx::{Pool, Sqlite};
use tokio::sync::broadcast;

use crate::db;
use crate::AppState;

pub async fn handler(Extension(state): Extension<Arc<AppState>>, ws: WebSocketUpgrade) -> Response {
    println!("Websocket connection established");
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (sender, receiver) = socket.split();
    let tx = state.chat_announcer.clone();

    let mut send_task = tokio::spawn(write(sender, state.clone()));
    let mut recv_task = tokio::spawn(read(receiver, state.db.clone(), tx));

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

async fn read(
    mut ws_receiver: SplitStream<WebSocket>,
    pool: Pool<Sqlite>,
    tx: broadcast::Sender<String>,
) {
    // Prints all messages from the client
    while let Some(msg) = ws_receiver.next().await {
        let message = if let Ok(msg) = msg {
            msg
        } else {
            println!("Error receiving message. Did client got disconnected?");
            return;
        };

        // FIXME: idk why but client disconnected also sends an empty message here

        // TODO: Replace `Someone` with an actual username
        let username = "Someone";
        let text_message = message.to_text().unwrap();
        let time = time::OffsetDateTime::now_utc().unix_timestamp();

        // Announce to broadcast channel
        tx.send(format!("{} {}: {}", time, username, text_message))
            .unwrap();

        // Current workaround store non-empty message to DB
        if !message.to_text().unwrap().is_empty() {
            db::add_message(&pool, username, message.to_text().unwrap(), time).await;
        }
    }
}

// WS Message sender to browser client
async fn write(mut ws_sender: SplitSink<WebSocket, Message>, state: Arc<AppState>) {
    let db = state.db.clone();
    let tx = state.chat_announcer.clone();

    let msg = Message::Text(format!("Your username is: {}", username));

    // Fetch old message and send them to client
    let old_messages = db::fetch_past_messages(&db).await;
    for message in old_messages {
        let msg = Message::Text(format!(
            "{} {}: {}",
            message.created_at, message.username, message.message
        ));
        let sent_msg = ws_sender.send(msg).await;

        // Handle error
        if let Err(e) = sent_msg {
            // Don't print error message if client got disconnected normally
            println!("Error sending old message, did client got disconnected?: {e}",);
            break;
        }
    }

    // Loop over new chat messages and send them to ws_sender
    while let Ok(message) = tx.subscribe().recv().await {
        let msg = Message::Text(message);
        let sent_msg = ws_sender.send(msg).await;

        // Handle error
        if let Err(e) = sent_msg {
            // Don't print error message if client got disconnected normally
            println!("Error sending message, did client got disconnected?: {e}",);
            break;
        }
    }
}
