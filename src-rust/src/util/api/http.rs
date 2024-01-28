use dotenv::var;
use reqwest::header::HeaderMap;

pub fn get_headers_with_authorization() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());
    headers.insert("Authorization", format!("Bearer {}", var("COTERM_TOKEN").expect("COTERM_TOKEN is not set.")).parse().unwrap());

    headers
}