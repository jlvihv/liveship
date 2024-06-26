use chrono::Utc;
use serde::{Deserialize, Serialize};
use strum::Display;

pub type JsonValue = serde_json::Value;
pub type JsonMap = serde_json::Map<String, JsonValue>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum HttpBody {
    String(String),
    Object(JsonMap),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveInfo {
    pub url: String,
    // 主播名
    pub anchor_name: String,
    // 主播头像
    pub anchor_avatar: String,
    // 直播间标题
    pub title: String,
    // 直播状态
    pub status: LiveStatus,
    // 多少人正在看
    pub viewer_count: String,
    // 直播间封面，如果没有，就是空字符串
    pub room_cover: String,
    // 直播流地址信息
    pub streams: Vec<Stream>,
    // 直播平台
    pub platform_kind: PlatformKind,
}

// 存储设置，用来指明保存位置，文件名等信息
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageSetting {
    // 保存路径
    pub path: String,
    // 文件名
    pub filename: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub url: String,
    pub resolution: String,
    pub protocol: StreamingProtocol,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum StreamingProtocol {
    Flv,
    Hls,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum LiveStatus {
    Live,
    NotLive,
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiResponse {
    pub code: i32,
    pub message: String,
    pub data: JsonValue,
}

#[derive(Debug, Deserialize)]
pub struct StorageSettingRequest {
    pub platform_kind: PlatformKind,
    pub anchor_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRecordRequest {
    pub url: String,
    pub auto_record: bool,
    pub stream: Stream,
    pub platform_kind: PlatformKind,
    pub anchor_name: String,
}

#[allow(unused)]
pub struct Message {
    pub url: String,
    pub action: Action,
}

#[allow(unused)]
pub enum Action {
    // 开始录制
    StartRecord,
    // 停止录制
    StopRecord,
    // 暂停录制
    PauseRecord,
    // 继续录制
    ResumeRecord,
    // 获取录制状态
    GetRecordStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum RecordStatus {
    // 录制中
    Recording,
    // 未录制
    NotRecording,
}

// 直播平台类型
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Display)]
pub enum PlatformKind {
    Douyin,
    Tiktok,
    Xiaohongshu,
    Bilibili,
    Huya,
    Kuaishou,
    Douyu,
    Twitch,
    Youtube,
    Unknown,
}

// 录制记录
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingHistory {
    pub url: String,
    // 录制状态
    pub status: RecordStatus,
    // 开始时间，时间戳毫秒数
    pub start_time: i64,
    // 结束时间，时间戳毫秒数，如果是 0, 说明还在录制
    pub end_time: i64,
    // 保存路径
    pub path: String,
    // 以下字段不保存到数据库，也不从数据库读取，每次都重新计算
    // 文件尺寸
    pub file_size: u64,
    // 是否已删除
    pub deleted: bool,
    // 直播间信息
    pub live_info: Option<LiveInfo>,
}

// 录制计划
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingPlan {
    pub url: String,
    // 录制策略
    pub strategy: RecordingStrategy,
    pub stream_protocol: StreamingProtocol,
    pub stream_resolution: String,
    pub enabled: bool,
    // 创建于，时间戳 i64
    pub created_at: i64,
    // 更新于，时间戳 i64，如果是 0, 说明没有更新过
    pub updated_at: i64,
    // 直播间信息
    pub live_info: Option<LiveInfo>,
    // 录制选项
    #[serde(default)]
    pub option: RecordingOption,
}

// 录制选项
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RecordingOption {
    // 使用代理
    pub use_proxy: Option<String>,
    // 自动转 mp4
    pub auto_convert_to_mp4: bool,
    // 删除原文件
    pub delete_original_file: bool,
}

// 录制策略
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum RecordingStrategy {
    // 定时录制，指定开始时间和结束时间
    Timed(i64, i64),
    // 定时录制，但是只录制指定时长，单位秒
    TimedWithDuration(i64, i64),
    // 定时录制，直到主播下播
    TimedUntilAnchorEnd(i64),
    // 主播开播就录制，直到主播下播
    AnchorLive,
    // 主播开播就录制，指定录制时长，单位秒
    AnchorLiveWithDuration(i64),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    // ffmpeg 可执行文件路径
    pub ffmpeg_path: String,
    // 文件保存路径
    pub save_path: String,
    // 开播信息检查间隔，单位秒
    pub live_info_check_interval: u64,
}

pub mod config {
    use super::*;

    impl Default for AppConfig {
        fn default() -> Self {
            let video_path = dirs::video_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "./".to_string());
            // 使用系统视频路径加 "liveship" 作为默认保存路径
            let save_path = std::path::Path::new(&video_path)
                .join("liveship")
                .to_string_lossy()
                .to_string();
            Self {
                ffmpeg_path: "ffmpeg".into(),
                save_path,
                live_info_check_interval: 60,
            }
        }
    }
}

pub mod history {
    use super::*;

    impl RecordingHistory {
        pub fn new(url: &str, path: &str) -> Self {
            Self {
                url: url.into(),
                path: path.into(),
                status: RecordStatus::Recording,
                start_time: Utc::now().timestamp_millis(),
                end_time: 0,
                file_size: 0,
                deleted: false,
                live_info: None,
            }
        }
    }
}

pub mod plan {
    use super::*;

    impl RecordingPlan {
        pub fn new(
            url: &str,
            stream_protocol: StreamingProtocol,
            stream_resolution: String,
            option: Option<RecordingOption>,
        ) -> RecordingPlan {
            RecordingPlan {
                url: url.into(),
                strategy: crate::model::RecordingStrategy::AnchorLive,
                enabled: true,
                created_at: Utc::now().timestamp_millis(),
                updated_at: 0,
                live_info: None,
                stream_protocol,
                stream_resolution,
                option: option.unwrap_or_default(),
            }
        }

        pub fn new_with_url(url: &str) -> RecordingPlan {
            RecordingPlan::new(url, StreamingProtocol::Flv, "".into(), None)
        }
    }
}

pub mod platform {
    use super::*;

    // 为 PlatformKind 实现 From 方法，用来从网址字符串转换成 PlatformKind
    impl<T: AsRef<str>> From<T> for PlatformKind {
        fn from(url: T) -> Self {
            let url = url.as_ref().to_lowercase();
            match url {
                _ if url.starts_with("https://live.douyin.com/")
                    || url.starts_with("https://v.douyin.com/")
                    || url == "douyin" =>
                {
                    PlatformKind::Douyin
                }
                _ if url.starts_with("https://www.tiktok.com/") || url == "tiktok" => {
                    PlatformKind::Tiktok
                }
                _ if url.starts_with("https://www.xiaohongshu.com/") || url == "xiaohongshu" => {
                    PlatformKind::Xiaohongshu
                }
                _ if url.starts_with("https://www.huya.com/") || url == "huya" => {
                    PlatformKind::Huya
                }
                _ => PlatformKind::Unknown,
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfig {
    pub enabled: bool,
    pub address: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryHistory {
    pub url: String,
    pub anchor_name: String,
    pub platform_kind: String,
    pub created_at: i64,
}
