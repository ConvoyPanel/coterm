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

use crate::des;
use crate::helpers::{
    self, convert_axum_to_tungstenite, convert_tungstenite_to_axum, create_request,
};

#[derive(Deserialize, Debug, Clone)]
pub struct XTermCredentials {
    pub node_fqdn: String,
    pub node_port: u32,
    pub node_pve_name: String,
    pub vmid: u32,
    pub port: u32,
    pub ticket: String,
    pub username: String,
    pub realm_type: String,
    pub pve_auth_cookie: String,
}

pub async fn create_xterm_credentials(
    server_uuid: String,
) -> Result<XTermCredentials, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        format!("Bearer {}", dotenv::var("TOKEN").unwrap())
            .parse()
            .unwrap(),
    );
    let mut payload = HashMap::new();
    payload.insert("type".to_owned(), "xtermjs".to_owned());

    let response = Client::new()
        .post(format!(
            "{}/api/coterm/servers/{}/create-console-session",
            dotenv::var("CONVOY_URL").unwrap(),
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
        let credentials: XTermCredentials = serde_json::from_value(data["data"].clone()).unwrap();
        println!("Data: {:?}", credentials);
        Ok(credentials)
    } else {
        println!("Error: {:?}", response);
        Err(response.error_for_status().unwrap_err())
    }
}

pub async fn proxy_xterm_traffic(client_socket: WebSocket) {
    let (mut client_sender, mut client_receiver) = client_socket.split();

    // TODO: make server_uuid dynamic
    let creds = create_xterm_credentials("c6c4bc0d".to_owned())
        .await
        .unwrap();

    let (remote_socket, _) = connect_async(
        create_request(helpers::Credentials::XTerm(creds.clone()))
            .body(())
            .unwrap(),
    )
    .await
    .unwrap();

    let (mut remote_sender, mut remote_receiver) = remote_socket.split();

    let remote_creds = format!("{}@{}:{}", creds.username, creds.realm_type, creds.ticket);
    remote_sender.send(TMessage::Text(remote_creds.clone())).await.unwrap();
    println!("Sent credentials to remote: {}", remote_creds);

    let client_to_remote = async {
        while let Some(Ok(msg)) = client_receiver.next().await {
            println!("client_to_remote: {:?}", msg);

            remote_sender
                .send(convert_axum_to_tungstenite(msg))
                .await
                .unwrap();
        }
    };

    // send from remote to client and back
    let remote_to_client = async {
        while let Some(Ok(msg)) = remote_receiver.next().await {
            println!("remote_to_client: {:?}", msg);

            client_sender
                .send(convert_tungstenite_to_axum(msg))
                .await
                .unwrap();
        }
    };

    join!(client_to_remote, remote_to_client);
}
