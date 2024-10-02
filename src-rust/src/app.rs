use axum::Router;
use dotenv::var;
use tracing::warn;
use url::Url;
use crate::routes;
use crate::util::broadcast_config::create_assets_service;

#[derive(Clone)]
pub struct AppState {
    pub coterm_url: Url,
    pub convoy_url: Url,
    pub token_id: String,
    pub token_secret: String,
    pub tls_verification_disabled: bool,
}

pub fn create_app() -> Router {
    let coterm_url = Url::parse(&var("COTERM_URL").expect("COTERM_URL is not set.")).unwrap();
    let convoy_url = Url::parse(&var("CONVOY_URL").expect("CONVOY_URL is not set.")).unwrap();

    let token_str = var("COTERM_TOKEN").expect("COTERM_TOKEN is not set.");
    let mut token = token_str.split("|");
    let token_id = token.next().expect("Your Coterm configuration is missing a properly formatted token value.");
    let token_secret = token.next().expect("Your Coterm configuration is missing a properly formatted token value.");


    let tls_verification_disabled = var("DANGEROUS_DISABLE_TLS_VERIFICATION")
        .unwrap_or("false".to_string()).parse::<bool>().unwrap_or(false);
    if tls_verification_disabled {
        warn!("TLS verification is disabled. This is dangerous and should only be used for testing purposes.\nYou are vulnerable to man-in-the-middle attacks, and this is very irresponsible if you are providing this for end users.");
    }

    let state = AppState {
        coterm_url,
        convoy_url,
        token_id: token_id.to_owned(),
        token_secret: token_secret.to_owned(),
        tls_verification_disabled,
    };

    Router::new()
        .merge(routes::websocket::create_route())
        .nest_service("/", create_assets_service())
        .with_state(state)
}