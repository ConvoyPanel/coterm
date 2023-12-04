use axum::{Router, ServiceExt};
use axum::routing::IntoMakeService;
use crate::routes;
use crate::util::broadcast_config::create_assets_service;

#[derive(Clone)]
pub struct AppState {
    pub token_combined: String,
    pub token: String,
    pub token_id: String,
}

pub async fn create_app() -> Router {
    let mut token_combined = dotenv!("TOKEN").split("|");
    let token_id = token_combined.nth(0).expect("Your Coterm configuration is missing a properly formatted token value.");
    let token = token_combined.nth(1).expect("Your Coterm configuration is missing a properly formatted token value.");

    let state = AppState {
        token_combined: dotenv!("TOKEN").to_owned(),
        token: token.to_owned(),
        token_id: token_id.to_owned(),
    };

    Router::new()
        .merge(routes::websocket::create_route())
        .nest_service("/", create_assets_service())
        .with_state(state)
}