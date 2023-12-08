use axum::extract::{State, WebSocketUpgrade};
use axum::extract::ws::WebSocket;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::util::terminals::novnc::start_novnc_proxy;
use crate::util::terminals::xtermjs::start_xtermjs_proxy;

pub fn create_route() -> Router<AppState> {
    Router::new()
        .route("/ws", get(start_ws_session))
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    server_uuid: String,
    console_type: String,
}

async fn start_ws_session(ws: WebSocketUpgrade, State(state): State<AppState>, jar: CookieJar) -> Result<impl IntoResponse, StatusCode> {
    let jwt = jar.get("token")
        .map(|cookie| cookie.value().to_owned())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let decoding_key = DecodingKey::from_secret(state.token.as_ref());
    if let Ok(jwt) = decode::<Claims>(&jwt, &decoding_key, &Validation::default()) {
        match jwt.claims.console_type.as_str() {
            "novnc" => {
                return Ok(ws.on_upgrade(|ws: WebSocket| { start_novnc_proxy(jwt.claims.server_uuid, ws) }));
            }
            "xtermjs" => {
                return Ok(ws.on_upgrade(|ws: WebSocket| { start_xtermjs_proxy(jwt.claims.server_uuid, ws) }));
            }
            _ => {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }
}