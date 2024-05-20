use anyhow::Result;
use chrono::Utc;
use std::{path::PathBuf, process::Child};

use dashmap::DashMap;
use log::{error, info};
use once_cell::sync::Lazy;

use crate::{
    ffmpeg, kv,
    model::{JsonValue, LiveStatus, RecordStatus, RecordingHistory, RecordingPlan},
    recorder, utils,
};

// 用一个 dashmap 用来保存已经开始的录制任务
pub static TASKS: Lazy<DashMap<String, Child>> = Lazy::new(|| DashMap::new());

/// 内部方法，不对外暴露
pub mod inner {
    use crate::model::{PlatformKind, Stream};

    use super::*;

    pub async fn start_record_default(room_url: &str) -> Result<()> {
        // 如果已经在录制了，就不再录制，返回错误
        if inner::get_record_status(room_url).await? == RecordStatus::Recording {
            return Err(anyhow::anyhow!("Already recording"));
        }
        let platform_impl = recorder::get_platform_impl(room_url)?;
        let live_info = platform_impl.get_live_info(room_url).await?;
        // 保存直播信息
        kv::live::add(&live_info).unwrap_or_else(|e| {
            error!("Could not add live info: {}", e);
        });
        // 如果不在播，就不录制
        if live_info.status == LiveStatus::NotLive {
            return Err(anyhow::anyhow!("该主播不在播"));
        }
        let stream = live_info
            .streams
            .first()
            .ok_or_else(|| anyhow::anyhow!("No stream found in live info: {:?}", live_info))?;
        let (path, filename) =
            utils::generate_path_and_filename(&platform_impl.kind(), &live_info.anchor_name)
                .await?;
        // 如果路径不存在，则创建
        if !std::path::Path::new(&path).exists() {
            std::fs::create_dir_all(&path)?;
        }

        // 使用系统提供的函数拼接路径和文件名
        let path = PathBuf::from(path);
        let full_filename = path.join(filename);
        let full_filename = full_filename
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Could not convert path to string: {:?}", path))?;

        inner::record_with_ffmpeg(room_url, &stream.url, full_filename).await?;

        // 记录录制历史
        let history = RecordingHistory::new(room_url, full_filename);
        kv::history::add(&history).unwrap_or_else(|e| {
            error!("Could not add recording history: {}", e);
        });

        Ok(())
    }

    /// 为 api 提供的录制方法，使用 api 传入的 stream 信息，不再获取直播间信息
    pub async fn start_record_with_stream(
        room_url: &str,
        stream: Stream,
        platform_kind: &PlatformKind,
        anchor_name: &str,
    ) -> Result<()> {
        // 如果已经在录制了，就不再录制，返回错误
        if inner::get_record_status(room_url).await? == RecordStatus::Recording {
            return Err(anyhow::anyhow!("Already recording"));
        }
        let (path, filename) =
            utils::generate_path_and_filename(platform_kind, anchor_name).await?;
        // 如果路径不存在，则创建
        if !std::path::Path::new(&path).exists() {
            std::fs::create_dir_all(&path)?;
        }

        // 使用系统提供的函数拼接路径和文件名
        let path = PathBuf::from(path);
        let full_filename = path.join(filename);
        let full_filename = full_filename
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Could not convert path to string: {:?}", path))?;

        inner::record_with_ffmpeg(room_url, &stream.url, full_filename).await?;

        // 记录录制历史
        let history = RecordingHistory::new(room_url, full_filename);
        kv::history::add(&history).unwrap_or_else(|e| {
            error!("Could not add recording history: {}", e);
        });

        Ok(())
    }

    pub(super) async fn record_with_ffmpeg(
        url: &str,
        stream_url: &str,
        full_filename: &str,
    ) -> Result<()> {
        let ffmpeg_path = kv::config::get()?.ffmpeg_path;
        let mut child = match ffmpeg::record(&ffmpeg_path, stream_url, full_filename) {
            Ok(child) => child,
            Err(e) => {
                error!("Could not start recording: {}", e);
                return Err(e);
            }
        };

        if let Some(status) = child.try_wait()? {
            let output = child.wait_with_output()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let error_message = format!(
                "status: {:?}\nstdout: {}\nstderr: {}",
                status, stdout, stderr
            );
            error!("可能不是预期的结果，但程序已退出：{}", error_message);
            return Err(anyhow::anyhow!(error_message));
        }

        // 将 child 插入到 TASKS 中
        TASKS.insert(url.into(), child);
        Ok(())
    }

    /// 获取录制状态，就是看对应 task 的 Child 是否还在运行
    pub(super) async fn get_record_status(url: &str) -> Result<RecordStatus> {
        // 任务状态都维护在数据库里，避免直接操作 TASKS
        // 从数据库获取状态
        if kv::recording::is_exist(url)? {
            Ok(RecordStatus::Recording)
        } else {
            Ok(RecordStatus::NotRecording)
        }
    }
}

