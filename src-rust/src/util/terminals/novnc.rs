use axum::extract::ws::WebSocket;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message as TMessage, MaybeTlsStream, WebSocketStream};
use crate::util::api::novnc::{create_novnc_credentials, NoVncCredentials};
use crate::util::api::proxmox::{build_ws_request, Credentials};
use axum::extract::ws::Message as AMessage;
use tokio::join;
use crate::util::crypto::des;
use crate::util::websocket::{convert_axum_to_tungstenite, convert_tungstenite_to_axum};

pub async fn start_novnc_proxy(server_uuid: String, client_ws: WebSocket) {
    let credentials = create_novnc_credentials(server_uuid).await.unwrap();
    let (remote_ws, _) = tokio_tungstenite::connect_async(
        build_ws_request(
            Credentials::NoVnc(credentials.clone())
        ).body(()).unwrap()
    ).await.unwrap();

    authenticate(&client_ws, &remote_ws, credentials).await;


    let (mut client_sender, mut client_receiver) = client_ws.split();
    let (mut remote_sender, mut remote_receiver) = remote_ws.split();

    let client_to_remote = async {
        while let Some(Ok(msg)) = client_receiver.next().await {
            remote_sender.send(convert_axum_to_tungstenite(msg)).await.unwrap();
        }
    };

    let remote_to_client = async {
        while let Some(Ok(msg)) = remote_receiver.next().await {
            client_sender.send(convert_tungstenite_to_axum(msg)).await.unwrap();
        }
    };

    join!(client_to_remote, remote_to_client);
}

async fn authenticate(client_ws: &WebSocket, remote_ws: &WebSocketStream<MaybeTlsStream<TcpStream>>, credentials: NoVncCredentials) {
    let (mut client_sender, mut client_receiver) = client_ws.split();
    let (mut remote_sender, mut remote_receiver) = remote_ws.split();

    let capture_client_messages = async {
        while let Some(Ok(msg)) = client_receiver.next().await {
            if msg == AMessage::Binary(vec![1]) {
                break;
            }
        }
    };

    let capture_remote_messages = async {
        let mut messages_received = 0;

        while let Some(Ok(msg)) = remote_receiver.next().await {
            if messages_received < 5 { // TODO: change this to 3
                messages_received += 1;
            }

            if msg == TMessage::Binary(vec![1, 2]) {
                remote_sender.send(TMessage::Binary(vec![2]))
                    .await
                    .unwrap();
                client_sender.send(AMessage::Binary(vec![1, 1])).await.unwrap();
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
                    .send(TMessage::Binary(challenge))
                    .await
                    .unwrap();

                break;
            }
        }
    };

    join!(capture_client_messages, capture_remote_messages);
}