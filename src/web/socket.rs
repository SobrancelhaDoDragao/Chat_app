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

pub async fn connection(stream: WebSocket, state: Arc<AppState>) {
    // By splitting, we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();
    let mut username = String::new();

    // A primeira mensagem do websocket é o nome do usuario sendo enviado
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(name) = message {
            check_username(&state, &mut username, &name);

            if !username.is_empty() {
                break;
            } else {
                let _ = sender
                    .send(Message::Text(String::from("Username already taken.")))
                    .await;

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

    // Esperando mensagem serem enviadas para o state para depois serem enviadas para o usuario.
    let mut from_state_to_user = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any webscoket error, break loop
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Para manter onwnership
    let tx = state.tx.clone();
    let name = username.clone();

    // Enviando mensagem para o broadcast do state
    let mut from_user_to_state = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // add username before message
            let _ = tx.send(format!("{name}: {text}"));
        }
    });

    // If any one of the task run to completion, we abort the other.
    tokio::select! {
        _ = (&mut from_state_to_user) => from_user_to_state.abort(),
        _ = (&mut from_user_to_state) => from_state_to_user.abort(),
    }

    // Send "user left" message
    let msg = format!("{username} left");
    let _ = state.tx.send(msg);

    // Remove username from map
    state.users.lock().unwrap().remove(&username);
}

fn check_username(state: &AppState, string: &mut String, name: &str) {
    let mut user_set = state.users.lock().unwrap();

    if !user_set.contains(name) {
        user_set.insert(name.to_owned());

        string.push_str(name);
    }
}