pub mod record {
    use crate::model::{PlatformKind, Stream};

    use super::*;

    /// 开始录制，就是调用 ffmpeg::record 方法
    pub async fn start(
        url: &str,
        auto_record: bool,
        stream: Stream,
        platform_kind: PlatformKind,
        anchor_name: String,
    ) -> Result<JsonValue> {
        // 如果要自动录制，加入录制计划表
        if auto_record {
            let plan = RecordingPlan::new(url);
            kv::plan::add(&plan)?;
        }
        inner::start_record_with_stream(url, stream, &platform_kind, &anchor_name).await?;
        Ok(JsonValue::Null)
    }

    /// 停止录制，就是杀死对应 task 的 Child
    pub async fn stop(url: &str) -> Result<JsonValue> {
        if let Some(mut task) = TASKS.get_mut(url) {
            // 杀掉任务
            task.value_mut().kill().map_err(|e| {
                error!("Could not kill task: {}", e);
                e
            })?;
            // 杀掉任务后，变成了僵尸进程，通过 wait 来回收资源
            task.value_mut().wait().map_err(|e| {
                error!("Could not wait for task: {}", e);
                e
            })?;
            info!("停止录制成功：{}", url);
        }
        // 删除对应的 task
        TASKS.remove(url);
        // 更新录制历史
        kv::history::end(url)?;
        Ok(JsonValue::Null)
    }

    /// 获取录制状态，返回 JsonValue，用于 API
    pub async fn status(url: &str) -> Result<JsonValue> {
        Ok(serde_json::to_value(inner::get_record_status(url).await?)?)
    }
}

pub mod plan {
    use super::*;

    /// 获取所有录制计划
    pub async fn get_all() -> Result<JsonValue> {
        Ok(serde_json::to_value(kv::plan::get_all()?)?)
    }

    /// 新增录制计划
    pub async fn add(plan: &RecordingPlan) -> Result<JsonValue> {
        kv::plan::add(plan)?;
        Ok(JsonValue::Null)
    }

    /// 删除录制计划
    pub async fn delete(url: &str) -> Result<JsonValue> {
        kv::plan::delete(url)?;
        Ok(JsonValue::Null)
    }

    /// 更新录制计划状态
    pub async fn update_status(url: &str, enabled: bool) -> Result<JsonValue> {
        kv::plan::update_status(url, enabled)?;
        Ok(JsonValue::Null)
    }

    /// 获取上次轮询时间
    pub async fn get_last_polling_time() -> Result<JsonValue> {
        Ok(serde_json::to_value(kv::plan::get_last_polling_time()?)?)
    }
}

pub mod history {
    use crate::explorer;

    use super::*;

    /// 获取所有录制历史
    pub async fn get_all() -> Result<JsonValue> {
        let mut histories = kv::history::get_all()?;
        // 遍历列表，计算每个文件的尺寸，更新到 histories 中，仅在文件存在时才计算
        for history in &mut histories {
            if std::path::Path::new(&history.path).exists() {
                let file_size = std::fs::metadata(&history.path)?.len();
                history.file_size = file_size;
            } else {
                // 当开始录制于 20 秒前，但文件不存在，我们认为文件已经被删除
                if Utc::now().timestamp_millis() - history.start_time > 20000 {
                    history.deleted = true;
                }
            }
        }
        Ok(serde_json::to_value(histories)?)
    }

    /// 删除一条历史记录
    pub async fn delete(url: &str, start_time: i64) -> Result<JsonValue> {
        kv::history::delete(url, start_time)?;
        Ok(JsonValue::Null)
    }

    /// 在文件管理器中打开文件夹
    pub async fn open(path: &str) -> Result<JsonValue> {
        explorer::open_in_folder(path).map_err(|e| anyhow::anyhow!(e))?;
        Ok(JsonValue::Null)
    }
}

pub mod config {
    use super::*;

    /// 获取配置
    pub async fn get() -> Result<JsonValue> {
        Ok(serde_json::to_value(kv::config::get()?)?)
    }

    /// 设置配置
    pub async fn set(value: JsonValue) -> Result<JsonValue> {
        let config: crate::model::AppConfig = serde_json::from_value(value)?;
        kv::config::set(&config)?;
        Ok(JsonValue::Null)
    }
}

pub mod live {
    use super::*;

    /// 获取直播信息
    pub async fn info(url: &str) -> Result<JsonValue> {
        let platform_impl = recorder::get_platform_impl(url)?;
        let info = platform_impl.get_live_info(url).await?;
        // 保存到数据库
        let _ =
            kv::live::add(&info).map_err(|e| error!("Could not save live info to database: {}", e));
        Ok(serde_json::to_value(info)?)
    }
}

pub mod ffmpeg_api {
    use super::*;

    /// 检查 ffmpeg
    pub async fn check(ffmpeg_path: &str) -> Result<JsonValue> {
        Ok(JsonValue::String(ffmpeg::check_ffmpeg(ffmpeg_path)?))
    }
}
