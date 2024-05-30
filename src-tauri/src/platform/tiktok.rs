use crate::{
    model::{JsonValue, LiveInfo, LiveStatus, PlatformKind, Stream, StreamingProtocol},
    recorder::Recorder,
    request,
};
use anyhow::{Ok, Result};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};

pub struct Tiktok;

#[async_trait]
impl Recorder for Tiktok {
    async fn get_live_info(&self, url: &str) -> Result<LiveInfo> {
        let body = request::get_with_headers(url, Self::headers()?)
            .await?
            .text()
            .await?;
        let re = regex::Regex::new(
            r#"<script id="SIGI_STATE" type="application/json">(.*?)</script><script id="SIGI_RETRY" type="application/json">"#,
        )?;
        let json_str = re
            .captures(&body)
            .ok_or_else(|| anyhow::anyhow!("No match"))?
            .get(1)
            .ok_or_else(|| anyhow::anyhow!("No match"))?
            .as_str();
        let json: serde_json::Value = serde_json::from_str(json_str)?;
        let live_room = json
            .pointer("/LiveRoom/liveRoomUserInfo")
            .ok_or_else(|| anyhow::anyhow!("live room user info not found"))?;
        let user = live_room
            .get("user")
            .ok_or_else(|| anyhow::anyhow!("user not found"))?
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("user is not an object"))?;
        let anchor_name = user
            .get("nickname")
            .ok_or_else(|| anyhow::anyhow!("nickname not found"))?
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("nickname is not a string"))?;
        let status = user
            .get("status")
            .ok_or_else(|| anyhow::anyhow!("status not found"))?
            .as_number()
            .ok_or_else(|| anyhow::anyhow!("status is not a number"))?
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("status is not an integer"))?;
        // status 2 表示正在直播
        if status == 2 {
            let stream_data = live_room
                .pointer("/liveRoom/streamData/pull_data/stream_data")
                .ok_or_else(|| anyhow::anyhow!("stream data not found"))?
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("stream data is not a string"))?;
            let stream_data: JsonValue = serde_json::from_str(stream_data)?;
            let stream_data = stream_data
                .get("data")
                .ok_or_else(|| anyhow::anyhow!("data not found"))?
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("data is not an object"))?;
            let mut streams = Vec::new();
            // 遍历这个对象的所有键值对
            for (key, value) in stream_data.iter() {
                println!("key: {}, value: {}", key, value);
                let stream = Stream {
                    url: value
                        .pointer("/main/flv")
                        .ok_or_else(|| anyhow::anyhow!("flv not found"))?
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("flv is not a string"))?
                        .into(),
                    resolution: key.into(),
                    protocol: StreamingProtocol::Flv,
                };
                streams.push(stream);
            }
            let live_info = LiveInfo {
                url: url.into(),
                anchor_name: anchor_name.into(),
                status: LiveStatus::Live,
                streams,
            };
        }
        // println!("user: {:#?}", user);

        todo!()
    }

    fn kind(&self) -> PlatformKind {
        PlatformKind::Bilibili
    }
}

