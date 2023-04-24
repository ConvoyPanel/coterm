use axum::{
    body::{boxed, Body, BoxBody},
    http::{Request, Response, StatusCode, Uri},
    routing::{get, get_service},
    Router,
};
use std::{net::SocketAddr, path::PathBuf};
use tower::ServiceExt;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new().allow_origin(Any);

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dist");

    // let serve_dir = ServeDir::new("./assets").append_index_html_on_directories(true);

    // let app = Router::new()
    //     .route("/", get(|| async { "Hi from /foo" }))
    //     .fallback_service(serve_dir)
    //     .layer(cors);

    let app = Router::new().nest_service("/", ServeDir::new(assets_dir));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}
