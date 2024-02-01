use std::net::SocketAddr;

use dotenv::{dotenv, var};
use tracing::info;

use crate::app::create_app;
use crate::util::broadcast_config::show_brand_message;
use tokio::signal::unix::{signal, SignalKind};

mod util;
mod app;
mod routes;

#[tokio::main]
async fn main() {
    show_brand_message();

    dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(if var("DEBUG").unwrap_or("false".to_string()).parse::<bool>().unwrap_or(false) {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();

    let addr = SocketAddr::from(([0, 0, 0, 0], util::broadcast_config::get_broadcast_port()));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Convoy terminal is ready at {addr}");

    let server = async {
        axum::serve(listener, create_app().await.into_make_service()).await.unwrap();
    };

    let mut signal_stream = signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = server => {
            // do nothing
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Convoy terminal is shutting down");
        }
        _ = signal_stream.recv() => {
            info!("Convoy terminal is shutting down");
        }
    }
}
