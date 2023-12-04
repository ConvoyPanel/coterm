use axum::extract::ws::{CloseFrame as ACloseFrame, Message as AMessage};
use tokio_tungstenite::{
    tungstenite::{
        Message as TMessage, protocol::CloseFrame as TCloseFrame,
        protocol::frame::coding::CloseCode as TCloseCode,
    },
};

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