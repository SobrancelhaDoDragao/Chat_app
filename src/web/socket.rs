/// Arquivo para gerenciar websocket e states
use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

pub struct AppState {
    pub users: Mutex<HashSet<String>>,
    pub tx: broadcast::Sender<String>,
}

impl AppState {
    pub fn new() -> AppState {
        let users = Mutex::new(HashSet::new());
        let (tx, _rx) = broadcast::channel(100);

        AppState { users, tx }
    }
}

pub async fn websocket(mut stream: WebSocket, state: Arc<AppState>) {
    // By splitting, we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();
    let mut username = String::new();

    // A primeira mensagem do websocket e o nome do usuario sendo enviado
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(name) = message {
            let mut user_set = state.users.lock().unwrap();
            username.push_str(&name);
            user_set.insert(name.to_owned());

            if !username.is_empty() {
                break;
            } else {
                let _ = sender.send(Message::Text(String::from("Username already taken.")));

                return;
            }
        }
    }

    // We subscribe *before* the "joined" message, so that we will also
    // display it to our client.
    let mut rx = state.tx.subscribe();

    // Now send the "joined" message to all subscribers
    let msg = format!("{username} joined.");

    let _ = state.tx.send(msg);

    // Spawn the first task that will receive broadcast messages and send text
    // mesages over the websocket to our client.
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any webscoket error, break loop
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Nome defido agora o chat é iniciado
    // Para manter onwnership
    let tx = state.tx.clone();
    let name = username.clone();

    tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // add username before message
            let _ = tx.send(format!("{name}: {text}"));
        }
    });
}
