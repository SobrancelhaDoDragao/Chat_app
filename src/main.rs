//! projeto com o intuito de criar um chat app em Rust com Websockets
use axum::Router;
use dotenv::dotenv;
use log::info;
use std::env;

mod web;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    info!("Server starting...");

    let app_host = env::var("APP_HOST").unwrap_or("0.0.0.0".to_string());
    let app_port = env::var("APP_PORT").unwrap_or("80".to_string());

    info!(
        "Server configured to accept connections on host {}:{}",
        app_host, app_port
    );

    let bind_address = app_host + ":" + &app_port;

    let routes = Router::new().merge(web::routes::all_routes());

    axum::Server::bind(&bind_address.parse().unwrap())
        .serve(routes.into_make_service())
        .await
        .unwrap();
}
