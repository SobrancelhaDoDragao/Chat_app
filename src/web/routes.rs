/// Arquivo para gerenciar routas e handlers.
use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};

use std::{fs, sync::Arc};

use crate::web::socket::{connection, AppState};

const PATH_TO_HTML: &str = "src/web/templates/";

pub fn all_routes() -> Router {
    let app_state = Arc::new(AppState::new());

    Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(websocket_handler))
        .with_state(app_state)
}

// Handlers
async fn index_handler() -> impl IntoResponse {
    let html_file: &str = "index.html";

    let html_content = fs::read_to_string(format!("{}{}", PATH_TO_HTML, html_file))
        .expect("Erro ao ler o arquivo HTML");

    Html(html_content)
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|socket| connection(socket, state))
}