impl Tiktok {
    fn headers() -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            HeaderValue::from_str(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36 Edg/114.0.1823.79"
            )?,
        );
        headers.insert("Accept", HeaderValue::from_str("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")?);
        headers.insert("Cookie", HeaderValue::from_str("ttwid=1%7CM-rF193sJugKuNz2RGNt-rh6pAAR9IMceUSzlDnPCNI%7C1683274418%7Cf726d4947f2fc37fecc7aeb0cdaee52892244d04efde6f8a8edd2bb168263269; tiktok_webapp_theme=light; tt_chain_token=VWkygAWDlm1cFg/k8whmOg==; passport_csrf_token=6e422c5a7991f8cec7033a8082921510; passport_csrf_token_default=6e422c5a7991f8cec7033a8082921510; d_ticket=f8c267d4af4523c97be1ccb355e9991e2ae06; odin_tt=320b5f386cdc23f347be018e588873db7f7aea4ea5d1813681c3fbc018ea025dde957b94f74146dbc0e3612426b865ccb95ec8abe4ee36cca65f15dbffec0deff7b0e69e8ea536d46e0f82a4fc37d211; cmpl_token=AgQQAPNSF-RO0rT04baWtZ0T_jUjl4fVP4PZYM2QPw; uid_tt=319b558dbba684bb1557206c92089cd113a875526a89aee30595925d804b81c7; uid_tt_ss=319b558dbba684bb1557206c92089cd113a875526a89aee30595925d804b81c7; sid_tt=ad5e736f4bedb2f6d42ccd849e706b1d; sessionid=ad5e736f4bedb2f6d42ccd849e706b1d; sessionid_ss=ad5e736f4bedb2f6d42ccd849e706b1d; store-idc=useast5; store-country-code=us; store-country-code-src=uid; tt-target-idc=useast5; tt-target-idc-sign=qXNk0bb1pDQ0FbCNF120Pl9WWMLZg9Edv5PkfyCbS4lIk5ieW5tfLP7XWROnN0mEaSlc5hg6Oji1pF-yz_3ZXnUiNMrA9wNMPvI6D9IFKKVmq555aQzwPIGHv0aQC5dNRgKo5Z5LBkgxUMWEojTKclq2_L8lBciw0IGdhFm_XyVJtbqbBKKgybGDLzK8ZyxF4Jl_cYRXaDlshZjc38JdS6wruDueRSHe7YvNbjxCnApEFUv-OwJANSPU_4rvcqpVhq3JI2VCCfw-cs_4MFIPCDOKisk5EhAo2JlHh3VF7_CLuv80FXg_7ZqQ2pJeMOog294rqxwbbQhl3ATvjQV_JsWyUsMd9zwqecpylrPvtySI2u1qfoggx1owLrrUynee1R48QlanLQnTNW_z1WpmZBgVJqgEGLwFoVOmRzJuFFNj8vIqdjM2nDSdWqX8_wX3wplohkzkPSFPfZgjzGnQX28krhgTytLt7BXYty5dpfGtsdb11WOFHM6MZ9R9uLVB; sid_guard=ad5e736f4bedb2f6d42ccd849e706b1d%7C1690990657%7C15525213%7CMon%2C+29-Jan-2024+08%3A11%3A10+GMT; sid_ucp_v1=1.0.0-KGM3YzgwYjZhODgyYWI1NjIwNTA0NjBmOWUxMGRhMjIzYTI2YjMxNDUKGAiqiJ30keKD5WQQwfCppgYYsws4AkDsBxAEGgd1c2Vhc3Q1IiBhZDVlNzM2ZjRiZWRiMmY2ZDQyY2NkODQ5ZTcwNmIxZA; ssid_ucp_v1=1.0.0-KGM3YzgwYjZhODgyYWI1NjIwNTA0NjBmOWUxMGRhMjIzYTI2YjMxNDUKGAiqiJ30keKD5WQQwfCppgYYsws4AkDsBxAEGgd1c2Vhc3Q1IiBhZDVlNzM2ZjRiZWRiMmY2ZDQyY2NkODQ5ZTcwNmIxZA; tt_csrf_token=dD0EIH8q-pe3qDQsCyyD1jLN6KizJDRjOEyk; __tea_cache_tokens_1988={%22_type_%22:%22default%22%2C%22user_unique_id%22:%227229608516049831425%22%2C%22timestamp%22:1683274422659}; ttwid=1%7CM-rF193sJugKuNz2RGNt-rh6pAAR9IMceUSzlDnPCNI%7C1694002151%7Cd89b77afc809b1a610661a9d1c2784d80ebef9efdd166f06de0d28e27f7e4efe; msToken=KfJAVZ7r9D_QVeQlYAUZzDFbc1Yx-nZz6GF33eOxgd8KlqvTg1lF9bMXW7gFV-qW4MCgUwnBIhbiwU9kdaSpgHJCk-PABsHCtTO5J3qC4oCTsrXQ1_E0XtbqiE4OVLZ_jdF1EYWgKNPT2SnwGkQ=; msToken=KfJAVZ7r9D_QVeQlYAUZzDFbc1Yx-nZz6GF33eOxgd8KlqvTg1lF9bMXW7gFV-qW4MCgUwnBIhbiwU9kdaSpgHJCk-PABsHCtTO5J3qC4oCTsrXQ1_E0XtbqiE4OVLZ_jdF1EYWgKNPT2SnwGkQ=")?);
        Ok(headers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_stream_info() {
        // let url = "https://www.tiktok.com/@rumteenoladi/live";
        let url = "https://www.tiktok.com/@yermaaddd/live";
        let json_data = Tiktok.get_live_info(url).await.unwrap();
    }
}
