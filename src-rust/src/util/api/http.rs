use reqwest::header::HeaderMap;

pub fn get_headers_with_authorization() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());
    headers.insert("Authorization", format!("Bearer {}", dotenv!("TOKEN")).parse().unwrap());

    headers
}