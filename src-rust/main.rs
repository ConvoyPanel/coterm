use axum::extract::ws::{CloseFrame as ACloseFrame, Message as AMessage};
use axum::http::HeaderMap;
use axum::{
    body::{boxed, Body, BoxBody},
    extract::ws::{WebSocket, WebSocketUpgrade},
    http::{Response, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Router,
};
use base64;
use base64::{engine::general_purpose, Engine as _};
use dotenv::dotenv;
use futures_util::{sink::SinkExt, stream::StreamExt};
use httparse::{Header, Request, EMPTY_HEADER};
use rand::Rng;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use url::form_urlencoded::byte_serialize;
use std::{net::SocketAddr, path::PathBuf};
use tokio::join;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        protocol::frame::coding::CloseCode as TCloseCode, protocol::CloseFrame as TCloseFrame,
        Message as TMessage,
    },
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tokio_tungstenite::tungstenite::Error as TWebSocketError;



#[tokio::main]
async fn main() {
    let cors = CorsLayer::new().allow_origin(Any);

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dist");

    let app = Router::new()
        .route("/ws", get(ws_upgrader_handler))
        .route("/test", get(|| async { "Hi from /foo" }))
        .nest_service("/", ServeDir::new(assets_dir))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ws_upgrader_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(client_socket: WebSocket) {
    let credentials = create_no_vnc_credentials().await.unwrap();
    let (mut client_sender, mut client_receiver) = client_socket.split();

    let path = format!("wss://{}:{}/api2/json/nodes/{}/qemu/{}/vncwebsocket?port={}&vncticket={}", credentials.node_fqdn, credentials.node_port, credentials.node_pve_name, credentials.vmid, credentials.port, credentials.ticket);
    let mut headers = [EMPTY_HEADER; 16];
    let mut remote_request = Request::new(&mut headers);
    remote_request.method = Some("GET");
    remote_request.version = Some(1);
    remote_request.path = Some(&path);
    let websocket_key = generate_websocket_key();
    // encode the pve_auth_cookie in the cookie encodeURIComponent equivalent
    let cookie = format!("PVEAuthCookie={}", credentials.pve_auth_cookie);
    println!("cookie: {}", cookie);
    let host = format!("{}:{}", credentials.node_fqdn, credentials.node_port);
    let origin = format!("https://{}:{}", credentials.node_fqdn, credentials.node_port);
    let mut actual_headers = [
        Header {
            name: "sec-websocket-key",
            value: &websocket_key.as_bytes(),
        },
        Header {
            name: "host",
            value: host.as_bytes(),
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
            name: "cookie",
            value: cookie.as_bytes(),
        },
        Header {
            name: "origin",
            value: origin.as_bytes(),
        },
        Header {
            name: "user-agent",
            value: b"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36",
        },
        Header {
            name: "sec-websocket-protocol",
            value: b"binary",
        },
        Header {
            name: "pragma",
            value: b"no-cache",
        }
    ];
    remote_request.headers = &mut actual_headers;

// if it errors, get the body of the error (Http with body p) as text and print it
    let (remote_socket, _) = connect_async(remote_request)
        .await
        // .unwrap();
        .unwrap_or_else(|e| {
            match e {
                TWebSocketError::Http(http_response) => {
                    // Get the HTTP response code and error message
                    let status_code = http_response.status();
                    let error_message = if let Some(error_message) = http_response.into_body() {
                        error_message
                    } else {
                        panic!("Failed to connect to WebSocket fuck: {}", status_code);
                    };

                    panic!("Failed to connect to WebSocket: {} - {}", status_code, String::from_utf8_lossy(&error_message).to_string());
                },
                _ => {
                    // Use default panic for all other error cases
                    panic!("Failed to connect to WebSocket: {}", e);
                }
            }
        });

    let (mut remote_sender, mut remote_receiver) = remote_socket.split();

    // send from client to remote and back
    let client_to_remote = async {
        while let Some(Ok(msg)) = client_receiver.next().await {
            println!("client_to_remote: {:?}", msg);

            remote_sender
                .send(convert_axum_to_tungstenite(msg))
                .await
                .unwrap();
        }
    };

    // send from remote to client and back
    let remote_to_client = async {
        while let Some(Ok(msg)) = remote_receiver.next().await {
            println!("remote_to_client: {:?}", msg);

            client_sender
                .send(convert_tungstenite_to_axum(msg))
                .await
                .unwrap();
        }
    };

    join!(client_to_remote, remote_to_client);
}

fn convert_axum_to_tungstenite(axum_msg: AMessage) -> TMessage {
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

fn convert_tungstenite_to_axum(tungstenite_msg: TMessage) -> AMessage {
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

fn generate_websocket_key() -> String {
    let mut rng = rand::thread_rng();
    let mut key_bytes = [0u8; 16];
    rng.fill(&mut key_bytes);

    general_purpose::STANDARD_NO_PAD.encode(&key_bytes)
}

#[derive(Deserialize)]
#[derive(Debug)]
struct NoVncCredentials {
    node_fqdn: String,
    node_port: u32,
    node_pve_name: String,
    vmid: u32,
    port: u32,
    ticket: String,
    pve_auth_cookie: String,
}

async fn create_no_vnc_credentials() -> Result<NoVncCredentials, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());
    headers.insert("Authorization", "Bearer redacted".parse().unwrap());
    let client = Client::new();
    let response = client.post("https://redacted/api/coterm/servers/c6c4bc0d-e7d4-467c-ad96-74d8f8c0f2df/create-console-session")
        .headers(headers)
        .send()
        .await.unwrap();

    if response.status().is_success() {
        let data: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        // deserialize the response which is nested in "data"
        let credentials: NoVncCredentials = serde_json::from_value(data["data"].clone()).unwrap();
        println!("Data: {:?}", credentials);
        Ok(credentials)
    } else {
        println!("Error: {:?}", response);
        Err(response.error_for_status().unwrap_err())
    }
}

// async fn login(username: String, password: String) {
//     let client = Client::new();
//     let body = json!({
//         "username": username,
//         "password": password
//     });
//     let mut headers = HeaderMap::new();
//     headers.insert("Content-Type", "application/json".parse().unwrap());
//     headers.insert("Accept", "application/json".parse().unwrap());


//     let response = client
//         .post("https://pve-node.com:8006/api2/json/access/ticket")
//         .headers(headers)
//         .body(body.to_string())
//         .send()
//         .await.unwrap();

//         if response.status().is_success() {
//             let data: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
//             println!("Data: {:?}", data["data"]["username"]);
//         } else {
//             println!("Error: {:?}", response);
//         }
// }
