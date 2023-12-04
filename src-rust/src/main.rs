#[macro_use]
extern crate dotenv_codegen;

use std::net::SocketAddr;
use axum::ServiceExt;
use dotenv::dotenv;
use tracing::info;

use crate::app::create_app;
use crate::util::broadcast_config::show_brand_message;

mod util;
mod app;
mod routes;

#[tokio::main]
async fn main() {
    show_brand_message();

    dotenv().ok();
    tracing_subscriber::fmt().init();

    let addr = SocketAddr::from(([0, 0, 0, 0], util::broadcast_config::get_broadcast_port()));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Convoy terminal is ready at {addr}");

    axum::Server::bind(&addr)
        .serve(create_app().await.into_make_service())
        .await
        .unwrap();
}
