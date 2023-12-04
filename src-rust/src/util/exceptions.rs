use axum::body::{Body, Bytes};
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;

pub struct DisplayError {
    pub status: StatusCode,
    pub message: &'static str,
}

impl IntoResponse for DisplayError {
    fn into_response(self) -> Response<Body> {
        let body = Body::from(Bytes::from_static(self.message.as_bytes()));

        axum::http::Response::builder()
            .status(self.status)
            .body(body)
            .unwrap()
    }
}