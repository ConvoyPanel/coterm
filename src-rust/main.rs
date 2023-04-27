use axum::extract::ws::{CloseFrame as ACloseFrame, Message as AMessage};
use axum::{
    body::{boxed, Body, BoxBody},
    extract::ws::{WebSocket, WebSocketUpgrade},
    http::{Response, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use httparse::{Header, Request, EMPTY_HEADER};
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

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new().allow_origin(Any);

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dist");

    let app = Router::new()
        .route("/demo", get(demo))
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

async fn demo(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|mut socket: WebSocket| async move {
        while let Some(msg) = socket.recv().await {
            let msg = if let Ok(msg) = msg {
                msg
            } else {
                // client disconnected
                return;
            };

            println!("demo ws msg received: {:?}", msg);

            if socket
                .send("please don't contact me again. I'm on the no-call list".into())
                .await
                .is_err()
            {
                // client disconnected
                return;
            }
        }
    })
}

async fn ws_upgrader_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(client_socket: WebSocket) {
    let (mut client_sender, mut client_receiver) = client_socket.split();

    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut remote_request = Request::new(&mut headers);
    remote_request.method = Some("GET");
    remote_request.version = Some(1);
    remote_request.path = Some("ws://localhost:3000/demo");
    let mut actual_headers = [Header {
        name: "sec-websocket-key",
        value: b"xI6cUQeu/u73MZN6C0ChtQ==",
    },
    Header {
        name: "host",
        value: b"localhost:3000",
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
    ];
    remote_request.headers = &mut actual_headers;

    let (remote_socket, _) = connect_async(remote_request)
        .await
        .expect("Failed to connect to remote websocket");
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
