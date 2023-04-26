use axum::{
    body::{boxed, Body, BoxBody},
    extract::ws::{WebSocket, WebSocketUpgrade},
    http::{Request, Response, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use std::sync::{Arc, Mutex};
use std::{net::SocketAddr, path::PathBuf};
use tokio::{join, sync::oneshot};
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
        let remote_ws = remote_ws.clone();
        let client_ws = client_ws.clone();

        let mut client_guard = client_ws.lock().unwrap();

        while let Some(msg) = client_guard.recv().await {
            let result = remote_ws
                .lock()
                .unwrap()
                .write_message(TMessage::binary(msg.unwrap().into_data()));

            if result.is_err() {
                // Handle error
                break;
            }
        }
    };

    let remote_to_client_handle = || async {
        let remote_ws = remote_ws.clone();
        let client_ws = client_ws.clone();

        let mut remote_guard = remote_ws.lock().unwrap();

        while let Ok(msg) = remote_guard.read_message() {
            let result = client_ws
                .lock()
                .unwrap()
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

// async fn handle_socket(mut _client_ws: WebSocket) {
//     let (_remote_ws, _) = tungstenite::connect("ws://localhost:3000/demo-ws")
//         .expect("Failed to connect to remote URL");
//     let remote_ws = Arc::new(Mutex::new(_remote_ws));
//     let client_ws = Arc::new(Mutex::new(_client_ws));

//     let (remote_tx, remote_rx) = oneshot::channel();
//     let (client_tx, client_rx) = oneshot::channel();

//     let client_to_remote_handle = tokio::spawn(async move {
//         let mut client_guard = client_ws.lock().unwrap();

//         while let Some(msg) = client_guard.recv().await {
//             let remote_ws = remote_ws.clone();
//             let client_ws = client_ws.clone();
//             let result = remote_ws.lock().unwrap().write_message(TMessage::binary(msg.unwrap().into_data()));

//             if result.is_err() {
//                 // Handle error
//                 break;
//             }
//         }

//         client_tx.send("client disconnected");
//     });

//     let remote_to_client_handle = tokio::spawn(async move {
//         let mut remote_guard = remote_ws.lock().unwrap();

//         while let Ok(msg) = remote_guard.read_message() {
//             let client_ws = client_ws.clone();
//             let result = client_ws.lock().unwrap().send(axum::extract::ws::Message::Binary(msg.into_data())).await;

//             if result.is_err() {
//                 // Handle error
//                 break;
//             }
//         }

//         remote_tx.send("remote disconnected");
//     });

//     join_all(client_to_remote_handle, remote_to_client_handle);

//     // tokio::spawn(async move {
//     //     let client_ws = client_ws.clone();

//     //     while let Some(msg) = client_ws.lock().expect("couldn\'t lock client_ws to receive message").recv().await {
//     //         // let msg = if let Ok(msg) = msg {
//     //         //     msg
//     //         // } else {
//     //         //     // client disconnected
//     //         //     return;
//     //         // };

//     //         // if client_ws.lock().expect("couldn\'t lock client_ws to send message").send("testfff".into()).await.is_err() {
//     //         //     // client disconnected
//     //         //     return;
//     //         // }
//     //     }
//     // });

//     // tokio::select! {
//     //     val = remote_rx => {
//     //         println!("remote_rx completed first with {:?}", val);
//     //     }
//     //     val = client_rx => {
//     //         println!("client_rx completed first with {:?}", val);
//     //     }
//     // }

//     // loop {
//     //     println!("pre");
//     //     // Receive message from client WebSocket
//     //     let msg = match client_socket.recv().await {
//     //         Some(msg) => {
//     //             println!("client msg {:?}", msg);
//     //             msg.unwrap()
//     //         },
//     //         None => {
//     //             println!("client disconnected");
//     //             return
//     //         }, // Client WebSocket disconnected
//     //     };

//     //     println!("post");

//     //     // Send message to remote WebSocket
//     //     if remote_ws.write_message(TMessage::binary(msg.into_data())).is_err() {
//     //         // Remote WebSocket disconnected
//     //         return;
//     //     }

//     //     println!("remote post");

//     //     // Receive message from remote WebSocket
//     //     let msg = match remote_ws.read_message() {
//     //         Ok(msg) => msg,
//     //         Err(_) => return, // Remote WebSocket disconnected
//     //     };

//     //     println!("remote msg {:?}", msg);

//     //     // Send message to client WebSocket
//     //     if client_socket.send(axum::extract::ws::Message::Binary(msg.into_data())).await.is_err() {
//     //         // Client WebSocket disconnected
//     //         return;
//     //     }
//     // }
// }
