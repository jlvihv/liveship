use anyhow::{anyhow, Result};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use reqwest::header::{HeaderMap, HeaderValue};

use crate::{
    model::{JsonMap, JsonValue, LiveInfo, LiveStatus, PlatformKind, Stream, StreamingProtocol},
    recorder::Recorder,
    request,
};

static REGEX1: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r#"(\{\\"state\\":.*?)]\\n"]\)"#).unwrap());
static REGEX2: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r#""roomStore":(.*?),"linkmicStore""#).unwrap());
static REGEX3: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r#""nickname":"(.*?)","avatar_thumb"#).unwrap());

pub struct Douyin;

#[async_trait]
impl Recorder for Douyin {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Douyin
    }

    // 获取直播信息，需要返回直播状态、流地址、主播名、直播标题等信息
    async fn get_live_info(&self, room_url: &str) -> Result<LiveInfo> {
        let stream_info = Self::get_stream_info(room_url).await?;
        let stream_info = JsonValue::Object(stream_info);
        // 使用 json pointer 的方式从 json 中提取数据
        // 总是提供默认值，尽量不返回错误
        let status = stream_info
            .pointer("/status")
            .unwrap_or_else(|| &JsonValue::Null)
            .as_u64()
            .unwrap_or(4);
        // 状态 2 为直播中，4 为未直播
        let live_status = match status {
            2 => LiveStatus::Live,
            4 => LiveStatus::NotLive,
            _ => LiveStatus::Unknown,
        };
        let anchor_name = stream_info
            .pointer("/anchor_name")
            .unwrap_or_else(|| &JsonValue::Null)
            .as_str()
            .unwrap_or_default();
        // 如果当前不是直播状态，直接返回空信息，因为也获取不到
        if live_status != LiveStatus::Live {
            return Ok(LiveInfo {
                url: room_url.into(),
                anchor_name: anchor_name.into(),
                anchor_avatar: "".into(),
                title: "".into(),
                status: live_status,
                viewer_count: "".into(),
                room_cover: "".into(),
                streams: vec![],
                platform_kind: PlatformKind::Douyin,
            });
        }
        let title = stream_info
            .pointer("/title")
            .unwrap_or_else(|| &JsonValue::Null)
            .as_str()
            .unwrap_or_default();
        let anchor_avatar = stream_info
            .pointer("/owner/avatar_thumb/url_list/1")
            .unwrap_or_else(|| &JsonValue::Null)
            .as_str()
            .unwrap_or_default();
        let viewer_count = stream_info
            .pointer("/user_count_str")
            .unwrap_or_else(|| &JsonValue::Null)
            .as_str()
            .unwrap_or_default();
        let stream_url = stream_info
            .pointer("/stream_url")
            .ok_or_else(|| anyhow!("stream_url not found"))?
            .as_object()
            .ok_or_else(|| anyhow!("stream_url is not a object"))?
            .clone();
        // let default_resolution = stream_url
        //     .get("default_resolution")
        //     .unwrap_or_else(|| &JsonValue::Null)
        //     .as_str()
        //     .unwrap_or_default();
        let flv_url_map = stream_url
            .get("flv_pull_url")
            .ok_or_else(|| anyhow!("flv_pull_url not found"))?
            .as_object()
            .ok_or_else(|| anyhow!("flv_pull_url is not a object"))?;
        let hls_url_map = stream_url
            .get("hls_pull_url_map")
            .ok_or_else(|| anyhow!("hls_pull_url_map not found"))?
            .as_object()
            .ok_or_else(|| anyhow!("hls_pull_url_map is not a object"))?;
        let mut streams = vec![];
        flv_url_map.iter().for_each(|(resolution, url)| {
            streams.push(Stream {
                resolution: resolution.to_string(),
                url: url.as_str().unwrap_or_default().to_string(),
                protocol: StreamingProtocol::Flv,
            });
        });
        hls_url_map.iter().for_each(|(resolution, url)| {
            streams.push(Stream {
                resolution: resolution.to_string(),
                url: url.as_str().unwrap_or_default().to_string(),
                protocol: StreamingProtocol::Hls,
            });
        });

        Ok(LiveInfo {
            url: room_url.into(),
            anchor_name: anchor_name.into(),
            anchor_avatar: anchor_avatar.into(),
            title: title.into(),
            status: live_status,
            viewer_count: viewer_count.into(),
            room_cover: "".into(),
            platform_kind: PlatformKind::Douyin,
            streams,
        })
    }
}

impl Douyin {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_stream_info(url: &str) -> Result<JsonMap> {
        let body = request::get_with_headers(&url, Self::headers()?).await?;
        let body_string = body.text().await?;
        // 匹配其中的 json 数据
        let json_str = match REGEX1.captures(&body_string) {
            Some(caps) => caps
                .get(1)
                .ok_or_else(|| anyhow!("json data not found"))?
                .as_str(),
            None => return Err(anyhow!("json data not found")),
        };
        // 清理 json 字符串
        let json_str = json_str.replace("\\", "").replace("u0026", "&");

