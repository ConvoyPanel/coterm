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
use crate::helpers::{create_request, self, convert_axum_to_tungstenite, convert_tungstenite_to_axum};

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
        format!("Bearer {}", dotenv::var("TOKEN").unwrap()).parse().unwrap(),
    );
    let mut payload = HashMap::new();
    payload.insert("type".to_owned(), "novnc".to_owned());

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
        let credentials: NoVncCredentials = serde_json::from_value(data["data"].clone()).unwrap();
        println!("Data: {:?}", credentials);
        Ok(credentials)
    } else {
        println!("Error: {:?}", response);
        Err(response.error_for_status().unwrap_err())
    }
}

pub async fn proxy_novnc_traffic(client_socket: WebSocket) {
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