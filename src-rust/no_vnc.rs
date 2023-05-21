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
use httparse::{Header, Request, EMPTY_HEADER};
use jsonwebtoken::{decode, DecodingKey, Validation};
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
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

#[derive(Deserialize, Debug, Clone)]
pub struct NoVncCredentials {
    pub node_fqdn: String,
    pub node_port: u32,
    pub node_pve_name: String,
    pub vmid: u32,
    pub port: u32,
    pub ticket: String,
    pub pve_auth_cookie: String,
}

pub async fn create_no_vnc_credentials(
    server_uuid: String,
) -> Result<NoVncCredentials, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        format!("Bearer {}", dotenv!("TOKEN")).parse().unwrap(),
    );
    let mut payload = HashMap::new();
    payload.insert("type".to_owned(), "no_vnc".to_owned());

    let response = Client::new()
        .post(format!(
            "{}/api/coterm/servers/{}/create-console-session",
            dotenv!("CONVOY_URL"),
            server_uuid
        ))
        .json(&payload)
        .headers(headers)
        .send()
        .await
        .unwrap();

    if response.status().is_success() {
        let data: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        // deserialize the response which is nested in "data"
        let credentials: NoVncCredentials = serde_json::from_value(data["data"].clone()).unwrap();
        println!("Data: {:?}", credentials);
        Ok(credentials)
    } else {
        println!("Error: {:?}", response);
        Err(response.error_for_status().unwrap_err())
    }
}
