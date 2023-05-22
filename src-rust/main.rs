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
use crate::no_vnc::{create_no_vnc_credentials, proxy_novnc_traffic};
use crate::xterm::{create_xterm_credentials, proxy_xterm_traffic};
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

    let cors = CorsLayer::new().allow_origin(Any);

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dist");

    let app = Router::new()
        .route("/ws", get(|ws: WebSocketUpgrade| async {
            ws.on_upgrade(proxy_xterm_traffic)
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
