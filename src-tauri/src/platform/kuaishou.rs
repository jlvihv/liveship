use crate::{
    model::{LiveInfo, PlatformKind},
    recorder::Recorder,
    request,
};
use anyhow::{Ok, Result};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};

pub struct Kuaishou;

#[async_trait]
impl Recorder for Kuaishou {
    async fn get_live_info(&self, url: &str) -> Result<LiveInfo> {
        let body = request::get_with_headers(url, Self::headers()?)
            .await?
            .text()
            .await?;
        println!("kuaishou html body: {:#?}", body);
        let re =
            regex::Regex::new(r#"<script>window.__INITIAL_STATE__=(.*?);\(function\(\)\{var s;"#)?;
        let json_str = re
            .captures(&body)
            .ok_or_else(|| anyhow::anyhow!("No match"))?
            .get(1)
            .ok_or_else(|| anyhow::anyhow!("No match"))?
            .as_str();
        let json: serde_json::Value = serde_json::from_str(json_str)?;
        println!("{:#?}", json);
        todo!()
    }

    fn kind(&self) -> PlatformKind {
        PlatformKind::Kuaishou
    }
}

impl Kuaishou {
    fn headers() -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            HeaderValue::from_str(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36 Edg/114.0.1823.79"
            )?,
        );
        headers.insert("Accept", HeaderValue::from_str("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")?);
        Ok(headers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_stream_info() {
        // let url = "https://live.kuaishou.com/u/yall1102";
        let url = "https://live.kuaishou.com/u/my6677766";
        let json_data = Kuaishou.get_live_info(url).await.unwrap();
        println!("{:#?}", json_data);
    }
}
