use std::collections::HashMap;

use reqwest::Client;
use serde::Deserialize;

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
    let mut body = HashMap::new();
    body.insert("type".to_owned(), "novnc".to_owned());

    let response = Client::new()
        .post(
            format!(
                "{convoy_url}/api/coterm/servers/{uuid}/create-console-session",
                convoy_url = dotenv!("CONVOY_URL"),
                uuid = server_uuid
            )
        )
        .json(&body)
        .headers(get_headers_with_authorization())
        .send()
        .await
        .unwrap();

    if response.status().is_success() {
        let response_body = response.json::<NoVncCredentials>().await.unwrap();
        Ok(response_body)
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}