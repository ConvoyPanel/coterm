use axum::http::request::Builder;
use axum::http::Request;
use base64;
use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
use reqwest::header::{
    CONNECTION, COOKIE, HOST, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_PROTOCOL, SEC_WEBSOCKET_VERSION,
    UPGRADE,
};
use urlencoding::encode;
use axum::extract::ws::{CloseFrame as ACloseFrame, Message as AMessage};
use tokio_tungstenite::tungstenite::Error as TWebSocketError;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        protocol::frame::coding::CloseCode as TCloseCode, protocol::CloseFrame as TCloseFrame,
        Message as TMessage,
    },
};

use crate::no_vnc::NoVncCredentials;
use crate::xterm::XTermCredentials;

pub enum Credentials {
    XTerm(XTermCredentials),
    NoVnc(NoVncCredentials),
}

pub fn create_request(creds: Credentials) -> Builder {
    let mut request = Request::builder()
        .header(SEC_WEBSOCKET_KEY, generate_websocket_key())
        .header(SEC_WEBSOCKET_VERSION, "13")
        .header(SEC_WEBSOCKET_PROTOCOL, "binary")
        .header(CONNECTION, "Upgrade")
        .header(UPGRADE, "websocket");

    match creds {
        Credentials::XTerm(creds) => {
            let path = format!(
                "wss://{}:{}/api2/json/nodes/{}/qemu/{}/vncwebsocket?port={}&vncticket={}",
                creds.node_fqdn,
                creds.node_port,
                creds.node_pve_name,
                creds.vmid,
                creds.port,
                encode(&creds.ticket)
            );
            request = request
                .header(COOKIE, format!("PVEAuthCookie={}", creds.pve_auth_cookie))
                .header(HOST, format!("{}:{}", creds.node_fqdn, creds.node_port))
                .uri(path);
        }
        Credentials::NoVnc(creds) => {
            let path = format!(
                "wss://{}:{}/api2/json/nodes/{}/qemu/{}/vncwebsocket?port={}&vncticket={}",
                creds.node_fqdn,
                creds.node_port,
                creds.node_pve_name,
                creds.vmid,
                creds.port,
                encode(&creds.ticket)
            );
            request = request
                .header(COOKIE, format!("PVEAuthCookie={}", creds.pve_auth_cookie))
                .header(HOST, format!("{}:{}", creds.node_fqdn, creds.node_port))
                .uri(path);
        }
    }

    request
}

fn generate_websocket_key() -> String {
    let mut rng = rand::thread_rng();
    let mut key_bytes = [0u8; 16];
    rng.fill(&mut key_bytes);

    general_purpose::STANDARD_NO_PAD.encode(&key_bytes)
}


pub fn convert_axum_to_tungstenite(axum_msg: AMessage) -> TMessage {
    match axum_msg {
        AMessage::Binary(payload) => TMessage::Binary(payload.to_vec()),
        AMessage::Text(payload) => TMessage::Text(payload),
        AMessage::Ping(payload) => TMessage::Ping(payload),
        AMessage::Pong(payload) => TMessage::Pong(payload),
        AMessage::Close(Some(ACloseFrame { code, reason })) => {
            let close_frame = TCloseFrame {
                code: TCloseCode::from(code),
                reason: reason.into(),
            };
            TMessage::Close(Some(TCloseFrame::from(close_frame)))
        }
        AMessage::Close(None) => TMessage::Close(None),
    }
}

pub fn convert_tungstenite_to_axum(tungstenite_msg: TMessage) -> AMessage {
    match tungstenite_msg {
        TMessage::Binary(payload) => AMessage::Binary(payload.into()),
        TMessage::Text(payload) => AMessage::Text(payload),
        TMessage::Ping(payload) => AMessage::Ping(payload.into()),
        TMessage::Pong(payload) => AMessage::Pong(payload.into()),
        TMessage::Close(payload) => {
            let (code, reason) = payload
                .map(|c| (c.code, c.reason))
                .unwrap_or((TCloseCode::from(1000), String::new().into()));
            let close_frame = ACloseFrame {
                code: code.into(),
                reason,
            };
            AMessage::Close(Some(close_frame))
        }
        TMessage::Frame(_) => panic!("Frame messages are not supported"),
    }
}