use std::{format, println, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::Response,
    Extension,
};
use axum_client_ip::SecureClientIp;
use futures::{
    prelude::*,
    stream::{SplitSink, SplitStream},
};
use sqlx::{Pool, Sqlite};
use tokio::sync::broadcast;

use crate::AppState;
use crate::{db, names::generate_name};

pub async fn handler(
    Extension(state): Extension<Arc<AppState>>,
    ws: WebSocketUpgrade,
    SecureClientIp(ip): SecureClientIp,
) -> Response {
    let username = generate_name();

    println!("> New connection {} ({})", username, ip);
    ws.on_upgrade(move |socket| handle_socket(socket, state, username))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>, username: String) {
    let (sender, receiver) = socket.split();
    let tx = state.chat_announcer.clone();

    let mut send_task = tokio::spawn(write(sender, state.clone(), username.clone()));
    let mut recv_task = tokio::spawn(read(receiver, state.db.clone(), tx, username));

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

// Browser client to WS message receiver
// C -> S
async fn read(
    mut ws_receiver: SplitStream<WebSocket>,
    pool: Pool<Sqlite>,
    tx: broadcast::Sender<String>,
    username: String,
) {
    // Prints all messages from the client
    while let Some(msg) = ws_receiver.next().await {
        let message = if let Ok(msg) = msg {
            msg
        } else {
            println!("> [{username}] Error receiving message - Did client disconnect mid-send?");
            return;
        };

        match message {
            Message::Text(text) => {
                let time = time::OffsetDateTime::now_utc().unix_timestamp();
                let message = format!("{} {}: {}", time, username, text);

                // Debug print bcs why not
                println!("{message}");

                // Announce to broadcast channel
                tx.send(message).unwrap();

                // Add message to database
                db::add_message(&pool, &username, &text, time).await;
            }
            Message::Close(..) => {
                println!("> ({username}) disconnected");
                break;
            }
            _ => (),
        }
    }
}

// WS Message sender to browser client
// S -> C
async fn write(
    mut ws_sender: SplitSink<WebSocket, Message>,
    state: Arc<AppState>,
    username: String,
) {
    let db = state.db.clone();
    let tx = state.chat_announcer.clone();

    let msg = Message::Text(format!("Your username is: {}", username));
    let _ = ws_sender
        .send(msg)
        .await
        .expect("Unable to send username. Did connection get dropped or server hanged?");

    // Fetch old message and send them to client
    let old_messages = db::fetch_past_messages(&db).await;
    for message in old_messages {
        let msg = Message::Text(format!(
            "{} {}: {}",
            message.created_at.unix_timestamp(),
            message.username,
            message.message
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
