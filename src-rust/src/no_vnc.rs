use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    body::{BoxBody, boxed},
    extract::ws::WebSocket,
};
use axum::extract::ws::Message as AMessage;
use axum::http::HeaderMap;
use base64;
use base64::Engine as _;
use futures_util::{sink::SinkExt, stream::StreamExt};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tokio::join;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message as TMessage,
};

use crate::des;
use crate::helpers::{self, convert_axum_to_tungstenite, convert_tungstenite_to_axum, create_request};

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
        Ok(credentials)
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}

pub async fn proxy_novnc_traffic(server_uuid: String, client_socket: WebSocket) {
    let (mut client_sender, mut client_receiver) = client_socket.split();

    let creds = create_no_vnc_credentials(server_uuid).await.unwrap();

    let (remote_socket, _) = connect_async(create_request(helpers::Credentials::NoVnc(creds.clone())).body(()).unwrap())
        .await
        .unwrap();

    let (remote_sender, mut remote_receiver) = remote_socket.split();
    let remote_sender = Arc::new(Mutex::new(remote_sender));

    // send from client to remote and back
    let client_to_remote = async {
        let mut already_intercepted_auth_method = false;
        while let Some(Ok(msg)) = client_receiver.next().await {
            let remote_sender = remote_sender.clone();

            if msg == AMessage::Binary(vec![1]) && !already_intercepted_auth_method {
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

            if msg == TMessage::Binary(vec![1, 2]) {
                let remote_sender = remote_sender.clone();
                remote_sender
                    .lock()
                    .await
                    .send(TMessage::Binary(vec![2]))
                    .await
                    .unwrap();
                client_sender.send(AMessage::Binary(vec![1, 1])).await.unwrap();
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

                continue;
            }


            client_sender
                .send(convert_tungstenite_to_axum(msg))
                .await
                .unwrap();
        }
    };

    join!(client_to_remote, remote_to_client);
}