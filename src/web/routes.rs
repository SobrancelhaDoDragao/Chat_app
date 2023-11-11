/// Arquivo para gerenciar routas e handlers.
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::fs;

const PATH_TO_HTML: &str = "src/web/templates/";

pub fn all_routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/ws", get(handler))
        .with_state(AppState { /* ... */ })
}

#[derive(Clone)]
struct AppState {}

// Handlers
async fn index() -> impl IntoResponse {
    let html_file: &str = "index.html";

    let html_content = fs::read_to_string(format!("{}{}", PATH_TO_HTML, html_file))
        .expect("Erro ao ler o arquivo HTML");

    Html(html_content)
}

async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            println!("Mensagem {:?}", msg);
            msg
        } else {
            // client disconnected
            return;
        };

        if socket.send(msg).await.is_err() {
            // client disconnected
            return;
        }
    }
}
