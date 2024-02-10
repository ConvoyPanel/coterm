use std::sync::Arc;

use dotenv::var;
use rustls::{ClientConfig, RootCertStore};
use rustls_native_certs::load_native_certs;
use tokio_tungstenite::Connector;
use tokio_tungstenite::tungstenite::handshake::client::{generate_key, Request};
use tokio_tungstenite::tungstenite::http::header::{CONNECTION, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_PROTOCOL, SEC_WEBSOCKET_VERSION, UPGRADE};
use tracing::{debug, debug_span};
use urlencoding::encode;

use crate::util::api::novnc::NoVncCredentials;
use crate::util::api::xtermjs::XTermjsCredentials;
use crate::util::crypto::rustls::NoCertificateVerification;

pub enum Credentials {
    XTerm(XTermjsCredentials),
    NoVnc(NoVncCredentials),
}

pub fn build_ws_request(credentials: Credentials) -> (Request, Connector) {
    let span = debug_span!("Constructing WS request");

    span.in_scope(|| {
        let mut request = Request::builder()
            .header(SEC_WEBSOCKET_KEY, generate_key())
            .header(SEC_WEBSOCKET_VERSION, "13")
            .header(SEC_WEBSOCKET_PROTOCOL, "binary")
            .header(CONNECTION, "Upgrade")
            .header(UPGRADE, "websocket");
        debug!("Request headers constructed");

        let do_not_verify_tls = var("DANGEROUS_DISABLE_TLS_VERIFICATION")
            .unwrap_or("false".to_string()).parse::<bool>().unwrap_or(false);
        debug!("TLS verification disabled: {do_not_verify_tls}", do_not_verify_tls = do_not_verify_tls);

        let tls_connector = if do_not_verify_tls {
            ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoCertificateVerification {}))
                .with_no_client_auth()
        } else {
            let mut roots = RootCertStore::empty();
            for cert in load_native_certs().expect("Failed to load native certs") {
                roots.add(cert).unwrap();
            }

            ClientConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth()
        };

        let tls_connector = Connector::Rustls(Arc::new(tls_connector));
        debug!("TLS connector constructed");

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

        debug!("WS request constructed");

        (request, tls_connector)
    })
}