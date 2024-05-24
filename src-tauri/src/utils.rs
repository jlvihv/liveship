use crate::{kv, model::PlatformKind};
use anyhow::Result;
use std::path::PathBuf;

/// 生成文件名
pub fn generate_filename(anchor_name: &str) -> String {
    let anchor_name = anchor_name
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();
    let time = chrono::Local::now();
    let time_str = time.format("%Y%m%d_%H%M%S").to_string();
    format!("{}_{}.ts", anchor_name, time_str)
}

/// 生成文件路径，路径是配置中的路径 + 平台名文件夹 + 主播名文件夹
pub async fn generate_path(platform_kind: &PlatformKind, anchor_name: &str) -> Result<String> {
    // let path = db::config::get().await?.save_path;
    let path = kv::config::get()?.save_path;
    let path = PathBuf::from(path);
    let platform_name = platform_kind.to_string();
    let anchor_name = anchor_name
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();
    let full_path = path.join(platform_name).join(anchor_name);
    Ok(full_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Could not convert path to string: {:?}", full_path))?
        .to_string())
}

pub async fn generate_path_and_filename(
    platform_kind: &PlatformKind,
    anchor_name: &str,
) -> Result<(String, String)> {
    let filename = generate_filename(anchor_name);
    let path = generate_path(platform_kind, anchor_name).await?;
    Ok((path, filename))
}
