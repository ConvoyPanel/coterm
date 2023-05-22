use axum::extract::ws::{CloseFrame as ACloseFrame, Message as AMessage};
use axum::http::HeaderMap;
use axum::{
    body::{boxed, Body, BoxBody},
    extract::ws::{WebSocket, WebSocketUpgrade},
    http::{Response, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Router,
};
use base64;
use base64::{engine::general_purpose, Engine as _};
use dotenv::dotenv;
use futures_util::{sink::SinkExt, stream::StreamExt};
use helpers::create_request;
use httparse::{Header, Request, EMPTY_HEADER};
use jsonwebtoken::{decode, Validation, DecodingKey};
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use std::{net::SocketAddr, path::PathBuf};
use tokio::join;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Error as TWebSocketError;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        protocol::frame::coding::CloseCode as TCloseCode, protocol::CloseFrame as TCloseFrame,
        Message as TMessage,
    },
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use urlencoding::encode;

use crate::helpers::{convert_axum_to_tungstenite, convert_tungstenite_to_axum};
use crate::no_vnc::create_no_vnc_credentials;
use crate::xterm::create_xterm_credentials;
mod des;
mod helpers;
mod no_vnc;
mod xterm;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    server_uuid: String,
}

#[tokio::main]
async fn main() {
    //let decoding_key = DecodingKey::from_secret("eyJpdiI6Ikd2VUp4TEU1am9NbTFZWkpHeUVrS3c9PSIsInZhbHVlIjoic2g5dVFpekJsNmxNQWhNc0dQTEI0TERJMlNNVGdGRHFjUUxNRFlXbzd0VzI0M1dHZzRQYWhrT0pHUTBHSUpDekU2bUgyekZoSzFiZEpWakhhc1dOSjc0TTE3eGFQeHA0S0xjYmdmZVVIblU9IiwibWFjIjoiZGM1MWViYWZkMWE0N2ViNWJhMDE5ZWMzZDYzN2VhMzc0ZGYwNzZjODBmYWRkYTIxNTA5YzM1NjczZDgwZWZiMSIsInRhZyI6IiJ9".as_ref());
    //let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiIsImp0aSI6ImIxM2ZhYzY0YWIxZDBiMjU4MjRlYWU0YjYyNzA1NjYwNThjM2FhOWRjNThlNmMzZjcyYTdmYjRkZTdlYzdhNDcifQ.eyJpc3MiOiJodHRwczovL3VzLWVhc3QucGVyZm9ybWF2ZS5jb20iLCJhdWQiOiJodHRwczovL3VzLWVhc3QucGVyZm9ybWF2ZS5jb206NDQzIiwianRpIjoiYjEzZmFjNjRhYjFkMGIyNTgyNGVhZTRiNjI3MDU2NjA1OGMzYWE5ZGM1OGU2YzNmNzJhN2ZiNGRlN2VjN2E0NyIsImlhdCI6MTY4Mzc2OTUxNS42NDAwNjUsIm5iZiI6MTY4Mzc2OTIxNS42NDAwODMsImV4cCI6MTY4Mzc2OTU0NS42Mzc0MDgsInNlcnZlcl91dWlkIjoiYzZjNGJjMGQtZTdkNC00NjdjLWFkOTYtNzRkOGY4YzBmMmRmIiwidXNlcl91dWlkIjpudWxsLCJ1bmlxdWVfaWQiOiJmM1dsMjFQRlNjeWw4VkQ4In0.LeNXZ8-ROAYxNDPybGjfhnbJ6GgcSDs5Q9jdpvM9Q9Y";
    //println!("{:?}", decode::<Claims>(&token, &decoding_key, &Validation::default()));
    dotenv().ok();

    // let credentials = create_xterm_credentials("c6c4bc0d".to_owned()).await.unwrap();
    // println!("Credentials: {:?}", credentials);
    // return;
    let cors = CorsLayer::new().allow_origin(Any);

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dist");

    let app = Router::new()
        .route("/ws", get(|ws: WebSocketUpgrade| async {
            ws.on_upgrade(handle_socket)
        }))
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

async fn handle_socket(client_socket: WebSocket) {
    let (mut client_sender, mut client_receiver) = client_socket.split();

    let creds = create_no_vnc_credentials("c6c4bc0d".to_owned()).await.unwrap();

    let (remote_socket, _) = connect_async(create_request(helpers::Credentials::NoVnc(creds.clone())).body(()).unwrap())
        .await
        .unwrap();

        let (remote_sender, mut remote_receiver) = remote_socket.split();
    let remote_sender = Arc::new(Mutex::new(remote_sender));

    // send from client to remote and back
    let client_to_remote = async {
        let mut already_intercepted_auth_method = false;
        while let Some(Ok(msg)) = client_receiver.next().await {
            println!("client_to_remote: {:?}", msg);
            let remote_sender = remote_sender.clone();

            if msg == AMessage::Binary(vec![1]) && !already_intercepted_auth_method {
                println!("Intercepted auth method selection");
                already_intercepted_auth_method = true;
                continue;
            }

            remote_sender
                .lock()
                .await
                .send(convert_axum_to_tungstenite(msg))
                .await
                .unwrap();
        }
    };

    // send from remote to client and back
    let remote_to_client = async {
        let mut messages_received = 0;
        while let Some(Ok(msg)) = remote_receiver.next().await {
            if (messages_received < 5) {
                messages_received += 1;
            }
            println!("remote_to_client: {:?} | {:?}", msg.to_text(), msg);

            println!("messages_received: {}", messages_received);

            if msg == TMessage::Binary(vec![1,2]) {
                let remote_sender = remote_sender.clone();
                remote_sender
                    .lock()
                    .await
                    .send(TMessage::Binary(vec![2]))
                    .await
                    .unwrap();
                client_sender.send(AMessage::Binary(vec![1, 1])).await.unwrap();
                println!("Intercepted authentication method message");
                continue;
            }

            if messages_received == 3 {
                let remote_sender = remote_sender.clone();
                let ticket = creds.ticket.clone();

                let mut ticket = ticket.as_bytes().to_owned();
                for i in 0..8 {
                    let c = ticket[i];
                    let mut cs = 0u8;
                    for j in 0..8 {
                        cs |= ((c >> j) & 1) << (7 - j)
                    }
                    ticket[i] = cs;
                }

                let challenge = msg.into_data();

                let trimmed_ticket: &[u8; 8] = &ticket[0..8].try_into().unwrap();

                let response = des::encrypt(&challenge, &trimmed_ticket);

                remote_sender
                    .lock()
                    .await
                    .send(TMessage::Binary(response))
                    .await
                    .unwrap();

                println!("Sent ticket");
                continue;
            }



            client_sender
                .send(convert_tungstenite_to_axum(msg))
                .await
                .unwrap();

            println!("message forwarded remote_to_client");
        }
    };

    join!(client_to_remote, remote_to_client);
}