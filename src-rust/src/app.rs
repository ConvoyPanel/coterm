use axum::Router;
use dotenv::var;
use tracing::warn;

use crate::routes;
use crate::util::broadcast_config::create_assets_service;

#[derive(Clone)]
pub struct AppState {
    pub token_combined: String,
    pub token: String,
    pub token_id: String,
}

pub async fn create_app() -> Router {
    let token_env = var("COTERM_TOKEN").expect("COTERM_TOKEN is not set.");
    let token_cloned = token_env.clone();
    let mut token_combined = token_cloned.split("|");
    let token_id = token_combined.next().expect("Your Coterm configuration is missing a properly formatted token value.");
    let token = token_combined.next().expect("Your Coterm configuration is missing a properly formatted token value.");

    let do_not_verify_tls = var("DANGEROUS_DISABLE_TLS_VERIFICATION")
        .unwrap_or("false".to_string()).parse::<bool>().unwrap_or(false);
    if do_not_verify_tls {
        warn!("TLS verification is disabled. This is dangerous and should only be used for testing purposes.\nYou are vulnerable to man-in-the-middle attacks, and this is very irresponsible if you are providing this for end users.");
    }

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