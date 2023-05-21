use serde::{Deserialize};

#[derive(Deserialize, Debug)]
pub struct XTermCredentials {
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