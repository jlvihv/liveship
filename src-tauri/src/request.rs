use anyhow::Result;
use once_cell::sync::OnceCell;
use reqwest::header::HeaderMap;
use reqwest::Response;

static CLIENT: OnceCell<reqwest::Client> = OnceCell::new();

pub async fn get_with_headers(url: &str, headers: HeaderMap) -> Result<Response> {
    let client = CLIENT.get_or_init(|| reqwest::Client::new());
    let req = client.get(url).headers(headers);
    let resp = req.send().await?;
    Ok(resp)
}

pub async fn new_client_get_with_headers(url: &str, headers: HeaderMap) -> Result<Response> {
    let client = reqwest::Client::new();
    let req = client.get(url).headers(headers);
    let resp = req.send().await?;
    Ok(resp)
}
