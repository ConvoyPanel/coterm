use std::collections::HashMap;
use dotenv::var;

use reqwest::Client;
use serde::Deserialize;

use crate::util::api::http::get_headers_with_authorization;

#[derive(Deserialize, Debug, Clone)]
pub struct XTermjsCredentials {
    pub node_fqdn: String,
    pub node_port: u32,
    pub node_pve_name: String,
    pub vmid: u32,
    pub port: u32,
    pub ticket: String,
    pub username: String,
    pub realm_type: String,
    pub pve_auth_cookie: String,
}

pub async fn create_xtermjs_credentials(server_uuid: String) -> Result<XTermjsCredentials, reqwest::Error> {
    let mut body = HashMap::new();
    body.insert("type".to_owned(), "xtermjs".to_owned());

    let response = Client::new()
        .post(
            format!(
                "{convoy_url}/api/coterm/servers/{uuid}/create-console-session",
                convoy_url = var("CONVOY_URL").unwrap(),
                uuid = server_uuid
            )
        )
        .json(&body)
        .headers(get_headers_with_authorization())
        .send()
        .await
        .unwrap();

    if response.status().is_success() {
        // TODO: MUST READ FROM NESTED DATA PROPERTY
        let response_body = response.json::<XTermjsCredentials>().await.unwrap();
        Ok(response_body)
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}