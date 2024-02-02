use std::collections::HashMap;
use dotenv::var;

use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, debug_span, Instrument};

use crate::util::api::http::get_headers_with_authorization;

#[derive(Deserialize, Debug, Clone)]
pub struct NoVncCredentials {
    pub node_fqdn: String,
    pub node_port: u32,
    pub node_pve_name: String,
    pub vmid: u32,
    pub port: u32,
    pub ticket: String,
    pub pve_auth_cookie: String,
}

pub async fn create_novnc_credentials(server_uuid: String) -> Result<NoVncCredentials, reqwest::Error> {
    async {
        debug!("Begin creating noVNC creds");
        let mut body = HashMap::new();
        body.insert("type".to_owned(), "novnc".to_owned());

        let response = Client::new()
            .post(
                format!(
                    "{convoy_url}/api/coterm/servers/{uuid}/create-console-session",
                    convoy_url = var("CONVOY_URL").expect("CONVOY_URL is not set."),
                    uuid = server_uuid
                )
            )
            .json(&body)
            .headers(get_headers_with_authorization())
            .send()
            .await
            .unwrap();

        if response.status().is_success() {
            let data: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
            let credentials: NoVncCredentials = serde_json::from_value(data["data"].clone()).unwrap();

            debug!("NoVNC creds created");
            Ok(credentials)
        } else {
            debug!("Failed to create NoVNC creds");
            Err(response.error_for_status().unwrap_err())
        }
    }.instrument(debug_span!("Getting NoVNC creds {server_uuid}", server_uuid = server_uuid)).await
}