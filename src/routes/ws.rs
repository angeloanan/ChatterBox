use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::Response,
};
use futures::{
    prelude::*,
    stream::{SplitSink, SplitStream},
};
use tokio::sync::broadcast;

pub async fn handler(
    ws: WebSocketUpgrade,
    sender: tokio::sync::broadcast::Sender<String>,
) -> Response {
    println!("Websocket connection established");
    ws.on_upgrade(move |socket| handle_socket(socket, sender))
}

async fn handle_socket(socket: WebSocket, tx: broadcast::Sender<String>) {
    let (sender, receiver) = socket.split();

    let mut send_task = tokio::spawn(write(sender, tx.subscribe()));
    let mut recv_task = tokio::spawn(read(receiver, tx));

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

async fn read(mut ws_receiver: SplitStream<WebSocket>, tx: broadcast::Sender<String>) {
    // Prints all messages from the client
    while let Some(msg) = ws_receiver.next().await {
        let message = if let Ok(msg) = msg {
            msg
        } else {
            println!("Error receiving message. Did client got disconnected?");
            return;
        };
        // TODO: idk why but client disconnected also sends an empty message here

        tx.send(message.to_text().unwrap().to_owned()).unwrap();
    }
}

// WS Message sender to browser client
async fn write(mut ws_sender: SplitSink<WebSocket, Message>, mut tx: broadcast::Receiver<String>) {
    // Loop over new chat messages and send them to ws_sender
    while let Ok(message) = tx.recv().await {
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
