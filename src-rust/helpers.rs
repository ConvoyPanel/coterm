use base64;
use base64::{engine::general_purpose, Engine as _};
use httparse::{Header, Request, EMPTY_HEADER};
use rand::Rng;
use urlencoding::encode;

use crate::no_vnc::NoVncCredentials;
use crate::xterm::XTermCredentials;

pub enum Credentials {
    XTerm(XTermCredentials),
    NoVnc(NoVncCredentials),
}


pub fn create_request(creds: Credentials) -> Request<'static, 'static> {
    let mut placeholder_headers = [EMPTY_HEADER; 16];
    let mut request = Request::new(&mut placeholder_headers);

    request.method = Some("GET");
    request.version = Some(1);
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
    let mut path = String::new();
    let mut cookie = String::new();
    let mut host = String::new();

    match creds {
        Credentials::XTerm(creds) => {
            path = format!(
                "wss://{}:{}/api2/json/nodes/{}/qemu/{}/termproxy?port={}&vncticket={}",
                creds.node_fqdn,
                creds.node_port,
                creds.node_pve_name,
                creds.vmid,
                creds.port,
                encode(&creds.ticket)
            );
            cookie = format!("PVEAuthCookie={}", creds.pve_auth_cookie);
            host = format!("{}:{}", creds.node_fqdn, creds.node_port);

            headers.append(&mut vec![
                Header {
                    name: "host",
                    value: host.as_bytes(),
                },
                Header {
                    name: "cookie",
                    value: cookie.as_bytes(),
                },
            ]);

            request.path = Some(&path);
        }
        Credentials::NoVnc(creds) => {
            path = format!(
                "wss://{}:{}/api2/json/nodes/{}/qemu/{}/vncwebsocket?port={}&vncticket={}",
                creds.node_fqdn,
                creds.node_port,
                creds.node_pve_name,
                creds.vmid,
                creds.port,
                encode(&creds.ticket)
            );
            cookie = format!("PVEAuthCookie={}", creds.pve_auth_cookie);
            host = format!("{}:{}", creds.node_fqdn, creds.node_port);

            headers.append(&mut vec![
                Header {
                    name: "host",
                    value: host.as_bytes(),
                },
                Header {
                    name: "cookie",
                    value: cookie.as_bytes(),
                },
            ]);

            request.path = Some(&path);
        }
    }

    request.headers = &mut headers;

    request
}

fn generate_websocket_key() -> String {
    let mut rng = rand::thread_rng();
    let mut key_bytes = [0u8; 16];
    rng.fill(&mut key_bytes);

    general_purpose::STANDARD_NO_PAD.encode(&key_bytes)
}
