use axum::{
    body::{boxed, Body, BoxBody},
    extract::ws::{WebSocket, WebSocketUpgrade},
    http::{Request, Response, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use std::sync::Arc;
use std::{net::SocketAddr, path::PathBuf};
use tokio::{join, sync::Mutex, task};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tungstenite::Message as TMessage;
//use futures::future::join_all;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new().allow_origin(Any);

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dist");

    let app = Router::new()
        .route("/ws", get(ws_upgrader_handler))
        .route("/test", get(|| async { "Hi from /foo" }))
        .nest_service("/", ServeDir::new(assets_dir))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ws_upgrader_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut _client_ws: WebSocket) {
    let (_remote_ws, _) = tungstenite::connect("wss://socketsbay.com/wss/v2/1/demo/")
        .expect("Failed to connect to remote URL");
    let remote_ws = Arc::new(Mutex::new(_remote_ws));
    let client_ws = Arc::new(Mutex::new(_client_ws));

    println!("handle_socket");

    let client_to_remote_handle = || async {
        println!("what the hell");
        let remote_ws = Arc::clone(&remote_ws);
        let client_ws = Arc::clone(&client_ws);

        println!("client_to_remote_handle");

        let mut client_guard = client_ws.lock().await;

        while let Some(msg) = client_guard.recv().await {
            let result = remote_ws
                .lock()
                .await
                .write_message(TMessage::binary(msg.unwrap().into_data()));

            println!("client_to_remote_handle: {:?}", result);

            if result.is_err() {
                // Handle error
                break;
            }
        }
    };

    let remote_to_client_handle = || async {
        let remote_ws = Arc::clone(&remote_ws);
        let client_ws = Arc::clone(&client_ws);

        let mut remote_guard = remote_ws.lock().await;

        while let Ok(msg) = remote_guard.read_message() {
            let result = client_ws
                .lock()
                .await
                .send(axum::extract::ws::Message::Binary(msg.into_data()))
                .await;

            if result.is_err() {
                // Handle error
                break;
            }
        }
    };

    tokio::select! {
        _ = client_to_remote_handle() => {
            println!("client disconnected");
        }
        _ = remote_to_client_handle() => {
            println!("remote disconnected");
        }
    }
}
