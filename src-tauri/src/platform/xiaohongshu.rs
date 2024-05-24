use crate::{
    model::{JsonValue, LiveInfo, LiveStatus, PlatformKind, Stream, StreamingProtocol},
    recorder::Recorder,
    request,
};
use anyhow::Result;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};

pub struct Xiaohongshu;

#[async_trait]
impl Recorder for Xiaohongshu {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Xiaohongshu
    }

    async fn get_live_info(&self, url: &str) -> Result<LiveInfo> {
        // 取 appuid 字段的值
        let appuid = url.split("appuid=").collect::<Vec<&str>>();
        if appuid.len() < 2 {
            return Err(anyhow::anyhow!("appuid not found"));
        }
        let appuid = appuid[1].split("&").collect::<Vec<&str>>()[0];
        // 使用路径参数的方式，取 livestream 字段的值
        let room_id = url.split("/livestream/").collect::<Vec<&str>>();
        if room_id.len() < 2 {
            return Err(anyhow::anyhow!("room_id not found"));
        }
        // 取 livestream 后面的数字，后面可能是 ? 或者 /，所以再次 split
        let room_id = room_id[1].split("?").collect::<Vec<&str>>()[0];
        let room_id = room_id.split("/").collect::<Vec<&str>>()[0];
        let app_api = format!("https://www.xiaohongshu.com/api/sns/red/live/app/v1/ecology/outside/share_info?room_id={}", room_id);
        let resp = request::get_with_headers(&app_api, Self::headers()?).await?;
        let body: JsonValue = serde_json::from_slice(&resp.bytes().await?)?;
        let anchor_name = body
            .pointer("/data/host_info/nickname")
            .unwrap_or(&JsonValue::Null)
            .as_str()
            .unwrap_or_default();
        let anchor_avatar = body
            .pointer("/data/host_info/avatar")
            .unwrap_or(&JsonValue::Null)
            .as_str()
            .unwrap_or_default()
            .split("?")
            .collect::<Vec<&str>>()[0];
        let room_cover = body
            .pointer("/data/room/cover")
            .unwrap_or(&JsonValue::Null)
            .as_str()
            .unwrap_or_default()
            .split("?")
            .collect::<Vec<&str>>()[0];
        let room_title = body
            .pointer("/data/room/name")
            .unwrap_or(&JsonValue::Null)
            .as_str()
            .unwrap_or_default();
        let flv_url = format!(
            "http://live-play.xhscdn.com/live/{}.flv?uid={}",
            room_id, appuid
        );
        let live_info = LiveInfo {
            url: url.into(),
            anchor_name: anchor_name.into(),
            anchor_avatar: anchor_avatar.into(),
            title: room_title.into(),
            status: LiveStatus::Unknown,
            viewer_count: "".into(),
            room_cover: room_cover.into(),
            platform_kind: PlatformKind::Xiaohongshu,
            streams: vec![Stream {
                url: flv_url,
                resolution: "default".into(),
                protocol: StreamingProtocol::Flv,
            }],
        };

        Ok(live_info)
    }
}

impl Xiaohongshu {
    pub fn new() -> Self {
        Self
    }

    pub fn headers() -> Result<HeaderMap> {
        let mut header_map = HeaderMap::new();
        header_map.insert(
            "User-Agent",
            HeaderValue::from_str(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0",
            )?,
        );
        header_map.insert(
            "Accept",
            HeaderValue::from_str("application/json, text/plain, */*")?,
        );
        header_map.insert(
            "Accept-Language",
            HeaderValue::from_str("zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en-US;q=0.3,en;q=0.2")?,
        );
        header_map.insert("Referer", HeaderValue::from_str("https://www.redelight.cn/hina/livestream/569077534207413574/1707413727088?share_source=&share_source_id=null&source=share_out_of_app&host_id=58bafe4282ec39085a56ece9&xhsshare=WeixinSession&appuid=5f3f478a00000000010005b3&apptime=1707413727")?);
        Ok(header_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_stream_info() {
        let url = "https://www.xiaohongshu.com/hina/livestream/569077534207413574/1707413727088?appuid=5f3f478a00000000010005b3&";
        // let url = "https://www.xiaohongshu.com/hina/livestream/569199937572834291?timestamp=1714706217433&share_source=&share_source_id=null&source=share_out_of_app&host_id=647f815b000000001003678a&xhsshare=QQ&appuid=5ed89960000000000101fdef&apptime=1714706218";
        let json_data = Xiaohongshu::new().get_live_info(url).await.unwrap();
        println!("{:#?}", json_data);
    }
}