        let room_store = match REGEX2.captures(&json_str) {
            Some(caps) => caps
                .get(1)
                .ok_or_else(|| anyhow!("roomStore not found"))?
                .as_str(),
            None => return Err(anyhow!("roomStore not found")),
        };
        let anchor_name = match REGEX3.captures(&room_store) {
            Some(caps) => caps
                .get(1)
                .ok_or_else(|| anyhow!("anchor_name not found"))?
                .as_str(),
            None => return Err(anyhow!("anchor_name not found")),
        };
        let room_store = format!(
            "{}{}",
            room_store
                .split(r#","has_commerce_goods""#)
                .next()
                .ok_or_else(|| anyhow!("room_store not found"))?,
            "}}}",
        );
        let json_data: JsonValue = serde_json::from_str(&room_store)?;
        let mut json_data = json_data
            .pointer("/roomInfo/room")
            .unwrap_or(&JsonValue::Null)
            .as_object()
            .ok_or_else(|| anyhow!("room is not a object"))?
            .clone();
        json_data.insert(
            "anchor_name".to_string(),
            JsonValue::String(anchor_name.to_string()),
        );
        Ok(json_data)
    }

    pub fn headers() -> Result<HeaderMap> {
        let headers = [
            (
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/115.0",
            ),
            (
                "Accept-Language",
                "zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en-US;q=0.3,en;q=0.2",
            ),
            ("Referer", "https://live.douyin.com/"),
            ("Cookie", "ttwid=1%7CB1qls3GdnZhUov9o2NxOMxxYS2ff6OSvEWbv0ytbES4%7C1680522049%7C280d802d6d478e3e78d0c807f7c487e7ffec0ae4e5fdd6a0fe74c3c6af149511; my_rd=1; passport_csrf_token=3ab34460fa656183fccfb904b16ff742; passport_csrf_token_default=3ab34460fa656183fccfb904b16ff742; d_ticket=9f562383ac0547d0b561904513229d76c9c21; n_mh=hvnJEQ4Q5eiH74-84kTFUyv4VK8xtSrpRZG1AhCeFNI; store-region=cn-fj; store-region-src=uid; LOGIN_STATUS=1; __security_server_data_status=1; FORCE_LOGIN=%7B%22videoConsumedRemainSeconds%22%3A180%7D; pwa2=%223%7C0%7C3%7C0%22; download_guide=%223%2F20230729%2F0%22; volume_info=%7B%22isUserMute%22%3Afalse%2C%22isMute%22%3Afalse%2C%22volume%22%3A0.6%7D; strategyABtestKey=%221690824679.923%22; stream_recommend_feed_params=%22%7B%5C%22cookie_enabled%5C%22%3Atrue%2C%5C%22screen_width%5C%22%3A1536%2C%5C%22screen_height%5C%22%3A864%2C%5C%22browser_online%5C%22%3Atrue%2C%5C%22cpu_core_num%5C%22%3A8%2C%5C%22device_memory%5C%22%3A8%2C%5C%22downlink%5C%22%3A10%2C%5C%22effective_type%5C%22%3A%5C%224g%5C%22%2C%5C%22round_trip_time%5C%22%3A150%7D%22; VIDEO_FILTER_MEMO_SELECT=%7B%22expireTime%22%3A1691443863751%2C%22type%22%3Anull%7D; home_can_add_dy_2_desktop=%221%22; __live_version__=%221.1.1.2169%22; device_web_cpu_core=8; device_web_memory_size=8; xgplayer_user_id=346045893336; csrf_session_id=2e00356b5cd8544d17a0e66484946f28; odin_tt=724eb4dd23bc6ffaed9a1571ac4c757ef597768a70c75fef695b95845b7ffcd8b1524278c2ac31c2587996d058e03414595f0a4e856c53bd0d5e5f56dc6d82e24004dc77773e6b83ced6f80f1bb70627; __ac_nonce=064caded4009deafd8b89; __ac_signature=_02B4Z6wo00f01HLUuwwAAIDBh6tRkVLvBQBy9L-AAHiHf7; ttcid=2e9619ebbb8449eaa3d5a42d8ce88ec835; webcast_leading_last_show_time=1691016922379; webcast_leading_total_show_times=1; webcast_local_quality=sd; live_can_add_dy_2_desktop=%221%22; msToken=1JDHnVPw_9yTvzIrwb7cQj8dCMNOoesXbA_IooV8cezcOdpe4pzusZE7NB7tZn9TBXPr0ylxmv-KMs5rqbNUBHP4P7VBFUu0ZAht_BEylqrLpzgt3y5ne_38hXDOX8o=; msToken=jV_yeN1IQKUd9PlNtpL7k5vthGKcHo0dEh_QPUQhr8G3cuYv-Jbb4NnIxGDmhVOkZOCSihNpA2kvYtHiTW25XNNX_yrsv5FN8O6zm3qmCIXcEe0LywLn7oBO2gITEeg=; tt_scid=mYfqpfbDjqXrIGJuQ7q-DlQJfUSG51qG.KUdzztuGP83OjuVLXnQHjsz-BRHRJu4e986"),
        ];
        let mut header_map = HeaderMap::new();
        for &(k, v) in headers.iter() {
            header_map.insert(k, HeaderValue::from_str(v)?);
        }
        Ok(header_map)
    }
}
