use axum::extract::ws::WebSocket;
use futures_util::{SinkExt, StreamExt};
use tokio::join;
use tokio_tungstenite::tungstenite::Message as TMessage;

use crate::util::api::proxmox::{build_ws_request, Credentials};
use crate::util::api::xtermjs::create_xtermjs_credentials;
use crate::util::websocket::{convert_axum_to_tungstenite, convert_tungstenite_to_axum};

pub async fn start_xtermjs_proxy(server_uuid: String, client_ws: WebSocket) {
    let credentials = create_xtermjs_credentials(server_uuid).await.unwrap();

    let (remote_ws, _) = tokio_tungstenite::connect_async(
        build_ws_request(
            Credentials::XTerm(credentials.clone())
        ).body(()).unwrap()
    ).await.unwrap();

    let (mut client_sender, mut client_receiver) = client_ws.split();
    let (mut remote_sender, mut remote_receiver) = remote_ws.split();

    let payload = format!("{username}@{realm_type}:{ticket}\n", username = credentials.username, realm_type = credentials.realm_type, ticket = credentials.ticket);
    remote_sender.send(TMessage::Text(payload)).await.unwrap();


    let client_to_remote = async {
        while let Some(Ok(msg)) = client_receiver.next().await {
            remote_sender.send(convert_axum_to_tungstenite(msg)).await.unwrap();
        }

        remote_sender.close().await.unwrap();
    };

    let remote_to_client = async {
        while let Some(Ok(msg)) = remote_receiver.next().await {
            client_sender.send(convert_tungstenite_to_axum(msg)).await.unwrap();
        }

        client_sender.close().await.unwrap();
    };

    join!(client_to_remote, remote_to_client);
}