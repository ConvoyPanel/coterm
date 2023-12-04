use axum::Router;
use dotenv::var;

use crate::routes;
use crate::util::broadcast_config::create_assets_service;

#[derive(Clone)]
pub struct AppState {
    pub token_combined: String,
    pub token: String,
    pub token_id: String,
}

pub async fn create_app() -> Router {
    let token_env = var("TOKEN").expect("TOKEN is not set.");
    let token_cloned = token_env.clone();
    let mut token_combined = token_cloned.split("|");
    let token_id = token_combined.next().expect("Your Coterm configuration is missing a properly formatted token value.");
    let token = token_combined.next().expect("Your Coterm configuration is missing a properly formatted token value.");

    let state = AppState {
        token_combined: token_env,
        token: token.to_owned(),
        token_id: token_id.to_owned(),
    };

    Router::new()
        .merge(routes::websocket::create_route())
        .nest_service("/", create_assets_service())
        .with_state(state)
}