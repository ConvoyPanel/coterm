use axum::extract::ws::{CloseFrame as ACloseFrame, Message as AMessage};
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::Json;
use axum::{
    body::{boxed, Body, BoxBody},
    extract::ws::{WebSocket, WebSocketUpgrade},
    http::{Response as HttpResponse, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_auth::AuthBearer;
use axum_extra::extract::CookieJar;
use base64;
use base64::{engine::general_purpose, Engine as _};
use dotenv::dotenv;
use futures_util::{sink::SinkExt, stream::StreamExt};
use helpers::create_request;
use httparse::{Header, Request, EMPTY_HEADER};
use jsonwebtoken::{decode, DecodingKey, Validation};
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
    console_type: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cors = CorsLayer::new().allow_origin(Any);

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dist");

    let app = Router::new()
        .route("/ws", get(authenticate_and_upgrade))
        .nest_service("/", ServeDir::new(assets_dir))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn authenticate_and_upgrade(
    jar: CookieJar,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, StatusCode> {
    let jwt_in_a_cookie: Option<String> = jar.get("token").map(|cookie| cookie.value().to_owned());

    if let Some(jwt) = jwt_in_a_cookie {
        let token = dotenv::var("TOKEN").unwrap();
        let secret = token.split("|").nth(1).expect("Token is formatted incorrectly. Please check your .env file. The token should be in the format TOKEN_ID|TOKEN_SECRET");
        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        if let Ok(jwt) = decode::<Claims>(&jwt, &decoding_key, &Validation::default()) {
            match jwt.claims.console_type.as_str() {
                "novnc" => {
                    return Ok(ws.on_upgrade(|ws: WebSocket| { proxy_novnc_traffic(jwt.claims.server_uuid, ws) }));
                }
                _ => {
                    println!("{:#?}", jwt);
                    return Err(StatusCode::BAD_REQUEST);
                }
            }
        } else {
            println!("Invalid token");
            return Err(StatusCode::BAD_REQUEST);
        }
    } else {
        println!("No cookie found");
        return Err(StatusCode::BAD_REQUEST);
    }
}
