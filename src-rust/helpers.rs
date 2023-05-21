use base64;
use base64::{engine::general_purpose, Engine as _};
use httparse::{Header, Request, EMPTY_HEADER};
use rand::Rng;
use urlencoding::encode;

use crate::no_vnc::NoVncCredentials;
use crate::xterm::XTermCredentials;

enum Credentials {
    XTerm(XTermCredentials),
    NoVnc(NoVncCredentials),
}

pub fn create_request(creds: Credentials) {
    let mut placeholder_headers = [EMPTY_HEADER; 16];
    let mut remote_request = Request::new(&mut placeholder_headers);

    remote_request.method = Some("GET");
    remote_request.version = Some(1);
    let websocket_key = generate_websocket_key();

    let mut headers = vec![
        Header {
            name: "sec-websocket-key",
            value: &websocket_key.as_bytes(),
        },
        Header {
            name: "sec-websocket-version",
            value: b"13",
        },
        Header {
            name: "connection",
            value: b"Upgrade",
        },
        Header {
            name: "upgrade",
            value: b"websocket",
        },
        Header {
            name: "sec-websocket-protocol",
            value: b"binary",
        },
    ];

    match creds {
        Credentials::XTerm(creds) => {
            let path = format!(
                "wss://{}:{}/api2/json/nodes/{}/qemu/{}/termproxy?port={}&vncticket={}",
                creds.node_fqdn,
                creds.node_port,
                creds.node_pve_name,
                creds.vmid,
                creds.port,
                encode(&creds.ticket)
            );
            let cookie = format!("PVEAuthCookie={}", creds.pve_auth_cookie);
            let host = format!("{}:{}", creds.node_fqdn, creds.node_port);

            headers.append(&mut vec![
                Header {
                    name: "host",
                    value: &host.to_owned().as_bytes().to_owned(),
                },
                Header {
                    name: "cookie",
                    value: cookie.as_bytes(),
                },
            ]);

            remote_request.path = Some(&path);
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
            let cookie = format!("PVEAuthCookie={}", creds.pve_auth_cookie);
            let host = format!("{}:{}", creds.node_fqdn, creds.node_port);

            headers.append(&mut vec![
                Header {
                    name: "host",
                    value: &host.as_bytes().to_owned(),
                },
                Header {
                    name: "cookie",
                    value: cookie.as_bytes(),
                },
            ]);

            remote_request.path = Some(&path);
        }
    }

    remote_request.headers = &mut headers;
}

fn generate_websocket_key() -> String {
    let mut rng = rand::thread_rng();
    let mut key_bytes = [0u8; 16];
    rng.fill(&mut key_bytes);

    general_purpose::STANDARD_NO_PAD.encode(&key_bytes)
}
