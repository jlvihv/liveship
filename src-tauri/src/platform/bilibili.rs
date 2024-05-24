use crate::{
    model::{LiveInfo, PlatformKind},
    recorder::Recorder,
    request,
};
use anyhow::{Ok, Result};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};

pub struct Bilibili;

#[async_trait]
impl Recorder for Bilibili {
    async fn get_live_info(&self, url: &str) -> Result<LiveInfo> {
        let body = request::get_with_headers(url, Self::headers()?)
            .await?
            .text()
            .await?;
        println!("{:#?}", body);
        let re = regex::Regex::new(
            r#"window.__NEPTUNE_IS_MY_WAIFU__=(.*?)window.__NEPTUNE_IS_MY_WAIFU__"#,
        )?;
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
        PlatformKind::Bilibili
    }
}

impl Bilibili {
    fn headers() -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            HeaderValue::from_str(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/115.0",
            )?,
        );
        headers.insert("Accept", HeaderValue::from_str("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")?);
        headers.insert(
            "Referer",
            HeaderValue::from_str("https://live.bilibili.com/?spm_id_from=333.1296.0.0")?,
        );
        headers.insert("Cookie", HeaderValue::from_str("bilibili.com")?);
        Ok(headers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_stream_info() {
        let url = "https://live.bilibili.com/27210217";
        let json_data = Bilibili.get_live_info(url).await.unwrap();
        println!("{:#?}", json_data);
    }
}
