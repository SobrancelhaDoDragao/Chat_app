/// Arquivo para gerenciar websocket e states
use axum::extract::ws::{Message, WebSocket};
use std::collections::HashSet;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub users: HashSet<String>,
    pub tx: broadcast::Sender<String>,
}

impl AppState {
    pub fn new() -> AppState {
        let users: HashSet<String> = HashSet::new();
        let (tx, _rx) = broadcast::channel(100);

        AppState { users, tx }
    }
}

pub async fn websocket(mut socket: WebSocket, state: AppState) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            println!("Mensagem {:?}", msg);
            msg
        } else {
            // client disconnected
            return;
        };
        // Comfirmando o recebimento da mensagem
        if socket
            .send(Message::Text(format!(
                "O servidor comfirma o recebimento da mensagem: {:?}",
                msg
            )))
            .await
            .is_err()
        {
            // client disconnected
            return;
        }
    }
}
