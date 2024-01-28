use dotenv::var;
use native_tls::TlsConnector;
use tokio_tungstenite::Connector;
use tokio_tungstenite::tungstenite::handshake::client::{generate_key, Request};
use tokio_tungstenite::tungstenite::http::header::{CONNECTION, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_PROTOCOL, SEC_WEBSOCKET_VERSION, UPGRADE};
use urlencoding::encode;

use crate::util::api::novnc::NoVncCredentials;
use crate::util::api::xtermjs::XTermjsCredentials;

pub enum Credentials {
    XTerm(XTermjsCredentials),
    NoVnc(NoVncCredentials),
}

pub fn build_ws_request(credentials: Credentials) -> (Request, Connector) {
    let mut request = Request::builder()
        .header(SEC_WEBSOCKET_KEY, generate_key())
        .header(SEC_WEBSOCKET_VERSION, "13")
        .header(SEC_WEBSOCKET_PROTOCOL, "binary")
        .header(CONNECTION, "Upgrade")
        .header(UPGRADE, "websocket");

    let do_not_verify_tls = var("DANGEROUS_DISABLE_TLS_VERIFICATION")
        .unwrap_or("false".to_string())
        .to_lowercase() == "true";
    let tls_connector = TlsConnector::builder()
        .danger_accept_invalid_hostnames(do_not_verify_tls)
        .danger_accept_invalid_certs(do_not_verify_tls)
        .build()
        .unwrap();
    let tls_connector = Connector::NativeTls(tls_connector);

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

    let request = request.body(()).unwrap();

    (request, tls_connector)
}