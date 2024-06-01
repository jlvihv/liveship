use std::path::PathBuf;

use anyhow::{anyhow, Result};

pub fn config_dir() -> Result<PathBuf> {
    // 如果不存在则创建配置文件路径，位于家目录下的 .config/liveship
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("can not get config dir"))?
        .join("liveship");
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }
    Ok(config_dir)
}
