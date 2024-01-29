use std::sync::Arc;

use axum::extract::ws::Message as AMessage;
use axum::extract::ws::WebSocket;
use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use tokio::join;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{MaybeTlsStream, tungstenite::Message as TMessage, WebSocketStream};
use tracing::error;

use crate::util::api::novnc::{create_novnc_credentials, NoVncCredentials};
use crate::util::api::proxmox::{build_ws_request, Credentials};
use crate::util::crypto::des;
use crate::util::websocket::{convert_axum_to_tungstenite, convert_tungstenite_to_axum};

pub async fn start_novnc_proxy(server_uuid: String, client_ws: WebSocket) {
    let credentials = create_novnc_credentials(server_uuid).await.unwrap();

    let (request, connector) = build_ws_request(
        Credentials::NoVnc(credentials.clone())
    );
    let remote_ws = match tokio_tungstenite::connect_async_tls_with_config(
        request,
        None,
        false,
        Some(connector),
    ).await {
        Ok((ws, _)) => ws,
        Err(e) => {
            error!(
                "Failed to connect to Proxmox ({proxmox}): {error}",
                proxmox = credentials.node_fqdn,
                error = e,
            );

            client_ws.close().await.unwrap();
            return;
        }
    };

    let (client_sender, client_receiver) = client_ws.split();
    let (remote_sender, remote_receiver) = remote_ws.split();

    let client_sender = Arc::new(Mutex::new(client_sender));
    let client_receiver = Arc::new(Mutex::new(client_receiver));

    let remote_sender = Arc::new(Mutex::new(remote_sender));
    let remote_receiver = Arc::new(Mutex::new(remote_receiver));

    authenticate(
        client_sender.clone(),
        client_receiver.clone(),
        remote_sender.clone(),
        remote_receiver.clone(),
        credentials,
    ).await;

    let client_to_remote = async {
        while let Some(Ok(msg)) = client_receiver.lock().await.next().await {
            remote_sender.lock().await.send(convert_axum_to_tungstenite(msg)).await.unwrap();
        }

        remote_sender.lock().await.close().await.unwrap();
    };

    let remote_to_client = async {
        while let Some(Ok(msg)) = remote_receiver.lock().await.next().await {
            client_sender.lock().await.send(convert_tungstenite_to_axum(msg)).await.unwrap();
        }

        client_sender.lock().await.close().await.unwrap();
    };

    join!(client_to_remote, remote_to_client);
}

async fn authenticate(
    client_sender: Arc<Mutex<SplitSink<WebSocket, AMessage>>>,
    client_receiver: Arc<Mutex<SplitStream<WebSocket>>>,
    remote_sender: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, TMessage>>>,
    remote_receiver: Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    credentials: NoVncCredentials,
) {
    let capture_client_messages = async {
        while let Some(Ok(msg)) = client_receiver.lock().await.next().await {
            if msg == AMessage::Binary(vec![1]) {
                break;
            }

            remote_sender
                .lock()
                .await
                .send(convert_axum_to_tungstenite(msg))
                .await
                .unwrap();
        }
    };

    let capture_remote_messages = async {
        let mut messages_received = 0;

        while let Some(Ok(msg)) = remote_receiver.lock().await.next().await {
            if messages_received < 3 {
                messages_received += 1;
            }

            if msg == TMessage::Binary(vec![1, 2]) {
                remote_sender
                    .lock()
                    .await
                    .send(TMessage::Binary(vec![2]))
                    .await
                    .unwrap();
                client_sender
                    .lock()
                    .await
                    .send(AMessage::Binary(vec![1, 1]))
                    .await
                    .unwrap();

                continue;
            }

            if messages_received == 3 {
                let ticket = credentials.ticket.clone();

                let mut ticket = ticket.as_bytes().to_owned();

                // reverse the bits
                for i in 0..8 {
                    let c = ticket[i];
                    let mut cs = 0u8;
                    for j in 0..8 {
                        cs |= ((c >> j) & 1) << (7 - j)
                    }
                    ticket[i] = cs;
                }

                let trimmed_ticket: &[u8; 8] = &ticket[0..8].try_into().unwrap();
                let challenge = des::encrypt(&msg.into_data(), &trimmed_ticket);

                remote_sender
                    .lock()
                    .await
                    .send(TMessage::Binary(challenge))
                    .await
                    .unwrap();

                break;
            }

            client_sender
                .lock()
                .await
                .send(convert_tungstenite_to_axum(msg))
                .await
                .unwrap();
        }
    };

    join!(capture_client_messages, capture_remote_messages);
}