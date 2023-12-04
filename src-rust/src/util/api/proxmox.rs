use tokio_tungstenite::tungstenite::handshake::client::{generate_key, Request};
use tokio_tungstenite::tungstenite::http::header::{CONNECTION, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_PROTOCOL, SEC_WEBSOCKET_VERSION, UPGRADE};
use tokio_tungstenite::tungstenite::http::request::Builder;
use crate::util::api::novnc::NoVncCredentials;
use crate::util::api::xtermjs::XTermjsCredentials;
use urlencoding::encode;

pub enum Credentials {
    XTerm(XTermjsCredentials),
    NoVnc(NoVncCredentials),
}

pub fn build_ws_request(credentials: Credentials) -> Builder {
    let mut request = Request::builder()
        .header(SEC_WEBSOCKET_KEY, generate_key())
        .header(SEC_WEBSOCKET_VERSION, "13")
        .header(SEC_WEBSOCKET_PROTOCOL, "binary")
        .header(CONNECTION, "Upgrade")
        .header(UPGRADE, "websocket");

    match credentials {
        Credentials::NoVnc(credentials) => {
            let path = format!(
                "wss://{node_fqdn}:{node_port}/api2/json/nodes/{node}/qemu/{vmid}/vncwebsocket?port={vnc_port}&vncticket={ticket}",
                node_fqdn = credentials.node_fqdn,
                node_port = credentials.node_port,
                node = credentials.node_pve_name,
                vmid = credentials.vmid,
                vnc_port = credentials.port,
                ticket = encode(&credentials.ticket)
            );

            request = request
                .header("Cookie", format!("PVEAuthCookie={}", credentials.pve_auth_cookie))
                .header("Host", format!("{}:{}", credentials.node_fqdn, credentials.node_port))
                .uri(path);
        }
        Credentials::XTerm(credentials) => {
            let path = format!(
                "wss://{node_fqdn}:{node_port}/api2/json/nodes/{node}/qemu/{vmid}/vncwebsocket?port={vnc_port}&vncticket={ticket}",
                node_fqdn = credentials.node_fqdn,
                node_port = credentials.node_port,
                node = credentials.node_pve_name,
                vmid = credentials.vmid,
                vnc_port = credentials.port,
                ticket = encode(&credentials.ticket)
            );

            request = request
                .header("Cookie", format!("PVEAuthCookie={}", credentials.pve_auth_cookie))
                .header("Host", format!("{}:{}", credentials.node_fqdn, credentials.node_port))
                .uri(path);
        }
    }

    request
}