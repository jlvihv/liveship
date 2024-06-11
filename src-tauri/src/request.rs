use anyhow::Result;
use reqwest::header::HeaderMap;
use reqwest::{Body, Response};

pub async fn get(url: &str, headers: HeaderMap) -> Result<Response> {
    let client = reqwest::Client::new();
    let req = client.get(url).headers(headers);
    let resp = req.send().await?;
    Ok(resp)
}

pub async fn post<T: Into<Body>>(url: &str, headers: HeaderMap, body: T) -> Result<Response> {
    let client = reqwest::Client::new();
    let req = client.post(url).headers(headers).body(body);
    let resp = req.send().await?;
    Ok(resp)
}

pub async fn try_get(url: &str, headers: HeaderMap, timeout: u64) -> Result<Response> {
    let client = reqwest::Client::new();
    // Set timeout
    let req = client
        .get(url)
        .headers(headers)
        .timeout(std::time::Duration::from_secs(timeout));
    let resp = req.send().await?;
    Ok(resp)
}
