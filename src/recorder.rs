use anyhow::Result;
use async_trait::async_trait;

use crate::{
    model::{LiveInfo, PlatformKind},
    platform::{Douyin, Xiaohongshu},
};

// 录制器 trait
// 每个新添加的直播平台，都要实现这个 trait
#[async_trait]
pub trait Recorder {
    // 获取直播信息
    async fn get_live_info(&self, url: &str) -> Result<LiveInfo>;

    // 获取平台类型
    fn kind(&self) -> PlatformKind;
}

/// 获取对应平台的 trait 实现
pub fn get_platform_impl(url: &str) -> Result<Box<dyn Recorder + Send + Sync>> {
    let platform_kind = PlatformKind::from(url);
    match platform_kind {
        PlatformKind::Douyin => Ok(Box::new(Douyin::new())),
        PlatformKind::Xiaohongshu => Ok(Box::new(Xiaohongshu::new())),
        _ => Err(anyhow::anyhow!("Unknown platform: {}", url)),
    }
}
