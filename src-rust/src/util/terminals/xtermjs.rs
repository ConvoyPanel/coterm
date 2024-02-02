use axum::extract::ws::WebSocket;
use futures_util::{SinkExt, StreamExt};
use tokio::join;

use tokio_tungstenite::tungstenite::Message as TMessage;
use tracing::{debug, debug_span, error, Instrument};

use crate::util::api::proxmox::{build_ws_request, Credentials};
use crate::util::api::xtermjs::create_xtermjs_credentials;
use crate::util::websocket::{convert_axum_to_tungstenite, convert_tungstenite_to_axum};

pub async fn start_xtermjs_proxy(server_uuid: String, client_ws: WebSocket) {
    let span = debug_span!("Xterm.js proxy {server_uuid}", server_uuid = server_uuid.clone());

    async move {
        debug!("Starting proxy...");
        let credentials = create_xtermjs_credentials(server_uuid.clone()).await.unwrap();

        let (request, connector) = build_ws_request(
            Credentials::XTerm(credentials.clone())
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

        let (mut client_sender, mut client_receiver) = client_ws.split();
        let (mut remote_sender, mut remote_receiver) = remote_ws.split();

        async {
            debug!("Sending payload");
            let payload = format!(
                "{username}@{realm_type}:{ticket}\n",
                username = credentials.username,
                realm_type = credentials.realm_type,
                ticket = credentials.ticket
            );
            remote_sender.send(TMessage::Text(payload)).await.unwrap();
            debug!("Payload sent");
        }.instrument(tracing::debug_span!("Auth-ing xterm.js connection {server_uuid}", server_uuid = server_uuid)).await;

        let client_to_remote = async {
            debug!("Forwarding client-to-remote...");
            while let Some(Ok(msg)) = client_receiver.next().await {
                remote_sender.send(convert_axum_to_tungstenite(msg)).await.unwrap();
            }

            remote_sender.close().await.unwrap();
        };

        let remote_to_client = async {
            debug!("Forwarding remote-to-client...");
            while let Some(Ok(msg)) = remote_receiver.next().await {
                client_sender.send(convert_tungstenite_to_axum(msg)).await.unwrap();
            }

            client_sender.close().await.unwrap();
        };

        join!(client_to_remote, remote_to_client);

        debug!("Proxy connection closed");
    }.instrument(span).await;
}